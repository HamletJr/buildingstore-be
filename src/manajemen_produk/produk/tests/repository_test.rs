use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::test;

use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::repository::{ProdukRepository, RepositoryError};

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
    ProdukRepository::clear_all().await
}

// Test repository creation and retrieval
#[test]
async fn test_tambah_dan_ambil_produk() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Create test product
    let produk = create_test_products()[0].clone();
    
    // Add product to repository
    let result = ProdukRepository::tambah_produk(&produk).await;
    assert!(result.is_ok());
    
    let id = result.unwrap();
    assert!(id > 0); // ID should be positive
    
    // Retrieve product
    let retrieved = ProdukRepository::ambil_produk_by_id(id).await.unwrap();
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
async fn test_ambil_semua_produk() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Add multiple products
    let mut test_products = create_test_products();
    
    for product in &test_products {
        let _ = ProdukRepository::tambah_produk(product).await.unwrap();
    }
    
    // Retrieve all products
    let mut all_products = ProdukRepository::ambil_semua_produk().await.unwrap();
    
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

// Test retrieving non-existent product
#[test]
async fn test_ambil_produk_tidak_ada() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Try to retrieve non-existent product
    let retrieved = ProdukRepository::ambil_produk_by_id(9999).await.unwrap();
    assert!(retrieved.is_none());
}

// Test filtering by category
#[test]
async fn test_filter_by_kategori() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Add test products
    let test_products = create_test_products();
    for product in &test_products {
        let _ = ProdukRepository::tambah_produk(product).await.unwrap();
    }
    
    // Filter by "Elektronik" category
    let filtered = ProdukRepository::filter_produk_by_kategori("Elektronik").await.unwrap();
    
    // Should have 2 electronic products
    assert_eq!(filtered.len(), 2);
    
    // Verify all products are in the "Elektronik" category
    for product in filtered {
        assert_eq!(product.kategori, "Elektronik");
    }
    
    // Filter by non-existent category
    let non_existent = ProdukRepository::filter_produk_by_kategori("NonExistent").await.unwrap();
    assert!(non_existent.is_empty());
}

// Test updating a product
#[test]
async fn test_update_produk() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Add a product
    let mut produk = create_test_products()[0].clone();
    let id = ProdukRepository::tambah_produk(&produk).await.unwrap();
    
    // Update the product
    produk.nama = "Updated Laptop".to_string();
    produk.harga = 16_000_000.0;
    produk.stok = 15;
    produk.deskripsi = Some("Updated description".to_string());
    
    let update_result = ProdukRepository::update_produk(id, &produk).await.unwrap();
    assert!(update_result);
    
    // Retrieve and verify update
    let updated = ProdukRepository::ambil_produk_by_id(id).await.unwrap().unwrap();
    
    assert_eq!(updated.nama, "Updated Laptop");
    assert_eq!(updated.harga, 16_000_000.0);
    assert_eq!(updated.stok, 15);
    assert_eq!(updated.deskripsi, Some("Updated description".to_string()));
}

// Test updating non-existent product
#[test]
async fn test_update_produk_tidak_ada() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Try to update non-existent product
    let produk = create_test_products()[0].clone();
    let update_result = ProdukRepository::update_produk(9999, &produk).await.unwrap();
    
    // Should return false but not error
    assert!(!update_result);
}

// Test deleting a product
#[test]
async fn test_hapus_produk() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Add a product
    let produk = create_test_products()[0].clone();
    let id = ProdukRepository::tambah_produk(&produk).await.unwrap();
    
    // Delete the product
    let delete_result = ProdukRepository::hapus_produk(id).await.unwrap();
    assert!(delete_result);
    
    // Verify product is deleted
    let retrieved = ProdukRepository::ambil_produk_by_id(id).await.unwrap();
    assert!(retrieved.is_none());
}

// Test deleting non-existent product
#[test]
async fn test_hapus_produk_tidak_ada() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Try to delete non-existent product
    let delete_result = ProdukRepository::hapus_produk(9999).await.unwrap();
    
    // Should return false but not error
    assert!(!delete_result);
}

// Test filtering by price range
#[test]
async fn test_filter_by_price_range() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Add test products
    let test_products = create_test_products();
    for product in &test_products {
        let _ = ProdukRepository::tambah_produk(product).await.unwrap();
    }
    
    // Filter by price range
    let filtered = ProdukRepository::filter_produk_by_price_range(1_000_000.0, 10_000_000.0).await.unwrap();
    
    // Should have 1 product in this range
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].nama, "Smartphone");
    
    // Test empty range
    let empty_range = ProdukRepository::filter_produk_by_price_range(50_000_000.0, 100_000_000.0).await.unwrap();
    assert!(empty_range.is_empty());
    
    // Test edge cases (exact match)
    let exact_match = ProdukRepository::filter_produk_by_price_range(8_000_000.0, 8_000_000.0).await.unwrap();
    assert_eq!(exact_match.len(), 1);
    assert_eq!(exact_match[0].nama, "Smartphone");
}

// Test filtering by stock availability
#[test]
async fn test_filter_by_stock_availability() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Add test products
    let test_products = create_test_products();
    for product in &test_products {
        let _ = ProdukRepository::tambah_produk(product).await.unwrap();
    }
    
    // Filter by minimum stock
    let filtered = ProdukRepository::filter_produk_by_stock_availability(20).await.unwrap();
    
    // Should have 2 products with stock >= 20
    assert_eq!(filtered.len(), 2);
    
    // Verify stock values
    for product in filtered {
        assert!(product.stok >= 20);
    }
    
    // Test high stock threshold
    let high_stock = ProdukRepository::filter_produk_by_stock_availability(100).await.unwrap();
    assert!(high_stock.is_empty());
}

// Test clearing all products
#[test]
async fn test_clear_all() {
    // Start with clean repository
    let _ = cleanup_repository().await;
    
    // Add test products
    let test_products = create_test_products();
    for product in &test_products {
        let _ = ProdukRepository::tambah_produk(product).await.unwrap();
    }
    
    // Verify products exist
    let before_clear = ProdukRepository::ambil_semua_produk().await.unwrap();
    assert_eq!(before_clear.len(), test_products.len());
    
    // Clear all products
    let clear_result = ProdukRepository::clear_all().await;
    assert!(clear_result.is_ok());
    
    // Verify all products are gone
    let after_clear = ProdukRepository::ambil_semua_produk().await.unwrap();
    assert!(after_clear.is_empty());
}

// Test concurrent operations
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
            
            ProdukRepository::tambah_produk(&produk).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
    
    // Verify all products were added
    let all_products = ProdukRepository::ambil_semua_produk().await.unwrap();
    assert_eq!(all_products.len(), 5);
}

// Test ID sequence
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
        
        let id = ProdukRepository::tambah_produk(&produk).await.unwrap();
        ids.push(id);
    }
    
    // Verify IDs are sequential
    assert_eq!(ids.len(), 3);
    assert_eq!(ids[0] + 1, ids[1]);
    assert_eq!(ids[1] + 1, ids[2]);
    
    // Delete a product
    let _ = ProdukRepository::hapus_produk(ids[1]).await;
    
    // Add another product
    let produk = Produk::new(
        "New Product".to_string(),
        "Test".to_string(),
        1000.0,
        10,
        None,
    );
    
    let new_id = ProdukRepository::tambah_produk(&produk).await.unwrap();
    
    // Verify the new ID is greater than the last used ID
    assert!(new_id > ids[2]);
}