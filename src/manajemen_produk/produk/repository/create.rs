use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::repository::helper::{get_db_pool, validate_produk, RepositoryError};

pub async fn tambah_produk(produk: &Produk) -> Result<i64, RepositoryError> {
    // Validasi terlebih dahulu
    validate_produk(produk)?;
    
    let pool = get_db_pool()?;
    
    let result = sqlx::query(
        r#"
        INSERT INTO products (nama, kategori, harga, stok, deskripsi)
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

pub async fn tambah_batch_produk(produk_list: &[Produk]) -> Result<Vec<i64>, RepositoryError> {
    // Validasi semua produk terlebih dahulu
    for produk in produk_list {
        validate_produk(produk)?;
    }
    
    let pool = get_db_pool()?;
    let mut ids = Vec::new();
    
    // Start transaction
    let mut tx = pool.begin().await?;
    
    for produk in produk_list {
        let result = sqlx::query(
            r#"
            INSERT INTO products (nama, kategori, harga, stok, deskripsi)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(&produk.nama)
        .bind(&produk.kategori)
        .bind(produk.harga)
        .bind(produk.stok as i64)
        .bind(&produk.deskripsi)
        .execute(&mut *tx)
        .await?;
        
        ids.push(result.last_insert_rowid());
    }
    
    // Commit transaction
    tx.commit().await?;
    
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::produk::repository::delete::clear_all;
    use crate::manajemen_produk::produk::repository::read::ambil_produk_by_id;
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

    #[test]
    async fn test_tambah_dan_ambil_produk() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Create test product
        let produk = create_test_products()[0].clone();
        
        // Add product to repository
        let result = tambah_produk(&produk).await;
        assert!(result.is_ok());
        
        let id = result.unwrap();
        assert!(id > 0); // ID should be positive
        
        // Retrieve product
        let retrieved = ambil_produk_by_id(id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_produk = retrieved.unwrap();
        assert_eq!(retrieved_produk.id.unwrap(), id);
        assert_eq!(retrieved_produk.nama, produk.nama);
        assert_eq!(retrieved_produk.kategori, produk.kategori);
        assert_eq!(retrieved_produk.harga, produk.harga);
        assert_eq!(retrieved_produk.stok, produk.stok);
        assert_eq!(retrieved_produk.deskripsi, produk.deskripsi);
    }

    #[test]
    async fn test_tambah_batch_produk() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Create test products
        let test_products = create_test_products();
        
        // Add products in batch
        let result = tambah_batch_produk(&test_products).await;
        assert!(result.is_ok());
        
        let ids = result.unwrap();
        assert_eq!(ids.len(), test_products.len());
        
        // Verify all IDs are unique and positive
        for id in &ids {
            assert!(*id > 0);
        }
        
        // Verify IDs are sequential
        for i in 1..ids.len() {
            assert_eq!(ids[i-1] + 1, ids[i]);
        }
        
        // Verify all products were added correctly
        for (i, id) in ids.iter().enumerate() {
            let retrieved = ambil_produk_by_id(*id).await.unwrap();
            assert!(retrieved.is_some());
            
            let retrieved_produk = retrieved.unwrap();
            assert_eq!(retrieved_produk.nama, test_products[i].nama);
            assert_eq!(retrieved_produk.kategori, test_products[i].kategori);
        }
    }

    #[test]
    async fn test_concurrent_operations() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Create multiple products concurrently
        let mut handles = vec![];
        
        for i in 0..5 {
            let handle = tokio::spawn(async move {
                let produk = Produk::new(
                    format!("Product {}", i),
                    "Test".to_string(),
                    1000.0 * (i as f64 + 1.0),
                    10 * (i + 1),
                    Some(format!("Description {}", i)),
                );
                
                tambah_produk(&produk).await
            });
            
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
        
        // Verify all products were added
        let all_products = crate::manajemen_produk::produk::repository::read::ambil_semua_produk().await.unwrap();
        assert_eq!(all_products.len(), 5);
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
        assert_eq!(ids[0] + 1, ids[1]);
        assert_eq!(ids[1] + 1, ids[2]);
        
        // Delete a product
        let _ = crate::manajemen_produk::produk::repository::delete::hapus_produk(ids[1]).await;
        
        // Add another product
        let produk = Produk::new(
            "New Product".to_string(),
            "Test".to_string(),
            1000.0,
            10,
            None,
        );
        
        let new_id = tambah_produk(&produk).await.unwrap();
        
        // Verify the new ID is greater than the last used ID
        assert!(new_id > ids[2]);
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