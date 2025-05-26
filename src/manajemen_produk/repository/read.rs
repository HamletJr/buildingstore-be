use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository::dto::{get_db_pool, row_to_produk, RepositoryError};

pub async fn ambil_semua_produk() -> Result<Vec<Produk>, RepositoryError> {
    let pool = get_db_pool()?;
    
    let rows = sqlx::query("SELECT * FROM produk ORDER BY id")
        .fetch_all(pool)
        .await?;
    
    let mut products = Vec::new();
    for row in rows {
        products.push(row_to_produk(&row)?);
    }
    
    Ok(products)
}

pub async fn ambil_produk_by_id(id: i64) -> Result<Option<Produk>, RepositoryError> {
    let pool = get_db_pool()?;
    
    let row = sqlx::query("SELECT * FROM produk WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    
    match row {
        Some(row) => Ok(Some(row_to_produk(&row)?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::repository::create::tambah_produk;
    use crate::manajemen_produk::repository::delete::clear_all;
    use crate::manajemen_produk::repository::dto::init_database;

    fn create_test_products() -> Vec<Produk> {
        vec![
            Produk::new(
                "Laptop Gaming".to_string(), 
                "Elektronik".to_string(), 
                15_000_000.0, 
                10, 
                Some("Laptop dengan RTX 4060".to_string())
            ),
            Produk::new(
                "Cat Tembok".to_string(), 
                "Material".to_string(), 
                150_000.0, 
                50, 
                Some("Cat tembok anti air".to_string())
            ),
            Produk::new(
                "Smartphone".to_string(), 
                "Elektronik".to_string(), 
                8_000_000.0, 
                20, 
                Some("Smartphone dengan kamera 108MP".to_string())
            ),
        ]
    }

    async fn setup_and_cleanup_repository() -> Result<(), RepositoryError> {
        // Always initialize database first
        init_database().await?;
        clear_all().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ambil_semua_produk() {
        let _ = setup_and_cleanup_repository().await;
        
        let test_products = create_test_products();
        let mut inserted_ids = Vec::new();
        
        // Insert all test products
        for product in &test_products {
            let id = tambah_produk(product).await.unwrap();
            inserted_ids.push(id);
        }
        
        // Retrieve all products
        let all_products = ambil_semua_produk().await.unwrap();
        assert_eq!(all_products.len(), test_products.len());
        
        // Verify products are ordered by ID (PostgreSQL ORDER BY id)
        for (index, product) in all_products.iter().enumerate() {
            assert_eq!(product.id.unwrap(), inserted_ids[index]);
            assert_eq!(product.nama, test_products[index].nama);
            assert_eq!(product.kategori, test_products[index].kategori);
            assert_eq!(product.harga, test_products[index].harga);
            assert_eq!(product.stok, test_products[index].stok);
            assert_eq!(product.deskripsi, test_products[index].deskripsi);
        }
        
        // Cleanup
        let _ = clear_all().await;
    }

    #[tokio::test]
    async fn test_ambil_produk_by_id() {
        let _ = setup_and_cleanup_repository().await;
        
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Test retrieving existing product
        let retrieved = ambil_produk_by_id(id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_produk = retrieved.unwrap();
        assert_eq!(retrieved_produk.id.unwrap(), id);
        assert_eq!(retrieved_produk.nama, produk.nama);
        assert_eq!(retrieved_produk.kategori, produk.kategori);
        assert_eq!(retrieved_produk.harga, produk.harga);
        assert_eq!(retrieved_produk.stok, produk.stok);
        assert_eq!(retrieved_produk.deskripsi, produk.deskripsi);
        
        // Cleanup
        let _ = clear_all().await;
    }

    #[tokio::test]
    async fn test_ambil_produk_by_id_tidak_ada() {
        let _ = setup_and_cleanup_repository().await;
        
        // Test retrieving non-existent product
        let retrieved = ambil_produk_by_id(9999).await.unwrap();
        assert!(retrieved.is_none());
        
        // Test with ID 0
        let retrieved_zero = ambil_produk_by_id(0).await.unwrap();
        assert!(retrieved_zero.is_none());
        
        // Test with negative ID
        let retrieved_negative = ambil_produk_by_id(-1).await.unwrap();
        assert!(retrieved_negative.is_none());
        
        // Cleanup
        let _ = clear_all().await;
    }

    #[tokio::test]
    async fn test_ambil_semua_produk_kosong() {
        let _ = setup_and_cleanup_repository().await;
        
        // Test retrieving from empty database
        let all_products = ambil_semua_produk().await.unwrap();
        assert!(all_products.is_empty());
        
        // Cleanup
        let _ = clear_all().await;
    }

    #[tokio::test]
    async fn test_retrieve_multiple_categories() {
        let _ = setup_and_cleanup_repository().await;
        
        // Create products with different categories
        let elektronik_products = vec![
            Produk::new("Laptop".to_string(), "Elektronik".to_string(), 10_000_000.0, 5, None),
            Produk::new("HP".to_string(), "Elektronik".to_string(), 5_000_000.0, 15, None),
        ];
        
        let material_products = vec![
            Produk::new("Semen".to_string(), "Material".to_string(), 50_000.0, 100, None),
            Produk::new("Batu Bata".to_string(), "Material".to_string(), 1_000.0, 500, None),
        ];
        
        let mut all_inserted_ids = Vec::new();
        
        // Insert elektronik products
        for product in &elektronik_products {
            let id = tambah_produk(product).await.unwrap();
            all_inserted_ids.push(id);
        }
        
        // Insert material products
        for product in &material_products {
            let id = tambah_produk(product).await.unwrap();
            all_inserted_ids.push(id);
        }
        
        // Retrieve all products
        let all_products = ambil_semua_produk().await.unwrap();
        assert_eq!(all_products.len(), 4);
        
        // Verify products are returned in ID order
        for (index, product) in all_products.iter().enumerate() {
            assert_eq!(product.id.unwrap(), all_inserted_ids[index]);
        }
        
        // Verify we can retrieve each product individually
        for &id in &all_inserted_ids {
            let individual_product = ambil_produk_by_id(id).await.unwrap();
            assert!(individual_product.is_some());
        }
        
        // Count products by category
        let elektronik_count = all_products.iter()
            .filter(|p| p.kategori == "Elektronik")
            .count();
        let material_count = all_products.iter()
            .filter(|p| p.kategori == "Material")
            .count();
            
        assert_eq!(elektronik_count, 2);
        assert_eq!(material_count, 2);
        
        // Cleanup
        let _ = clear_all().await;
    }

    #[tokio::test]
    async fn test_data_integrity_after_operations() {
        let _ = setup_and_cleanup_repository().await;
        
        // Insert a product with specific data
        let original_product = Produk::new(
            "Test Product".to_string(),
            "Test Category".to_string(),
            1_234_567.89,
            42,
            Some("Test description with special chars: éñ@#$%".to_string())
        );
        
        let id = tambah_produk(&original_product).await.unwrap();
        
        // Retrieve and verify data integrity
        let retrieved = ambil_produk_by_id(id).await.unwrap().unwrap();
        
        // Test exact matches for all fields
        assert_eq!(retrieved.nama, original_product.nama);
        assert_eq!(retrieved.kategori, original_product.kategori);
        
        // For floating point comparison, allow small epsilon
        let price_diff = (retrieved.harga - original_product.harga).abs();
        assert!(price_diff < 0.01, "Price difference too large: {}", price_diff);
        
        assert_eq!(retrieved.stok, original_product.stok);
        assert_eq!(retrieved.deskripsi, original_product.deskripsi);
        
        // Verify timestamp fields are populated (PostgreSQL adds these)
        // Note: We can't test exact values since they're set by database
        
        // Cleanup
        let _ = clear_all().await;
    }
}