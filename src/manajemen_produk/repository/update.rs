use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository::dto::{get_db_pool, validate_produk, RepositoryError};

pub async fn update_produk(id: i64, produk: &Produk) -> Result<bool, RepositoryError> {
    // Validasi input
    validate_produk(produk)?;
    
    let pool = get_db_pool()?;
    
    let result = sqlx::query(
        r#"
        UPDATE produk 
        SET nama = $1, kategori = $2, harga = $3, stok = $4, deskripsi = $5
        WHERE id = $6
        "#
    )
    .bind(&produk.nama)
    .bind(&produk.kategori)
    .bind(produk.harga)
    .bind(produk.stok as i32)
    .bind(&produk.deskripsi)
    .bind(id)
    .execute(pool)
    .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}

pub async fn update_stok(id: i64, new_stok: u32) -> Result<bool, RepositoryError> {
    let pool = get_db_pool()?;
    
    let result = sqlx::query("UPDATE produk SET stok = $1 WHERE id = $2")
        .bind(new_stok as i32)
        .bind(id)
        .execute(pool)
        .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}

pub async fn update_harga(id: i64, new_harga: f64) -> Result<bool, RepositoryError> {
    if new_harga < 0.0 {
        return Err(RepositoryError::ValidationError("Harga tidak boleh negatif".to_string()));
    }
    
    let pool = get_db_pool()?;
    
    let result = sqlx::query("UPDATE produk SET harga = $1 WHERE id = $2")
        .bind(new_harga)
        .bind(id)
        .execute(pool)
        .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::repository::create::tambah_produk;
    use crate::manajemen_produk::repository::read::ambil_produk_by_id;
    use crate::manajemen_produk::repository::delete::clear_all;
    use crate::manajemen_produk::repository::dto::init_database;
    use tokio::test;
    use std::sync::Once;
    use uuid::Uuid; // Add uuid = "1.0" to Cargo.toml for unique test data

    static INIT: Once = Once::new();

    fn create_test_product() -> Produk {
        let unique_id = Uuid::new_v4().to_string()[..8].to_string();
        Produk::new(
            format!("Laptop Gaming {}", unique_id),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some(format!("Laptop dengan RTX 4060 {}", unique_id)),
        )
    }

    fn create_test_product_with_name(name: &str) -> Produk {
        let unique_id = Uuid::new_v4().to_string()[..8].to_string();
        Produk::new(
            format!("{} {}", name, unique_id),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some(format!("Product description {}", unique_id)),
        )
    }

    async fn setup_test_environment() -> Result<(), RepositoryError> {
        // Initialize database only once
        INIT.call_once(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let _ = init_database().await;
            });
        });
        Ok(())
    }

    async fn cleanup_specific_product(id: i64) -> Result<(), RepositoryError> {
        // Clean up specific product instead of all data
        let pool = get_db_pool()?;
        let _ = sqlx::query("DELETE FROM produk WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_update_produk_success() {
        setup_test_environment().await.expect("Failed to setup test environment");
        
        // Create initial product with unique data
        let mut produk = create_test_product();
        let id = tambah_produk(&produk).await
            .expect("Failed to create test product");
        
        // Update product data
        let unique_suffix = Uuid::new_v4().to_string()[..8].to_string();
        produk.nama = format!("Updated Laptop Gaming {}", unique_suffix);
        produk.harga = 17_500_000.0;
        produk.stok = 15;
        produk.deskripsi = Some(format!("Updated laptop dengan RTX 4070 {}", unique_suffix));
        
        // Perform update
        let update_result = update_produk(id, &produk).await
            .expect("Failed to update product");
        assert!(update_result, "Update should return true on success");
        
        // Verify the update
        let updated_product = ambil_produk_by_id(id).await
            .expect("Failed to fetch updated product")
            .expect("Product should exist after update");
        
        assert_eq!(updated_product.nama, produk.nama);
        assert_eq!(updated_product.harga, 17_500_000.0);
        assert_eq!(updated_product.stok, 15);
        assert_eq!(updated_product.deskripsi, produk.deskripsi);
        assert_eq!(updated_product.kategori, "Elektronik"); // Should remain unchanged
        
        // Clean up this specific test data
        let _ = cleanup_specific_product(id).await;
    }

    #[tokio::test]
    async fn test_update_produk_not_found() {
        setup_test_environment().await.expect("Failed to setup test environment");
        
        let produk = create_test_product();
        // Use a very high ID that's unlikely to exist
        let non_existent_id = i64::MAX - 1000;
        
        let update_result = update_produk(non_existent_id, &produk).await;
        
        assert!(update_result.is_err(), "Update should fail for non-existent product");
        match update_result {
            Err(RepositoryError::NotFound) => {
                // Expected behavior
            },
            Err(other_error) => panic!("Expected NotFound error, got: {:?}", other_error),
            Ok(_) => panic!("Expected error for non-existent product"),
        }
    }

    #[tokio::test]
    async fn test_update_stok_success() {
        setup_test_environment().await.expect("Failed to setup test environment");
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await
            .expect("Failed to create test product");
        
        let new_stok = 25_u32;
        let update_result = update_stok(id, new_stok).await
            .expect("Failed to update stock");
        assert!(update_result, "Stock update should return true on success");
        
        // Verify only stock was updated
        let updated_product = ambil_produk_by_id(id).await
            .expect("Failed to fetch updated product")
            .expect("Product should exist after stock update");
        
        assert_eq!(updated_product.stok, new_stok);
        assert_eq!(updated_product.nama, produk.nama); // Should remain unchanged
        assert_eq!(updated_product.harga, produk.harga); // Should remain unchanged
        assert_eq!(updated_product.kategori, produk.kategori); // Should remain unchanged
        
        let _ = cleanup_specific_product(id).await;
    }

    #[tokio::test]
    async fn test_update_stok_not_found() {
        setup_test_environment().await.expect("Failed to setup test environment");
        
        let non_existent_id = i64::MAX - 2000;
        let new_stok = 25_u32;
        
        let update_result = update_stok(non_existent_id, new_stok).await;
        
        assert!(update_result.is_err(), "Stock update should fail for non-existent product");
        match update_result {
            Err(RepositoryError::NotFound) => {
                // Expected behavior
            },
            Err(other_error) => panic!("Expected NotFound error, got: {:?}", other_error),
            Ok(_) => panic!("Expected error for non-existent product"),
        }
    }

    #[tokio::test]
    async fn test_update_harga_success() {
        setup_test_environment().await.expect("Failed to setup test environment");
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await
            .expect("Failed to create test product");
        
        let new_harga = 18_750_000.0;
        let update_result = update_harga(id, new_harga).await
            .expect("Failed to update price");
        assert!(update_result, "Price update should return true on success");
        
        // Verify only price was updated
        let updated_product = ambil_produk_by_id(id).await
            .expect("Failed to fetch updated product")
            .expect("Product should exist after price update");
        
        assert_eq!(updated_product.harga, new_harga);
        assert_eq!(updated_product.nama, produk.nama); // Should remain unchanged
        assert_eq!(updated_product.stok, produk.stok); // Should remain unchanged
        assert_eq!(updated_product.kategori, produk.kategori); // Should remain unchanged
        
        let _ = cleanup_specific_product(id).await;
    }

    #[tokio::test]
    async fn test_update_harga_negative_validation() {
        setup_test_environment().await.expect("Failed to setup test environment");
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await
            .expect("Failed to create test product");
        
        let negative_price = -1000.0;
        let update_result = update_harga(id, negative_price).await;
        
        assert!(update_result.is_err(), "Negative price should be rejected");
        match update_result {
            Err(RepositoryError::ValidationError(msg)) => {
                assert_eq!(msg, "Harga tidak boleh negatif", "Validation message should match");
            },
            Err(other_error) => panic!("Expected ValidationError, got: {:?}", other_error),
            Ok(_) => panic!("Expected validation error for negative price"),
        }
        
        // Verify original price is unchanged
        let unchanged_product = ambil_produk_by_id(id).await
            .expect("Failed to fetch product")
            .expect("Product should still exist");
        assert_eq!(unchanged_product.harga, produk.harga, "Original price should be unchanged");
        
        let _ = cleanup_specific_product(id).await;
    }

    #[tokio::test]
    async fn test_update_harga_not_found() {
        setup_test_environment().await.expect("Failed to setup test environment");
        
        let non_existent_id = i64::MAX - 3000;
        let new_harga = 20_000_000.0;
        
        let update_result = update_harga(non_existent_id, new_harga).await;
        
        assert!(update_result.is_err(), "Price update should fail for non-existent product");
        match update_result {
            Err(RepositoryError::NotFound) => {
                // Expected behavior
            },
            Err(other_error) => panic!("Expected NotFound error, got: {:?}", other_error),
            Ok(_) => panic!("Expected error for non-existent product"),
        }
    }

    #[tokio::test]
    async fn test_update_with_invalid_product_data() {
        setup_test_environment().await.expect("Failed to setup test environment");
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await
            .expect("Failed to create test product");
        
        // Create product with invalid data (empty name)
        let invalid_produk = Produk::new(
            "".to_string(), // Invalid: empty name
            "Elektronik".to_string(), 
            15_000_000.0, 
            10, 
            None
        );
        
        let update_result = update_produk(id, &invalid_produk).await;
        
        assert!(update_result.is_err(), "Update with invalid data should fail");
        match update_result {
            Err(RepositoryError::ValidationError(_)) => {
                // Expected behavior - validation should catch empty name
            },
            Err(other_error) => panic!("Expected ValidationError, got: {:?}", other_error),
            Ok(_) => panic!("Expected validation error for invalid product data"),
        }
        
        // Verify original product data remains unchanged
        let unchanged_product = ambil_produk_by_id(id).await
            .expect("Failed to fetch product")
            .expect("Product should still exist");
        assert_eq!(unchanged_product.nama, produk.nama, "Product name should be unchanged");
        assert_eq!(unchanged_product.harga, produk.harga, "Product price should be unchanged");
        assert_eq!(unchanged_product.stok, produk.stok, "Product stock should be unchanged");
        
        let _ = cleanup_specific_product(id).await;
    }

    #[tokio::test]
    async fn test_update_produk_with_postgresql_types() {
        setup_test_environment().await.expect("Failed to setup test environment");
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await
            .expect("Failed to create test product");
        
        // Test with PostgreSQL-specific edge cases
        let unique_suffix = Uuid::new_v4().to_string()[..8].to_string();
        let updated_produk = Produk::new(
            format!("Laptop Gaming RGB Xtreme Pro Max Ultra {}", unique_suffix), // Long name with unique suffix
            "Elektronik & Komputer".to_string(), // Name with special characters
            99_999_999.99, // Large decimal price
            999999, // Large stock number
            Some(format!("Deskripsi panjang dengan karakter khusus: àáâãäåæçèéêë & symbols: @#$%^&*() {}", unique_suffix)),
        );
        
        let update_result = update_produk(id, &updated_produk).await
            .expect("Failed to update product with PostgreSQL edge case data");
        assert!(update_result, "Update should succeed with valid PostgreSQL data");
        
        // Verify PostgreSQL handled the data correctly
        let fetched_product = ambil_produk_by_id(id).await
            .expect("Failed to fetch updated product")
            .expect("Product should exist");
        
        assert_eq!(fetched_product.nama, updated_produk.nama);
        assert_eq!(fetched_product.kategori, updated_produk.kategori);
        assert_eq!(fetched_product.harga, updated_produk.harga);
        assert_eq!(fetched_product.stok, updated_produk.stok);
        assert_eq!(fetched_product.deskripsi, updated_produk.deskripsi);
        
        let _ = cleanup_specific_product(id).await;
    }
}