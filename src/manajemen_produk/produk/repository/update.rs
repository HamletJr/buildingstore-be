use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::repository::helper::{lock_store_mut, validate_produk, RepositoryError};

pub async fn update_produk(id: i64, produk: &Produk) -> Result<bool, RepositoryError> {
    // Validasi input
    validate_produk(produk)?;
    
    let mut store = lock_store_mut()?;
    
    if store.contains_key(&id) {
        let updated_produk = Produk::with_id(
            id,
            produk.nama.clone(),
            produk.kategori.clone(),
            produk.harga,
            produk.stok,
            produk.deskripsi.clone(),
        );
        
        store.insert(id, updated_produk);
        Ok(true)
    } else {
        Err(RepositoryError::NotFound)
    }
}

pub async fn update_stok(id: i64, new_stok: u32) -> Result<bool, RepositoryError> {
    let mut store = lock_store_mut()?;
    
    if let Some(produk) = store.get(&id).cloned() {
        let updated_produk = Produk::with_id(
            id,
            produk.nama,
            produk.kategori,
            produk.harga,
            new_stok,
            produk.deskripsi,
        );
        
        store.insert(id, updated_produk);
        Ok(true)
    } else {
        Err(RepositoryError::NotFound)
    }
}

pub async fn update_harga(id: i64, new_harga: f64) -> Result<bool, RepositoryError> {
    if new_harga < 0.0 {
        return Err(RepositoryError::ValidationError("Harga tidak boleh negatif".to_string()));
    }
    
    let mut store = lock_store_mut()?;
    
    if let Some(produk) = store.get(&id).cloned() {
        let updated_produk = Produk::with_id(
            id,
            produk.nama,
            produk.kategori,
            new_harga,
            produk.stok,
            produk.deskripsi,
        );
        
        store.insert(id, updated_produk);
        Ok(true)
    } else {
        Err(RepositoryError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::produk::repository::create::tambah_produk;
    use crate::manajemen_produk::produk::repository::read::ambil_produk_by_id;
    use crate::manajemen_produk::produk::repository::delete::clear_all;
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

    // Test updating a product using direct function
    #[test]
    async fn test_update_produk_direct() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a product
        let mut produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Update the product using direct function
        produk.nama = "Updated Laptop Direct".to_string();
        produk.harga = 17_000_000.0;
        produk.stok = 12;
        produk.deskripsi = Some("Updated description direct".to_string());
        
        let update_result = update_produk(id, &produk).await.unwrap();
        assert!(update_result);
        
        // Retrieve and verify update
        let updated = ambil_produk_by_id(id).await.unwrap().unwrap();
        
        assert_eq!(updated.nama, "Updated Laptop Direct");
        assert_eq!(updated.harga, 17_000_000.0);
        assert_eq!(updated.stok, 12);
        assert_eq!(updated.deskripsi, Some("Updated description direct".to_string()));
    }

    // Test updating non-existent product using direct function
    #[test]
    async fn test_update_produk_tidak_ada_direct() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Try to update non-existent product
        let produk = create_test_products()[0].clone();
        let update_result = update_produk(9999, &produk).await;
        
        // Should return NotFound error
        assert!(update_result.is_err());
        match update_result {
            Err(RepositoryError::NotFound) => {}, // Expected
            _ => panic!("Expected NotFound error"),
        }
    }

    // Test updating stock only
    #[test]
    async fn test_update_stok() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a product
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Update stock only
        let new_stok = 25;
        let update_result = update_stok(id, new_stok).await.unwrap();
        assert!(update_result);
        
        // Retrieve and verify stock update
        let updated = ambil_produk_by_id(id).await.unwrap().unwrap();
        
        assert_eq!(updated.stok, new_stok);
        // Other fields should remain unchanged
        assert_eq!(updated.nama, produk.nama);
        assert_eq!(updated.kategori, produk.kategori);
        assert_eq!(updated.harga, produk.harga);
        assert_eq!(updated.deskripsi, produk.deskripsi);
    }

    // Test updating stock for non-existent product
    #[test]
    async fn test_update_stok_tidak_ada() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Try to update stock for non-existent product
        let update_result = update_stok(9999, 50).await;
        
        // Should return NotFound error
        assert!(update_result.is_err());
        match update_result {
            Err(RepositoryError::NotFound) => {}, // Expected
            _ => panic!("Expected NotFound error"),
        }
    }

    // Test updating price only
    #[test]
    async fn test_update_harga() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a product
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Update price only
        let new_harga = 18_000_000.0;
        let update_result = update_harga(id, new_harga).await.unwrap();
        assert!(update_result);
        
        // Retrieve and verify price update
        let updated = ambil_produk_by_id(id).await.unwrap().unwrap();
        
        assert_eq!(updated.harga, new_harga);
        // Other fields should remain unchanged
        assert_eq!(updated.nama, produk.nama);
        assert_eq!(updated.kategori, produk.kategori);
        assert_eq!(updated.stok, produk.stok);
        assert_eq!(updated.deskripsi, produk.deskripsi);
    }

    // Test updating price for non-existent product
    #[test]
    async fn test_update_harga_tidak_ada() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Try to update price for non-existent product
        let update_result = update_harga(9999, 1_000_000.0).await;
        
        // Should return NotFound error
        assert!(update_result.is_err());
        match update_result {
            Err(RepositoryError::NotFound) => {}, // Expected
            _ => panic!("Expected NotFound error"),
        }
    }

    // Test updating price with negative value
    #[test]
    async fn test_update_harga_negatif() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a product
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Try to update with negative price
        let update_result = update_harga(id, -1000.0).await;
        
        // Should return ValidationError
        assert!(update_result.is_err());
        match update_result {
            Err(RepositoryError::ValidationError(msg)) => {
                assert_eq!(msg, "Harga tidak boleh negatif");
            },
            _ => panic!("Expected ValidationError"),
        }
    }

    // Test updating price with zero value (should be allowed)
    #[test]
    async fn test_update_harga_nol() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a product
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Update with zero price (should be allowed)
        let update_result = update_harga(id, 0.0).await.unwrap();
        assert!(update_result);
        
        // Retrieve and verify price update
        let updated = ambil_produk_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.harga, 0.0);
    }

    // Test concurrent updates
    #[test]
    async fn test_concurrent_updates() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a product
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Create multiple concurrent update tasks
        let mut handles = vec![];
        
        // Concurrent stock updates
        for i in 1..=5 {
            let handle = tokio::spawn(async move {
                update_stok(id, i * 10).await
            });
            handles.push(handle);
        }
        
        // Concurrent price updates
        for i in 1..=5 {
            let handle = tokio::spawn(async move {
                update_harga(id, (i as f64) * 1_000_000.0).await
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
        
        // Verify product still exists and has valid values
        let updated = ambil_produk_by_id(id).await.unwrap().unwrap();
        assert!(updated.stok > 0);
        assert!(updated.harga > 0.0);
    }

    // Test update with invalid product data
    #[test]
    async fn test_update_with_invalid_data() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a valid product
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Try to update with invalid data (empty name)
        let invalid_produk = Produk::new(
            "".to_string(), // Empty name should be invalid
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some("Description".to_string()),
        );
        
        let update_result = update_produk(id, &invalid_produk).await;
        
        // Should return validation error
        assert!(update_result.is_err());
        match update_result {
            Err(RepositoryError::ValidationError(_)) => {}, // Expected
            _ => panic!("Expected ValidationError"),
        }
        
        // Original product should remain unchanged
        let original = ambil_produk_by_id(id).await.unwrap().unwrap();
        assert_eq!(original.nama, produk.nama);
    }

    // Test multiple sequential updates - FIXED VERSION
    #[test]
    async fn test_sequential_updates() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a product
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Perform sequential updates
        for i in 1..=5 {
            // Update stock
            let _ = update_stok(id, (i * 5) as u32).await.unwrap();
            
            // Update price
            let _ = update_harga(id, (i as f64) * 1_500_000.0).await.unwrap();
            
            // Get the current state of the product before updating
            let current_product = ambil_produk_by_id(id).await.unwrap().unwrap();
            
            // Create updated product with current stock and price preserved
            let updated_produk = Produk::with_id(
                id,
                format!("Updated Product {}", i),
                current_product.kategori,
                current_product.harga, // Keep the updated price
                current_product.stok,  // Keep the updated stock
                Some(format!("Description {}", i)),
            );
            
            let _ = update_produk(id, &updated_produk).await.unwrap();
        }
        
        // Verify final state
        let final_product = ambil_produk_by_id(id).await.unwrap().unwrap();
        assert_eq!(final_product.nama, "Updated Product 5");
        assert_eq!(final_product.stok, 25);
        assert_eq!(final_product.harga, 7_500_000.0);
        assert_eq!(final_product.deskripsi, Some("Description 5".to_string()));
    }
}