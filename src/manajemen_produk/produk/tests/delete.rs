use super::super::model::Produk;

#[test]
fn test_remove_produk_from_list() {
    let produk1 = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    
    let produk2 = Produk::new(
        "Cat Tembok".to_string(),
        "Material".to_string(),
        150_000.0,
        50,
        Some("Cat tembok anti air".to_string()),
    );
    
    let produk3 = Produk::new(
        "Smartphone".to_string(),
        "Elektronik".to_string(),
        8_000_000.0,
        20,
        Some("Smartphone dengan kamera 108MP".to_string()),
    );
    
    let mut produk_list = vec![produk1, produk2, produk3];
    
    // Remove produk dari list (misalnya berdasarkan nama)
    produk_list.retain(|p| p.nama != "Cat Tembok");
    
    assert_eq!(produk_list.len(), 2);
    assert_eq!(produk_list[0].nama, "Laptop Gaming");
    assert_eq!(produk_list[1].nama, "Smartphone");
}