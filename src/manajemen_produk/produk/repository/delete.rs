use crate::manajemen_produk::produk::repository::helper::{lock_store_mut, lock_counter, RepositoryError};

pub async fn hapus_produk(id: i64) -> Result<bool, RepositoryError> {
    let mut store = lock_store_mut()?;
    Ok(store.remove(&id).is_some())
}

pub async fn clear_all() -> Result<(), RepositoryError> {
    let mut store = lock_store_mut()?;
    store.clear();

    let mut counter = lock_counter()?;
    *counter = 0;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::produk::model::Produk;
    use crate::manajemen_produk::produk::repository::create::tambah_produk;
    use crate::manajemen_produk::produk::repository::read::ambil_produk_by_id;
    use crate::manajemen_produk::produk::repository::read::ambil_semua_produk;
    use tokio::test;

    // Helper function to create test products
    fn create_test_products() -> Vec<Produk> {
        vec![
            Produk::new(
                "Laptop Gaming".to_string(),
                "Elektronik".to_string(),
                15_000_000.0,
                10,
                Some("Laptop dengan RTX 4060".to_string()),
            ),
            Produk::new(
                "Cat Tembok".to_string(),
                "Material".to_string(),
                150_000.0,
                50,
                Some("Cat tembok anti air".to_string()),
            ),
            Produk::new(
                "Smartphone".to_string(),
                "Elektronik".to_string(),
                8_000_000.0,
                20,
                Some("Smartphone dengan kamera 108MP".to_string()),
            ),
        ]
    }

    // Helper function to clean repository between tests
    async fn cleanup_repository() -> Result<(), RepositoryError> {
        clear_all().await
    }

    // Test deleting a product using direct function
    #[test]
    async fn test_hapus_produk_direct() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a product
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Delete the product using direct function
        let delete_result = hapus_produk(id).await.unwrap();
        assert!(delete_result);
        
        // Verify product is deleted
        let retrieved = ambil_produk_by_id(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    // Test deleting non-existent product using direct function
    #[test]
    async fn test_hapus_produk_tidak_ada_direct() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Try to delete non-existent product using direct function
        let delete_result = hapus_produk(9999).await.unwrap();
        
        // Should return false but not error
        assert!(!delete_result);
    }

    // Test clearing all products using direct function
    #[test]
    async fn test_clear_all_direct() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add test products
        let test_products = create_test_products();
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        // Verify products exist
        let before_clear = ambil_semua_produk().await.unwrap();
        assert_eq!(before_clear.len(), test_products.len());
        
        // Clear all products using direct function
        let clear_result = clear_all().await;
        assert!(clear_result.is_ok());
        
        // Verify all products are gone
        let after_clear = ambil_semua_produk().await.unwrap();
        assert!(after_clear.is_empty());
    }

    // Test clearing empty repository
    #[test]
    async fn test_clear_all_empty_repository() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Verify repository is empty
        let before_clear = ambil_semua_produk().await.unwrap();
        assert!(before_clear.is_empty());
        
        // Clear empty repository
        let clear_result = clear_all().await;
        assert!(clear_result.is_ok());
        
        // Verify repository is still empty
        let after_clear = ambil_semua_produk().await.unwrap();
        assert!(after_clear.is_empty());
    }

    // Test multiple deletions
    #[test]
    async fn test_multiple_deletions() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add test products
        let test_products = create_test_products();
        let mut product_ids = vec![];
        
        for product in &test_products {
            let id = tambah_produk(product).await.unwrap();
            product_ids.push(id);
        }
        
        // Verify all products exist
        let all_products = ambil_semua_produk().await.unwrap();
        assert_eq!(all_products.len(), test_products.len());
        
        // Delete products one by one
        for id in &product_ids {
            let delete_result = hapus_produk(*id).await.unwrap();
            assert!(delete_result);
        }
        
        // Verify all products are deleted
        let remaining_products = ambil_semua_produk().await.unwrap();
        assert!(remaining_products.is_empty());
        
        // Try to delete again (should return false)
        for id in &product_ids {
            let delete_result = hapus_produk(*id).await.unwrap();
            assert!(!delete_result);
        }
    }

    // Test concurrent deletions
    #[test]
    async fn test_concurrent_deletions() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add multiple products
        let mut product_ids = vec![];
        for i in 0..10 {
            let produk = Produk::new(
                format!("Product {}", i),
                "Test".to_string(),
                1000.0 * (i as f64 + 1.0),
                10 * (i + 1),
                Some(format!("Description {}", i)),
            );
            
            let id = tambah_produk(&produk).await.unwrap();
            product_ids.push(id);
        }
        
        // Verify all products exist
        let all_products = ambil_semua_produk().await.unwrap();
        assert_eq!(all_products.len(), 10);
        
        // Delete products concurrently
        let mut handles = vec![];
        
        for id in product_ids {
            let handle = tokio::spawn(async move {
                hapus_produk(id).await
            });
            handles.push(handle);
        }
        
        // Wait for all deletions to complete
        let mut successful_deletions = 0;
        for handle in handles {
            let result = handle.await.unwrap().unwrap();
            if result {
                successful_deletions += 1;
            }
        }
        
        // All deletions should be successful
        assert_eq!(successful_deletions, 10);
        
        // Verify all products are deleted
        let remaining_products = ambil_semua_produk().await.unwrap();
        assert!(remaining_products.is_empty());
    }

    // Test deletion after update
    #[test]
    async fn test_hapus_after_update() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a product
        let mut produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Update the product
        produk.nama = "Updated Product".to_string();
        produk.harga = 20_000_000.0;
        let update_result = crate::manajemen_produk::produk::repository::update::update_produk(id, &produk).await.unwrap();
        assert!(update_result);
        
        // Verify update was successful
        let updated = ambil_produk_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.nama, "Updated Product");
        assert_eq!(updated.harga, 20_000_000.0);
        
        // Delete the updated product
        let delete_result = hapus_produk(id).await.unwrap();
        assert!(delete_result);
        
        // Verify product is deleted
        let retrieved = ambil_produk_by_id(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    // Test counter reset after clear_all
    #[test]
    async fn test_counter_reset_after_clear() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add some products to increment counter
        let test_products = create_test_products();
        let mut last_id = 0;
        
        for product in &test_products {
            let id = tambah_produk(product).await.unwrap();
            last_id = id;
        }
        
        // Verify counter has been incremented
        assert!(last_id >= test_products.len() as i64);
        
        // Clear all products (should reset counter)
        let clear_result = clear_all().await;
        assert!(clear_result.is_ok());
        
        // Add a new product after clear
        let new_product = Produk::new(
            "New Product".to_string(),
            "Test".to_string(),
            1000.0,
            10,
            None,
        );
        
        let new_id = tambah_produk(&new_product).await.unwrap();
        
        // New ID should start from 1 (counter was reset)
        assert_eq!(new_id, 1);
    }

    // Test selective deletion with remaining products
    #[test]
    async fn test_selective_deletion() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add test products
        let test_products = create_test_products();
        let mut product_ids = vec![];
        
        for product in &test_products {
            let id = tambah_produk(product).await.unwrap();
            product_ids.push(id);
        }
        
        // Delete only the middle product
        let middle_id = product_ids[1];
        let delete_result = hapus_produk(middle_id).await.unwrap();
        assert!(delete_result);
        
        // Verify only middle product is deleted
        let remaining_products = ambil_semua_produk().await.unwrap();
        assert_eq!(remaining_products.len(), 2);
        
        // Verify first and last products still exist
        let first_product = ambil_produk_by_id(product_ids[0]).await.unwrap();
        assert!(first_product.is_some());
        
        let last_product = ambil_produk_by_id(product_ids[2]).await.unwrap();
        assert!(last_product.is_some());
        
        // Verify middle product is gone
        let middle_product = ambil_produk_by_id(middle_id).await.unwrap();
        assert!(middle_product.is_none());
    }

    // Test deletion with negative ID
    #[test]
    async fn test_hapus_produk_negative_id() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Try to delete product with negative ID
        let delete_result = hapus_produk(-1).await.unwrap();
        
        // Should return false (not found)
        assert!(!delete_result);
    }

    // Test deletion with zero ID
    #[test]
    async fn test_hapus_produk_zero_id() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Try to delete product with zero ID
        let delete_result = hapus_produk(0).await.unwrap();
        
        // Should return false (not found)
        assert!(!delete_result);
    }

    // Test multiple clear_all operations
    #[test]
    async fn test_multiple_clear_all() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add products
        let test_products = create_test_products();
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        // Clear multiple times
        for _ in 0..3 {
            let clear_result = clear_all().await;
            assert!(clear_result.is_ok());
            
            let products = ambil_semua_produk().await.unwrap();
            assert!(products.is_empty());
        }
        
        // Add a product after multiple clears
        let new_product = Produk::new(
            "After Clear Product".to_string(),
            "Test".to_string(),
            1000.0,
            10,
            None,
        );
        
        let new_id = tambah_produk(&new_product).await.unwrap();
        assert_eq!(new_id, 1); // Counter should still be reset
    }
}