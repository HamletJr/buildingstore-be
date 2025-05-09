use super::super::model::Produk;

#[test]
fn test_update_produk_harga() {
    let mut produk = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    
    // Update harga
    produk.set_harga(14_500_000.0).unwrap();
    
    assert_eq!(produk.harga, 14_500_000.0);
}

#[test]
fn test_update_produk_stok() {
    let mut produk = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    
    // Update stok
    produk.set_stok(5);
    
    assert_eq!(produk.stok, 5);
}

#[test]
fn test_update_produk_deskripsi() {
    let mut produk = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    
    // Update deskripsi
    produk.deskripsi = Some("Laptop Gaming dengan RTX 4060 dan RAM 32GB".to_string());
    
    assert_eq!(produk.deskripsi, Some("Laptop Gaming dengan RTX 4060 dan RAM 32GB".to_string()));
}