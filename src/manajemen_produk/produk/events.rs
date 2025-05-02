use std::sync::{Arc, Mutex};
use super::model::Produk;

pub trait ProdukObserver: Send + Sync {
    fn on_stock_changed(&self, produk: &Produk, old_stock: u32);
}

#[derive(Clone)]
pub struct ProdukEventPublisher {
    observers: Arc<Mutex<Vec<Arc<dyn ProdukObserver>>>>,
}

impl ProdukEventPublisher {
    pub fn new() -> Self {
        Self {
            observers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_observer(&self, observer: Arc<dyn ProdukObserver>) {
        self.observers.lock().unwrap().push(observer);
    }

    pub fn notify_stock_changed(&self, produk: &Produk, old_stock: u32) {
        for observer in self.observers.lock().unwrap().iter() {
            observer.on_stock_changed(produk, old_stock);
        }
    }
}