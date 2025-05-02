#[test]
fn test_nama_not_empty_strategy() {
    let strategy = NamaNotEmpty;
    let mut produk = Produk::new("".to_string(), "Elektronik".to_string(), 1000.0, 10, None);
    assert!(strategy.validate(&produk).is_err());

    produk.nama = "Laptop".to_string();
    assert!(strategy.validate(&produk).is_ok());
}

#[test]
fn test_harga_non_negatif_strategy() {
    let strategy = HargaNonNegatif;
    let mut produk = Produk::new("Laptop".to_string(), "Elektronik".to_string(), -1000.0, 10, None);
    assert!(strategy.validate(&produk).is_err());

    produk.harga = 1000.0;
    assert!(strategy.validate(&produk).is_ok());
}