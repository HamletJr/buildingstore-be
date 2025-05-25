use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::HashMap;
use lazy_static::lazy_static;
use crate::manajemen_produk::produk::model::Produk;
use std::error::Error as StdError;
use std::fmt;

// Global storage - perbaikan tipe data untuk HashMap
lazy_static! {
    pub static ref PRODUCT_STORE: Arc<Mutex<HashMap<i64, Produk>>> = Arc::new(Mutex::new(HashMap::new()));
    pub static ref ID_COUNTER: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));
}

// Error types
#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    LockError,
    ValidationError(String),
    Other(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RepositoryError::NotFound => write!(f, "Record not found"),
            RepositoryError::LockError => write!(f, "Failed to acquire lock"),
            RepositoryError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            RepositoryError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl StdError for RepositoryError {}

// Helper functions untuk ID management
pub fn get_next_id() -> Result<i64, RepositoryError> {
    let mut counter = ID_COUNTER.lock().map_err(|_| RepositoryError::LockError)?;
    *counter += 1;
    Ok(*counter)
}

pub fn reset_id_counter() -> Result<(), RepositoryError> {
    let mut counter = ID_COUNTER.lock().map_err(|_| RepositoryError::LockError)?;
    *counter = 0;
    Ok(())
}

// Consistent lock helpers - diperbaiki untuk HashMap
pub fn lock_store() -> Result<MutexGuard<'static, HashMap<i64, Produk>>, RepositoryError> {
    PRODUCT_STORE.lock().map_err(|_| RepositoryError::LockError)
}

pub fn lock_store_mut() -> Result<MutexGuard<'static, HashMap<i64, Produk>>, RepositoryError> {
    PRODUCT_STORE.lock().map_err(|_| RepositoryError::LockError)
}

pub fn lock_counter() -> Result<MutexGuard<'static, i64>, RepositoryError> {
    ID_COUNTER.lock().map_err(|_| RepositoryError::LockError)
}

// Validation helpers
pub fn validate_produk(produk: &Produk) -> Result<(), RepositoryError> {
    if produk.nama.trim().is_empty() {
        return Err(RepositoryError::ValidationError("Nama produk tidak boleh kosong".to_string()));
    }
    
    if produk.kategori.trim().is_empty() {
        return Err(RepositoryError::ValidationError("Kategori tidak boleh kosong".to_string()));
    }
    
    if produk.harga < 0.0 {
        return Err(RepositoryError::ValidationError("Harga tidak boleh negatif".to_string()));
    }
    
    if produk.stok < 0 {
        return Err(RepositoryError::ValidationError("Stok tidak boleh negatif".to_string()));
    }
    
    Ok(())
}

// Statistics helper
pub fn get_store_stats() -> Result<(usize, i64), RepositoryError> {
    let store = lock_store()?;
    let counter = lock_counter()?;
    Ok((store.len(), *counter))
}