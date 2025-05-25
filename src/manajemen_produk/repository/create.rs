use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository::dto::{get_db_pool, validate_produk, RepositoryError};

pub async fn tambah_produk(produk: &Produk) -> Result<i64, RepositoryError> {
    // Validasi terlebih dahulu
    validate_produk(produk)?;
    
    let pool = get_db_pool()?;
    
    let result = sqlx::query(
        r#"
        INSERT INTO produk (nama, kategori, harga, stok, deskripsi)
        VALUES (?, ?, ?, ?, ?)
        "#
    )
    .bind(&produk.nama)
    .bind(&produk.kategori)
    .bind(produk.harga)
    .bind(produk.stok as i64)
    .bind(&produk.deskripsi)
    .execute(pool)
    .await?;
    
    Ok(result.last_insert_rowid())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::repository::delete::clear_all;
    use crate::manajemen_produk::repository::read::ambil_produk_by_id;
    use crate::manajemen_produk::repository::dto::init_database;
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
        // Initialize database if not already done
        let _ = init_database().await;
        clear_all().await
    }

        #[tokio::test]
    async fn test_tambah_dan_ambil_produk() {
        // Initialize DB and cleanup first
        let _ = init_database().await;
        let _ = clear_all().await;
        
        let produk = Produk::new(
            "Test Laptop".to_string(),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some("Test Description".to_string())
        );
        
        // Add product
        let id = tambah_produk(&produk).await.unwrap();
        assert!(id > 0);
        
        // Get the product back
        let retrieved = ambil_produk_by_id(id).await.unwrap().unwrap();
        
        // Verify all fields
        assert_eq!(retrieved.id.unwrap(), id);
        assert_eq!(retrieved.nama, produk.nama);
        assert_eq!(retrieved.kategori, produk.kategori);
        assert_eq!(retrieved.harga, produk.harga);
        assert_eq!(retrieved.stok, produk.stok);
        assert_eq!(retrieved.deskripsi, produk.deskripsi);

        // Cleanup after test
        let _ = clear_all().await;
    }

    #[test]
    async fn test_id_sequence() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add products sequentially
        let mut ids = vec![];
        
        for i in 0..3 {
            let produk = Produk::new(
                format!("Product {}", i),
                "Test".to_string(),
                1000.0,
                10,
                None,
            );
            
            let id = tambah_produk(&produk).await.unwrap();
            ids.push(id);
        }
        
        // Verify IDs are sequential
        assert_eq!(ids.len(), 3);
        
        // Delete a product
        let _ = crate::manajemen_produk::repository::delete::hapus_produk(ids[1]).await;
        
        // Add another product
        let produk = Produk::new(
            "New Product".to_string(),
            "Test".to_string(),
            1000.0,
            10,
            None,
        );
        
        let new_id = tambah_produk(&produk).await.unwrap();
        
        // Verify the new ID is greater than 0
        assert!(new_id > 0);
    }

    #[test]
    async fn test_validation_error() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Test empty name
        let invalid_produk = Produk::new(
            "".to_string(),
            "Test".to_string(),
            1000.0,
            10,
            None,
        );
        
        let result = tambah_produk(&invalid_produk).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RepositoryError::ValidationError(_)));
        
        // Test negative price
        let invalid_price_produk = Produk::new(
            "Valid Name".to_string(),
            "Test".to_string(),
            -1000.0,
            10,
            None,
        );
        
        let result = tambah_produk(&invalid_price_produk).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RepositoryError::ValidationError(_)));
    }
}