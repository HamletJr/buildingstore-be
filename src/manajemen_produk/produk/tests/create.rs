// File: src/manajemen_produk/produk/tests/create.rs
use super::super::model::Produk;

#[test]
fn test_create_produk_baru() {
    let produk = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );

    assert_eq!(produk.nama, "Laptop Gaming");
    assert_eq!(produk.kategori, "Elektronik");
    assert_eq!(produk.harga, 15_000_000.0);
    assert_eq!(produk.stok, 10);
    assert_eq!(produk.deskripsi, Some("Laptop dengan RTX 4060".to_string()));
}

#[test]
fn test_create_produk_without_deskripsi() {
    let produk = Produk::new(
        "Cat Tembok".to_string(),
        "Material".to_string(),
        150_000.0,
        50,
        None,
    );

    assert_eq!(produk.nama, "Cat Tembok");
    assert_eq!(produk.kategori, "Material");
    assert_eq!(produk.harga, 150_000.0);
    assert_eq!(produk.stok, 50);
    assert_eq!(produk.deskripsi, None);
}

#[test]
fn test_validasi_produk() {
    // Testing valid product
    let result = validate_produk(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    assert!(result.is_ok());
    
    // Testing invalid product (empty name)
    let result = validate_produk(
        "".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Nama produk tidak boleh kosong");
    
    // Testing invalid product (negative price)
    let result = validate_produk(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        -5000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Harga produk tidak boleh negatif");
}

fn validate_produk(
    nama: String,
    kategori: String,
    harga: f64,
    stok: u32,
    deskripsi: Option<String>,
) -> Result<(), &'static str> {
    if nama.trim().is_empty() {
        return Err("Nama produk tidak boleh kosong");
    }
    
    if kategori.trim().is_empty() {
        return Err("Kategori produk tidak boleh kosong");
    }
    
    if harga < 0.0 {
        return Err("Harga produk tidak boleh negatif");
    }
    
    Ok(())
}