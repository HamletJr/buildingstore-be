use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::validation::{
    ValidationRule, 
    NamaNotEmpty, 
    HargaNonNegatif,
    KategoriNotEmpty,
    StokNonNegatif,
    DeskripsiMaxLength
};

// Test each validation rule individually
#[test]
fn test_nama_not_empty() {
    let strategy = NamaNotEmpty;
    
    // Test invalid case
    let invalid_produk = Produk::new("".into(), "Elektronik".into(), 1000.0, 10, None);
    let result = strategy.validate(&invalid_produk);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Nama produk tidak boleh kosong");
    
    // Test valid case
    let valid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, None);
    assert!(strategy.validate(&valid_produk).is_ok());
}

#[test]
fn test_kategori_not_empty() {
    let strategy = KategoriNotEmpty;
    
    // Test invalid case
    let invalid_produk = Produk::new("Laptop".into(), "".into(), 1000.0, 10, None);
    let result = strategy.validate(&invalid_produk);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Kategori produk tidak boleh kosong");
    
    // Test valid case
    let valid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, None);
    assert!(strategy.validate(&valid_produk).is_ok());
}

#[test]
fn test_harga_non_negatif() {
    let strategy = HargaNonNegatif;
    
    // Test invalid case
    let invalid_produk = Produk::new("Laptop".into(), "Elektronik".into(), -1000.0, 10, None);
    let result = strategy.validate(&invalid_produk);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Harga tidak boleh negatif");
    
    // Test valid case
    let valid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, None);
    assert!(strategy.validate(&valid_produk).is_ok());
    
    // Test edge case (zero price is valid)
    let zero_price_produk = Produk::new("Laptop".into(), "Elektronik".into(), 0.0, 10, None);
    assert!(strategy.validate(&zero_price_produk).is_ok());
}

#[test]
fn test_stok_non_negatif() {
    let strategy = StokNonNegatif;
    
    // Create a product with negative stock (this shouldn't be possible with u32, 
    // but we can test the validation logic)
    let valid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 0, None);
    assert!(strategy.validate(&valid_produk).is_ok());
}

#[test]
fn test_deskripsi_max_length() {
    let strategy = DeskripsiMaxLength;
    
    // Create a product with description longer than maximum
    let long_description = "a".repeat(501); // Assuming max length is 500
    let invalid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, Some(long_description));
    let result = strategy.validate(&invalid_produk);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Deskripsi terlalu panjang (maksimal 500 karakter)");
    
    // Test valid case
    let valid_description = "a".repeat(500);
    let valid_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, Some(valid_description));
    assert!(strategy.validate(&valid_produk).is_ok());
    
    // Test edge case (empty description is valid)
    let empty_desc_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, Some("".into()));
    assert!(strategy.validate(&empty_desc_produk).is_ok());
    
    // Test None description
    let none_desc_produk = Produk::new("Laptop".into(), "Elektronik".into(), 1000.0, 10, None);
    assert!(strategy.validate(&none_desc_produk).is_ok());
}

// Integration tests for all validation rules
#[test]
fn test_produk_validation_all_rules() {
    // Create a valid product
    let valid_produk = Produk::new(
        "Laptop Gaming".into(),
        "Elektronik".into(),
        15_000_000.0,
        10,
        Some("Laptop dengan spesifikasi tinggi".into())
    );
    
    // Test all validation rules
    assert!(valid_produk.validate().is_ok());
    
    // Create an invalid product with multiple validation failures
    let invalid_produk = Produk::new(
        "".into(),
        "".into(),
        -1000.0,
        10,
        Some("a".repeat(501))
    );
    
    // Test all validation rules
    let validation_result = invalid_produk.validate();
    assert!(validation_result.is_err());
    
    // Check for all expected error messages
    let error_messages = validation_result.unwrap_err();
    assert!(error_messages.contains(&"Nama produk tidak boleh kosong".to_string()));
    assert!(error_messages.contains(&"Kategori produk tidak boleh kosong".to_string()));
    assert!(error_messages.contains(&"Harga tidak boleh negatif".to_string()));
    assert!(error_messages.contains(&"Deskripsi terlalu panjang (maksimal 500 karakter)".to_string()));
}

// Test edge cases
#[test]
fn test_validation_edge_cases() {
    // Edge case: product with minimum valid values
    let min_produk = Produk::new(
        "A".into(),  // Minimum name length
        "B".into(),  // Minimum category length
        0.0,         // Minimum price
        0,           // Minimum stock
        None,        // No description
    );
    assert!(min_produk.validate().is_ok());
    
    // Edge case: product with very large values
    let max_produk = Produk::new(
        "Very Long Product Name".into(),
        "Very Long Category Name".into(),
        f64::MAX,     // Maximum float value
        u32::MAX,     // Maximum u32 value
        Some("a".repeat(500)),  // Maximum description length
    );
    assert!(max_produk.validate().is_ok());
}