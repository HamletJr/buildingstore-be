#[derive(Debug, Clone, PartialEq)]
pub struct Produk {
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: u32,
    pub deskripsi: Option<String>,
}

impl Produk {
    pub fn new(
        nama: String,
        kategori: String,
        harga: f64,
        stok: u32,
        deskripsi: Option<String>,
    ) -> Self {
        Self {
            nama,
            kategori,
            harga,
            stok,
            deskripsi,
        }
    }
    
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.nama.trim().is_empty() {
            return Err("Nama produk tidak boleh kosong");
        }
        
        if self.kategori.trim().is_empty() {
            return Err("Kategori produk tidak boleh kosong");
        }
        
        if self.harga < 0.0 {
            return Err("Harga produk tidak boleh negatif");
        }
        
        Ok(())
    }
    
    pub fn create_with_validation(
        nama: String,
        kategori: String,
        harga: f64,
        stok: u32,
        deskripsi: Option<String>,
    ) -> Result<Self, &'static str> {
        let produk = Self::new(nama, kategori, harga, stok, deskripsi);
        produk.validate()?;
        Ok(produk)
    }
}

pub fn validate_produk(
    nama: String,
    kategori: String,
    harga: f64,
    _stok: u32,
    _deskripsi: Option<String>,
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