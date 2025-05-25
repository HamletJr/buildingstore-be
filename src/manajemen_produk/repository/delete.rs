use crate::manajemen_produk::repository::dto::{get_db_pool, RepositoryError};

pub async fn hapus_produk(id: i64) -> Result<bool, RepositoryError> {
    let pool = get_db_pool()?;
    
    let result = sqlx::query("DELETE FROM produk WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected() > 0)
}

pub async fn clear_all() -> Result<(), RepositoryError> {
    let pool = get_db_pool()?;
    
    // Start transaction
    let mut tx = pool.begin().await?;
    
    // Clear all products
    sqlx::query("DELETE FROM produk")
        .execute(&mut *tx)
        .await?;
    
    // Reset sequence counter (PostgreSQL way)
    sqlx::query("ALTER SEQUENCE produk_id_seq RESTART WITH 1")
        .execute(&mut *tx)
        .await?;
    
    // Commit transaction
    tx.commit().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::model::Produk;
    use crate::manajemen_produk::repository::create::tambah_produk;
    use crate::manajemen_produk::repository::read::{ambil_produk_by_id, ambil_semua_produk};
    use crate::manajemen_produk::repository::dto::init_database;

    fn create_test_product() -> Produk {
        Produk::new(
            "Laptop Gaming".to_string(),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some("Laptop dengan RTX 4060".to_string()),
        )
    }

    async fn cleanup_repository() -> Result<(), RepositoryError> {
        // Always initialize database first
        init_database().await?;
        clear_all().await
    }

    #[tokio::test]
    async fn test_hapus_produk() {
        let _ = cleanup_repository().await;
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Verify product exists before deletion
        let before_delete = ambil_produk_by_id(id).await.unwrap();
        assert!(before_delete.is_some());
        
        // Delete the product
        let delete_result = hapus_produk(id).await.unwrap();
        assert!(delete_result);
        
        // Verify product no longer exists
        let after_delete = ambil_produk_by_id(id).await.unwrap();
        assert!(after_delete.is_none());
        
        // Cleanup
        let _ = clear_all().await;
    }

    #[tokio::test]
    async fn test_hapus_produk_tidak_ada() {
        let _ = cleanup_repository().await;
        
        // Try to delete non-existent product
        let delete_result = hapus_produk(9999).await.unwrap();
        assert!(!delete_result);
        
        // Cleanup
        let _ = clear_all().await;
    }

    #[tokio::test]
    async fn test_clear_all() {
        let _ = cleanup_repository().await;
        
        // Add test products
        let id1 = tambah_produk(&create_test_product()).await.unwrap();
        let id2 = tambah_produk(&create_test_product()).await.unwrap();
        
        // Verify products exist
        let before_clear = ambil_semua_produk().await.unwrap();
        assert_eq!(before_clear.len(), 2);
        assert_eq!(before_clear[0].id.unwrap(), id1);
        assert_eq!(before_clear[1].id.unwrap(), id2);
        
        // Clear all products
        let clear_result = clear_all().await;
        assert!(clear_result.is_ok());
        
        // Verify all products are deleted
        let after_clear = ambil_semua_produk().await.unwrap();
        assert!(after_clear.is_empty());
        
        // Verify individual products are also gone
        let product1 = ambil_produk_by_id(id1).await.unwrap();
        let product2 = ambil_produk_by_id(id2).await.unwrap();
        assert!(product1.is_none());
        assert!(product2.is_none());
    }

    #[tokio::test]
    async fn test_counter_reset_after_clear() {
        let _ = cleanup_repository().await;
        
        // Add some products
        let id1 = tambah_produk(&create_test_product()).await.unwrap();
        let id2 = tambah_produk(&create_test_product()).await.unwrap();
        
        // Verify initial IDs start from 1 (PostgreSQL BIGSERIAL)
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        
        // Clear all products and reset sequence
        let clear_result = clear_all().await;
        assert!(clear_result.is_ok());
        
        // Add a new product after clearing
        let new_product = Produk::new(
            "New Product".to_string(), 
            "Test".to_string(), 
            1000.0, 
            10, 
            None
        );
        let new_id = tambah_produk(&new_product).await.unwrap();
        
        // Verify sequence was reset to 1
        assert_eq!(new_id, 1);
        
        // Add another product to verify sequence continues properly
        let another_product = Produk::new(
            "Another Product".to_string(), 
            "Test".to_string(), 
            2000.0, 
            5, 
            None
        );
        let another_id = tambah_produk(&another_product).await.unwrap();
        assert_eq!(another_id, 2);
        
        // Cleanup
        let _ = clear_all().await;
    }

    #[tokio::test]
    async fn test_delete_specific_product_from_multiple() {
        let _ = cleanup_repository().await;
        
        // Add multiple products
        let mut products = Vec::new();
        let mut ids = Vec::new();
        
        for i in 1..=5 {
            let product = Produk::new(
                format!("Product {}", i),
                "Test".to_string(),
                1000.0 * i as f64,
                10 + i,
                Some(format!("Description {}", i))
            );
            products.push(product);
        }
        
        for product in &products {
            let id = tambah_produk(product).await.unwrap();
            ids.push(id);
        }
        
        // Verify all products exist
        let all_products = ambil_semua_produk().await.unwrap();
        assert_eq!(all_products.len(), 5);
        
        // Delete middle product (index 2, ID 3)
        let delete_result = hapus_produk(ids[2]).await.unwrap();
        assert!(delete_result);
        
        // Verify only 4 products remain
        let remaining_products = ambil_semua_produk().await.unwrap();
        assert_eq!(remaining_products.len(), 4);
        
        // Verify the specific product was deleted
        let deleted_product = ambil_produk_by_id(ids[2]).await.unwrap();
        assert!(deleted_product.is_none());
        
        // Verify other products still exist
        for (index, &id) in ids.iter().enumerate() {
            if index != 2 { // Skip the deleted one
                let product = ambil_produk_by_id(id).await.unwrap();
                assert!(product.is_some());
            }
        }
        
        // Cleanup
        let _ = clear_all().await;
    }
}