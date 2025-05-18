#[derive(Debug, Clone)]
pub struct Produk {
    pub id: Option<i64>,
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: u32,
    pub deskripsi: Option<String>,
}

// Builder pattern implementation
pub struct ProdukBuilder {
    id: Option<i64>,
    nama: String,
    kategori: String,
    harga: f64,
    stok: u32,
    deskripsi: Option<String>,
}

impl ProdukBuilder {
    pub fn new(nama: String, kategori: String) -> Self {
        Self {
            id: None,
            nama,
            kategori,
            harga: 0.0,
            stok: 0,
            deskripsi: None,
        }
    }
    
    pub fn id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }
    
    pub fn harga(mut self, harga: f64) -> Self {
        self.harga = harga;
        self
    }
    
    pub fn stok(mut self, stok: u32) -> Self {
        self.stok = stok;
        self
    }
    
    pub fn deskripsi(mut self, deskripsi: String) -> Self {
        self.deskripsi = Some(deskripsi);
        self
    }
    
    pub fn build(self) -> Result<Produk, Vec<String>> {
        let produk = Produk {
            id: self.id,
            nama: self.nama,
            kategori: self.kategori,
            harga: self.harga,
            stok: self.stok,
            deskripsi: self.deskripsi,
        };
        
        // Validate the product before returning it
        match produk.validate() {
            Ok(_) => Ok(produk),
            Err(errors) => Err(errors),
        }
    }
}

impl Produk {
    // Constructor with ID - for database retrieval
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
    
    // Regular constructor
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
    
    // Comprehensive validation method using the ProdukValidator
    pub fn validate(&self) -> Result<(), Vec<String>> {
        use crate::manajemen_produk::produk::validation::ProdukValidator;
        
        let validator = ProdukValidator::default();
        validator.validate(self)
    }
}