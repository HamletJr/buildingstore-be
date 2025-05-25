// Struct yang merepresentasikan entitas produk dalam sistem.

// # Fields
// - `id`: ID unik produk (opsional, None untuk produk baru)
// - `nama`: Nama produk (wajib)
// - `kategori`: Kategori produk (wajib)
// - `harga`: Harga produk dalam bentuk float (wajib)
// - `stok`: Jumlah stok tersedia (wajib)
// - `deskripsi`: Deskripsi tambahan produk (opsional)

// # Methods
// - `with_id()`: Constructor untuk produk yang sudah ada di database
// - `new()`: Constructor untuk produk baru
// - `validate()`: Validasi data produk sebelum disimpan

#[derive(Debug, Clone)]
pub struct Produk {
    pub id: Option<i64>,
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: u32,
    pub deskripsi: Option<String>,
}

impl Produk {
    pub fn with_id(
        id: i64,
        nama: String,
        kategori: String,
        harga: f64,
        stok: u32,
        deskripsi: Option<String>,
    ) -> Self {
        Self {
            id: Some(id),
            nama,
            kategori,
            harga,
            stok,
            deskripsi,
        }
    }
    
    pub fn new(
        nama: String,
        kategori: String,
        harga: f64,
        stok: u32,
        deskripsi: Option<String>,
    ) -> Self {
        Self {
            id: None,
            nama,
            kategori,
            harga,
            stok,
            deskripsi,
        }
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        use crate::manajemen_produk::produk::validation::ProdukValidator;
        
        let validator = ProdukValidator::default();
        validator.validate(self)
    }
}

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

// CREATE tests
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



// READ tests
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

#[test]
fn test_update_produk_harga() {
    let mut produk = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    
    produk.harga = 14_500_000.0;
    
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
    
    produk.stok = 5;
    
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
    
    produk.deskripsi = Some("Laptop Gaming dengan RTX 4060 dan RAM 32GB".to_string());
    
    assert_eq!(produk.deskripsi, Some("Laptop Gaming dengan RTX 4060 dan RAM 32GB".to_string()));
}

// DELETE tests
#[test]
fn test_remove_produk_from_list() {
    let mut produk_list = setup_test_products();
    
    // Remove produk dari list (berdasarkan nama)
    produk_list.retain(|p| p.nama != "Cat Tembok");
    
    assert_eq!(produk_list.len(), 2);
    assert_eq!(produk_list[0].nama, "Laptop Gaming");
    assert_eq!(produk_list[1].nama, "Smartphone");
}

// Validation tests
#[test]
fn test_validasi_produk() {
    // Testing valid product
    let produk = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    assert!(produk.validate().is_ok());
    
    // Testing invalid product (empty name)
    let produk_invalid = Produk::new(
        "".to_string(),
        "Elektronik".to_string(),
        15_000_000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    assert!(produk_invalid.validate().is_err());
    assert_eq!(produk_invalid.validate().unwrap_err(), vec!["Nama produk tidak boleh kosong"]);
    
    // Testing invalid product (negative price)
    let produk_invalid_price = Produk::new(
        "Laptop Gaming".to_string(),
        "Elektronik".to_string(),
        -5000.0,
        10,
        Some("Laptop dengan RTX 4060".to_string()),
    );
    assert!(produk_invalid_price.validate().is_err());
    assert_eq!(produk_invalid_price.validate().unwrap_err(), vec!["Harga tidak boleh negatif"]);
}