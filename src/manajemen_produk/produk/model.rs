// File: src/manajemen_produk/produk/model.rs
use std::sync::{Arc, Mutex, Once};
use std::collections::HashMap;
use lazy_static::lazy_static;

#[derive(Debug, Clone, PartialEq)]
pub struct Produk {
    pub id: Option<i64>,  // Menambahkan ID untuk memudahkan operasi database
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: u32,
    pub deskripsi: Option<String>,
}

// Builder pattern implementation - sudah baik, sedikit perbaikan
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
    
    pub fn build(self) -> Result<Produk, &'static str> {
        // Validate before building
        validate_produk(&self.nama, &self.kategori, self.harga, self.stok, &self.deskripsi)?;
        
        Ok(Produk {
            id: self.id,
            nama: self.nama,
            kategori: self.kategori,
            harga: self.harga,
            stok: self.stok,
            deskripsi: self.deskripsi,
        })
    }
}

// Prototype Pattern - Clone dengan parameter khusus
impl Produk {
    // Regular constructor kept for backward compatibility
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
    
    // Menambahkan constructor dengan ID - untuk penggunaan dari database
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
    
    pub fn validate(&self) -> Result<(), &'static str> {
        validate_produk(&self.nama, &self.kategori, self.harga, self.stok, &self.deskripsi)
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
    
    // Factory method pattern - sudah baik
    pub fn create_laptop(nama: String, harga: f64, stok: u32, deskripsi: Option<String>) -> Self {
        Self::new(nama, "Elektronik".to_string(), harga, stok, deskripsi)
    }
    
    pub fn create_building_material(nama: String, harga: f64, stok: u32, deskripsi: Option<String>) -> Self {
        Self::new(nama, "Material".to_string(), harga, stok, deskripsi)
    }
    
    pub fn create_furniture(nama: String, harga: f64, stok: u32, deskripsi: Option<String>) -> Self {
        Self::new(nama, "Furnitur".to_string(), harga, stok, deskripsi)
    }
    
    pub fn create_plumbing(nama: String, harga: f64, stok: u32, deskripsi: Option<String>) -> Self {
        Self::new(nama, "Pipa & Sanitasi".to_string(), harga, stok, deskripsi)
    }
    
    // Prototype Pattern - Clone dengan parameter khusus
    pub fn clone_with_new_price(&self, new_price: f64) -> Result<Self, &'static str> {
        if new_price < 0.0 {
            return Err("Harga produk tidak boleh negatif");
        }
        
        Ok(Self {
            id: self.id,
            nama: self.nama.clone(),
            kategori: self.kategori.clone(),
            harga: new_price,
            stok: self.stok,
            deskripsi: self.deskripsi.clone(),
        })
    }
    
    pub fn clone_with_new_stock(&self, new_stock: u32) -> Self {
        Self {
            id: self.id,
            nama: self.nama.clone(),
            kategori: self.kategori.clone(),
            harga: self.harga,
            stok: new_stock,
            deskripsi: self.deskripsi.clone(),
        }
    }
}

// Abstract Factory Pattern
pub trait ProdukFactory {
    fn create_produk(&self, nama: String, harga: f64, stok: u32, deskripsi: Option<String>) -> Produk;
}

pub struct ElektronikFactory;
impl ProdukFactory for ElektronikFactory {
    fn create_produk(&self, nama: String, harga: f64, stok: u32, deskripsi: Option<String>) -> Produk {
        Produk::create_laptop(nama, harga, stok, deskripsi)
    }
}

pub struct MaterialFactory;
impl ProdukFactory for MaterialFactory {
    fn create_produk(&self, nama: String, harga: f64, stok: u32, deskripsi: Option<String>) -> Produk {
        Produk::create_building_material(nama, harga, stok, deskripsi)
    }
}

pub struct FurnitureFactory;
impl ProdukFactory for FurnitureFactory {
    fn create_produk(&self, nama: String, harga: f64, stok: u32, deskripsi: Option<String>) -> Produk {
        Produk::create_furniture(nama, harga, stok, deskripsi)
    }
}

pub struct PlumbingFactory;
impl ProdukFactory for PlumbingFactory {
    fn create_produk(&self, nama: String, harga: f64, stok: u32, deskripsi: Option<String>) -> Produk {
        Produk::create_plumbing(nama, harga, stok, deskripsi)
    }
}

// Singleton Pattern untuk Registry Factory
pub struct ProdukFactoryRegistry {
    factories: HashMap<String, Box<dyn ProdukFactory + Send + Sync>>,
}

impl ProdukFactoryRegistry {
    fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };
        
        // Register default factories
        registry.register("Elektronik".to_string(), Box::new(ElektronikFactory));
        registry.register("Material".to_string(), Box::new(MaterialFactory));
        registry.register("Furnitur".to_string(), Box::new(FurnitureFactory));
        registry.register("Pipa & Sanitasi".to_string(), Box::new(PlumbingFactory));
        
        registry
    }
    
    pub fn register(&mut self, kategori: String, factory: Box<dyn ProdukFactory + Send + Sync>) {
        self.factories.insert(kategori, factory);
    }
    
    pub fn get_factory(&self, kategori: &str) -> Option<&(dyn ProdukFactory + Send + Sync)> {
        self.factories.get(kategori).map(|f| f.as_ref())
    }
    
    pub fn create_produk(&self, kategori: &str, nama: String, harga: f64, stok: u32, deskripsi: Option<String>) -> Option<Produk> {
        self.get_factory(kategori).map(|factory| {
            factory.create_produk(nama, harga, stok, deskripsi)
        })
    }
}

// Implementasi singleton untuk registry factory
lazy_static! {
    static ref REGISTRY: Arc<Mutex<ProdukFactoryRegistry>> = Arc::new(Mutex::new(ProdukFactoryRegistry::new()));
}

pub fn get_produk_factory_registry() -> Arc<Mutex<ProdukFactoryRegistry>> {
    REGISTRY.clone()
}

// Object Pool Pattern untuk Produk Template
pub struct ProdukTemplatePool {
    templates: HashMap<String, Produk>,
}

impl ProdukTemplatePool {
    fn new() -> Self {
        let mut pool = Self {
            templates: HashMap::new(),
        };
        
        // Add some default templates
        pool.add_template(
            "laptop_gaming".to_string(),
            Produk::new(
                "Laptop Gaming Template".to_string(),
                "Elektronik".to_string(),
                0.0,  // Placeholder price
                0,    // Placeholder stock
                Some("Laptop gaming dengan performa tinggi".to_string()),
            )
        );
        
        pool.add_template(
            "smartphone".to_string(),
            Produk::new(
                "Smartphone Template".to_string(),
                "Elektronik".to_string(),
                0.0,
                0,
                Some("Smartphone dengan kamera berkualitas".to_string()),
            )
        );
        
        pool.add_template(
            "cat_tembok".to_string(),
            Produk::new(
                "Cat Tembok Template".to_string(),
                "Material".to_string(),
                0.0,
                0,
                Some("Cat tembok dengan kualitas terbaik".to_string()),
            )
        );
        
        pool
    }
    
    pub fn add_template(&mut self, key: String, template: Produk) {
        self.templates.insert(key, template);
    }
    
    pub fn get_template(&self, key: &str) -> Option<&Produk> {
        self.templates.get(key)
    }
    
    pub fn create_from_template(&self, key: &str, harga: f64, stok: u32) -> Option<Produk> {
        self.get_template(key).map(|template| {
            Produk {
                id: None,
                nama: template.nama.clone(),
                kategori: template.kategori.clone(),
                harga,
                stok,
                deskripsi: template.deskripsi.clone(),
            }
        })
    }
}

// Implementasi singleton untuk template pool
lazy_static! {
    static ref TEMPLATE_POOL: Arc<Mutex<ProdukTemplatePool>> = Arc::new(Mutex::new(ProdukTemplatePool::new()));
}

pub fn get_produk_template_pool() -> Arc<Mutex<ProdukTemplatePool>> {
    TEMPLATE_POOL.clone()
}

// Function untuk validasi produk - ekstrak ke function supaya bisa digunakan di berbagai tempat
pub fn validate_produk(
    nama: &str,
    kategori: &str,
    harga: f64,
    _stok: u32,
    _deskripsi: &Option<String>,
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