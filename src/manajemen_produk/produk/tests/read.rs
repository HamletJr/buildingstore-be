use super::super::model::Produk;

fn setup_test_products() -> Vec<Produk> {
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

#[test]
fn test_filter_produk_by_kategori() {
    let produk_list = setup_test_products();
    
    // Filter produk elektronik
    let elektronik_list: Vec<&Produk> = produk_list.iter()
        .filter(|p| p.kategori == "Elektronik")
        .collect();
    
    assert_eq!(elektronik_list.len(), 2);
    assert_eq!(elektronik_list[0].nama, "Laptop Gaming");
    assert_eq!(elektronik_list[1].nama, "Smartphone");
}

#[test]
fn test_sort_produk_by_harga() {
    let mut produk_list = setup_test_products();
    
    // Sort by harga (ascending)
    produk_list.sort_by(|a, b| a.harga.partial_cmp(&b.harga).unwrap());
    
    assert_eq!(produk_list[0].nama, "Cat Tembok");
    assert_eq!(produk_list[1].nama, "Smartphone");
    assert_eq!(produk_list[2].nama, "Laptop Gaming");
}

#[test]
fn test_find_produk_by_nama() {
    let produk_list = setup_test_products();
    
    let found_produk = produk_list.iter()
        .find(|p| p.nama == "Smartphone");
    
    assert!(found_produk.is_some());
    let produk = found_produk.unwrap();
    assert_eq!(produk.kategori, "Elektronik");
    assert_eq!(produk.harga, 8_000_000.0);
}