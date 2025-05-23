use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::error::Error as StdError;
use std::fmt;

use super::model::Produk;

// In-memory storage using HashMap
lazy_static! {
    static ref PRODUCT_STORE: Arc<Mutex<HashMap<i64, Produk>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref ID_COUNTER: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));
}

// Helper function to get next ID
fn get_next_id() -> i64 {
    let mut counter = ID_COUNTER.lock().unwrap();
    *counter += 1;
    *counter
}

// Custom error type
#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    LockError,
    Other(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RepositoryError::NotFound => write!(f, "Record not found"),
            RepositoryError::LockError => write!(f, "Failed to acquire lock"),
            RepositoryError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl StdError for RepositoryError {}

// Repository implementation
pub struct ProdukRepository;

impl ProdukRepository {
    pub async fn tambah_produk(produk: &Produk) -> Result<i64, RepositoryError> {
        let id = get_next_id();
        
        let mut store = match PRODUCT_STORE.lock() {
            Ok(store) => store,
            Err(_) => return Err(RepositoryError::LockError),
        };
        
        // Create a new product with the generated ID
        let new_produk = Produk::with_id(
            id,
            produk.nama.clone(),
            produk.kategori.clone(),
            produk.harga,
            produk.stok,
            produk.deskripsi.clone(),
        );
        
        store.insert(id, new_produk);
        Ok(id)
    }
    
    pub async fn ambil_semua_produk() -> Result<Vec<Produk>, RepositoryError> {
        let store = match PRODUCT_STORE.lock() {
            Ok(store) => store,
            Err(_) => return Err(RepositoryError::LockError),
        };
        
        let produk_list: Vec<Produk> = store.values().cloned().collect();
        Ok(produk_list)
    }
    
    pub async fn ambil_produk_by_id(id: i64) -> Result<Option<Produk>, RepositoryError> {
        let store = match PRODUCT_STORE.lock() {
            Ok(store) => store,
            Err(_) => return Err(RepositoryError::LockError),
        };
        
        match store.get(&id) {
            Some(produk) => Ok(Some(produk.clone())),
            None => Ok(None),
        }
    }
    
    pub async fn filter_produk_by_kategori(kategori: &str) -> Result<Vec<Produk>, RepositoryError> {
        let store = match PRODUCT_STORE.lock() {
            Ok(store) => store,
            Err(_) => return Err(RepositoryError::LockError),
        };
        
        let filtered: Vec<Produk> = store.values()
            .filter(|p| p.kategori == kategori)
            .cloned()
            .collect();
            
        Ok(filtered)
    }
    
    pub async fn update_produk(id: i64, produk: &Produk) -> Result<bool, RepositoryError> {
        let mut store = match PRODUCT_STORE.lock() {
            Ok(store) => store,
            Err(_) => return Err(RepositoryError::LockError),
        };
        
        if store.contains_key(&id) {
            // Create a new product with the original ID
            let updated_produk = Produk::with_id(
                id,
                produk.nama.clone(),
                produk.kategori.clone(),
                produk.harga,
                produk.stok,
                produk.deskripsi.clone(),
            );
            
            store.insert(id, updated_produk);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub async fn hapus_produk(id: i64) -> Result<bool, RepositoryError> {
        let mut store = match PRODUCT_STORE.lock() {
            Ok(store) => store,
            Err(_) => return Err(RepositoryError::LockError),
        };
        
        match store.remove(&id) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
    
    pub async fn filter_produk_by_price_range(
        min_price: f64, 
        max_price: f64
    ) -> Result<Vec<Produk>, RepositoryError> {
        let store = match PRODUCT_STORE.lock() {
            Ok(store) => store,
            Err(_) => return Err(RepositoryError::LockError),
        };
        
        let filtered: Vec<Produk> = store.values()
            .filter(|p| p.harga >= min_price && p.harga <= max_price)
            .cloned()
            .collect();
            
        Ok(filtered)
    }
    
    pub async fn filter_produk_by_stock_availability(min_stock: u32) -> Result<Vec<Produk>, RepositoryError> {
        let store = match PRODUCT_STORE.lock() {
            Ok(store) => store,
            Err(_) => return Err(RepositoryError::LockError),
        };
        
        let filtered: Vec<Produk> = store.values()
            .filter(|p| p.stok >= min_stock)
            .cloned()
            .collect();
            
        Ok(filtered)
    }

    pub async fn clear_all() -> Result<(), RepositoryError> {
        let mut store = match PRODUCT_STORE.lock() {
            Ok(store) => store,
            Err(_) => return Err(RepositoryError::LockError),
        };
        
        store.clear();
        
        // Reset ID counter
        let mut counter = match ID_COUNTER.lock() {
            Ok(counter) => counter,
            Err(_) => return Err(RepositoryError::LockError),
        };
        *counter = 0;
        
        Ok(())
    }
}