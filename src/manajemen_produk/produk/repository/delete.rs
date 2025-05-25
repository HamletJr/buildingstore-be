use crate::manajemen_produk::produk::repository::helper::{get_db_pool, RepositoryError};

pub async fn hapus_produk(id: i64) -> Result<bool, RepositoryError> {
    let pool = get_db_pool()?;
    
    let result = sqlx::query("DELETE FROM products WHERE id = ?")
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
    sqlx::query("DELETE FROM products")
        .execute(&mut *tx)
        .await?;
    
    // Reset auto-increment counter
    sqlx::query("DELETE FROM sqlite_sequence WHERE name = 'products'")
        .execute(&mut *tx)
        .await?;
    
    // Commit transaction
    tx.commit().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::produk::model::Produk;
    use crate::manajemen_produk::produk::repository::create::tambah_produk;
    use crate::manajemen_produk::produk::repository::read::{ambil_produk_by_id, ambil_semua_produk};
    use tokio::test;

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
        clear_all().await
    }

    #[test]
    async fn test_hapus_produk() {
        let _ = cleanup_repository().await;
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await.unwrap();
        
        let delete_result = hapus_produk(id).await.unwrap();
        assert!(delete_result);
        
        let retrieved = ambil_produk_by_id(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    async fn test_hapus_produk_tidak_ada() {
        let _ = cleanup_repository().await;
        
        let delete_result = hapus_produk(9999).await.unwrap();
        assert!(!delete_result);
    }

    #[test]
    async fn test_clear_all() {
        let _ = cleanup_repository().await;
        
        // Add test products
        let _ = tambah_produk(&create_test_product()).await.unwrap();
        let _ = tambah_produk(&create_test_product()).await.unwrap();
        
        let before_clear = ambil_semua_produk().await.unwrap();
        assert_eq!(before_clear.len(), 2);
        
        let clear_result = clear_all().await;
        assert!(clear_result.is_ok());
        
        let after_clear = ambil_semua_produk().await.unwrap();
        assert!(after_clear.is_empty());
    }

    #[test]
    async fn test_counter_reset_after_clear() {
        let _ = cleanup_repository().await;
        
        let id1 = tambah_produk(&create_test_product()).await.unwrap();
        let _ = tambah_produk(&create_test_product()).await.unwrap();
        assert!(id1 > 0);
        
        let clear_result = clear_all().await;
        assert!(clear_result.is_ok());
        
        let new_product = Produk::new("New Product".to_string(), "Test".to_string(), 1000.0, 10, None);
        let new_id = tambah_produk(&new_product).await.unwrap();
        assert_eq!(new_id, 1);
    }
}