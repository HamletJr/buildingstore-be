use super::helper::{lock_store, RepositoryError};
use crate::manajemen_produk::produk::model::Produk;

pub async fn ambil_semua_produk() -> Result<Vec<Produk>, RepositoryError> {
    let store = lock_store()?;
    Ok(store.values().cloned().collect())
}

pub async fn ambil_produk_by_id(id: i64) -> Result<Option<Produk>, RepositoryError> {
    let store = lock_store()?;
    Ok(store.get(&id).cloned())
}

pub async fn filter_produk_by_kategori(kategori: &str) -> Result<Vec<Produk>, RepositoryError> {
    let store = lock_store()?;
    Ok(store.values()
        .filter(|p| p.kategori == kategori)
        .cloned()
        .collect())
}

pub async fn filter_produk_by_price_range(min_price: f64, max_price: f64) -> Result<Vec<Produk>, RepositoryError> {
    let store = lock_store()?;
    Ok(store.values()
        .filter(|p| p.harga >= min_price && p.harga <= max_price)
        .cloned()
        .collect())
}

pub async fn filter_produk_by_stock_availability(min_stock: u32) -> Result<Vec<Produk>, RepositoryError> {
    let store = lock_store()?;
    Ok(store.values()
        .filter(|p| p.stok >= min_stock)
        .cloned()
        .collect())
}

pub async fn search_produk_by_name(name_query: &str) -> Result<Vec<Produk>, RepositoryError> {
    let store = lock_store()?;
    let query_lower = name_query.to_lowercase();
    Ok(store.values()
        .filter(|p| p.nama.to_lowercase().contains(&query_lower))
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::produk::repository::create::tambah_produk;
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

    #[test]
    async fn test_ambil_semua_produk() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add multiple products
        let mut test_products = create_test_products();
        
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        // Retrieve all products
        let mut all_products = ambil_semua_produk().await.unwrap();
        
        // Verify count
        assert_eq!(all_products.len(), test_products.len());
        
        // Sort both lists by name for consistent comparison
        all_products.sort_by(|a, b| a.nama.cmp(&b.nama));
        test_products.sort_by(|a, b| a.nama.cmp(&b.nama));
        
        // Verify products match (ignoring IDs which are assigned)
        for (i, product) in all_products.iter().enumerate() {
            assert_eq!(product.nama, test_products[i].nama);
            assert_eq!(product.kategori, test_products[i].kategori);
            assert_eq!(product.harga, test_products[i].harga);
            assert_eq!(product.stok, test_products[i].stok);
            assert_eq!(product.deskripsi, test_products[i].deskripsi);
        }
    }

    #[test]
    async fn test_ambil_produk_by_id() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add a product
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        // Retrieve product by ID
        let retrieved = ambil_produk_by_id(id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_produk = retrieved.unwrap();
        assert_eq!(retrieved_produk.id.unwrap(), id);
        assert_eq!(retrieved_produk.nama, produk.nama);
    }

    #[test]
    async fn test_ambil_produk_tidak_ada() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Try to retrieve non-existent product
        let retrieved = ambil_produk_by_id(9999).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    async fn test_filter_by_kategori() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add test products
        let test_products = create_test_products();
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        // Filter by "Elektronik" category
        let filtered = filter_produk_by_kategori("Elektronik").await.unwrap();
        
        // Should have 2 electronic products
        assert_eq!(filtered.len(), 2);
        
        // Verify all products are in the "Elektronik" category
        for product in filtered {
            assert_eq!(product.kategori, "Elektronik");
        }
        
        // Filter by non-existent category
        let non_existent = filter_produk_by_kategori("NonExistent").await.unwrap();
        assert!(non_existent.is_empty());
    }

    #[test]
    async fn test_filter_by_price_range() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add test products
        let test_products = create_test_products();
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        // Filter by price range
        let filtered = filter_produk_by_price_range(1_000_000.0, 10_000_000.0).await.unwrap();
        
        // Should have 1 product in this range
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].nama, "Smartphone");
        
        // Test empty range
        let empty_range = filter_produk_by_price_range(50_000_000.0, 100_000_000.0).await.unwrap();
        assert!(empty_range.is_empty());
        
        // Test edge cases (exact match)
        let exact_match = filter_produk_by_price_range(8_000_000.0, 8_000_000.0).await.unwrap();
        assert_eq!(exact_match.len(), 1);
        assert_eq!(exact_match[0].nama, "Smartphone");
    }

    #[test]
    async fn test_filter_by_stock_availability() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add test products
        let test_products = create_test_products();
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        // Filter by minimum stock
        let filtered = filter_produk_by_stock_availability(20).await.unwrap();
        
        // Should have 2 products with stock >= 20
        assert_eq!(filtered.len(), 2);
        
        // Verify stock values
        for product in filtered {
            assert!(product.stok >= 20);
        }
        
        // Test high stock threshold
        let high_stock = filter_produk_by_stock_availability(100).await.unwrap();
        assert!(high_stock.is_empty());
    }

    #[test]
    async fn test_search_produk_by_name() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Add test products
        let test_products = create_test_products();
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        // Search by partial name
        let laptop_results = search_produk_by_name("Laptop").await.unwrap();
        assert_eq!(laptop_results.len(), 1);
        assert_eq!(laptop_results[0].nama, "Laptop Gaming");
        
        // Search case insensitive
        let case_insensitive = search_produk_by_name("gaming").await.unwrap();
        assert_eq!(case_insensitive.len(), 1);
        assert_eq!(case_insensitive[0].nama, "Laptop Gaming");
        
        // Search with no results
        let no_results = search_produk_by_name("NonExistent").await.unwrap();
        assert!(no_results.is_empty());
        
        // Search with multiple results
        let multiple_results = search_produk_by_name("e").await.unwrap(); // Should match multiple products
        assert!(multiple_results.len() > 1);
    }

    #[test]
    async fn test_empty_repository() {
        // Start with clean repository
        let _ = cleanup_repository().await;
        
        // Test all read operations on empty repository
        let all_products = ambil_semua_produk().await.unwrap();
        assert!(all_products.is_empty());
        
        let filtered_kategori = filter_produk_by_kategori("Test").await.unwrap();
        assert!(filtered_kategori.is_empty());
        
        let filtered_price = filter_produk_by_price_range(0.0, 1000.0).await.unwrap();
        assert!(filtered_price.is_empty());
        
        let filtered_stock = filter_produk_by_stock_availability(1).await.unwrap();
        assert!(filtered_stock.is_empty());
        
        let search_results = search_produk_by_name("test").await.unwrap();
        assert!(search_results.is_empty());
    }
}