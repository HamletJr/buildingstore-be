// ====== 3. OBSERVER PATTERN ======
// transaksi/observer/transaksi_observer.rs

use std::sync::{Arc, Mutex};

use crate::transaksi_penjualan::model::transaksi::Transaksi;

pub trait TransaksiObserver: Send + Sync {
    fn on_transaction_created(&self, transaksi: &Transaksi);
    fn on_transaction_updated(&self, old_transaksi: &Transaksi, new_transaksi: &Transaksi);
    fn on_transaction_completed(&self, transaksi: &Transaksi);
    fn on_transaction_cancelled(&self, transaksi: &Transaksi);
    fn on_item_added(&self, transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi);
    fn on_item_updated(&self, transaksi_id: i32, old_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi, new_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi);
    fn on_item_removed(&self, transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi);
    fn get_name(&self) -> &'static str;
}

// Observer untuk logging
pub struct LoggingObserver;

impl TransaksiObserver for LoggingObserver {
    fn on_transaction_created(&self, transaksi: &Transaksi) {
        println!("[LOG] Transaksi baru dibuat - ID: {}, Pelanggan: {}, Total: {}", 
                transaksi.id, transaksi.nama_pelanggan, transaksi.total_harga);
    }
    
    fn on_transaction_updated(&self, old: &Transaksi, new: &Transaksi) {
        println!("[LOG] Transaksi diperbarui - ID: {}, Total: {} -> {}", 
                new.id, old.total_harga, new.total_harga);
    }
    
    fn on_transaction_completed(&self, transaksi: &Transaksi) {
        println!("[LOG] Transaksi diselesaikan - ID: {}, Total: {}", 
                transaksi.id, transaksi.total_harga);
    }
    
    fn on_transaction_cancelled(&self, transaksi: &Transaksi) {
        println!("[LOG] Transaksi dibatalkan - ID: {}, Stok dikembalikan", transaksi.id);
    }
    
    fn on_item_added(&self, transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[LOG] Item ditambahkan ke transaksi {} - Produk: {}, Qty: {}", 
                transaksi_id, item.id_produk, item.jumlah);
    }
    
    fn on_item_updated(&self, transaksi_id: i32, old_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi, new_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[LOG] Item diperbarui di transaksi {} - Produk: {}, Qty: {} -> {}", 
                transaksi_id, new_item.id_produk, old_item.jumlah, new_item.jumlah);
    }
    
    fn on_item_removed(&self, transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[LOG] Item dihapus dari transaksi {} - Produk: {}", 
                transaksi_id, item.id_produk);
    }
    
    fn get_name(&self) -> &'static str { "logging" }
}

// Observer untuk notifikasi
pub struct NotificationObserver;

impl TransaksiObserver for NotificationObserver {
    fn on_transaction_created(&self, transaksi: &Transaksi) {
        println!("[NOTIF] Email: Transaksi #{} berhasil dibuat untuk {}", 
                transaksi.id, transaksi.nama_pelanggan);
    }
    
    fn on_transaction_updated(&self, _old: &Transaksi, new: &Transaksi) {
        println!("[NOTIF] SMS: Transaksi #{} telah diperbarui", new.id);
    }
    
    fn on_transaction_completed(&self, transaksi: &Transaksi) {
        println!("[NOTIF] Email: Struk digital untuk transaksi #{} telah dikirim", transaksi.id);
        println!("[NOTIF] SMS: Terima kasih atas pembelian Anda! Total: {}", transaksi.total_harga);
    }
    
    fn on_transaction_cancelled(&self, transaksi: &Transaksi) {
        println!("[NOTIF] Email: Transaksi #{} telah dibatalkan", transaksi.id);
    }
    
    fn on_item_added(&self, transaksi_id: i32, _item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[NOTIF] Item ditambahkan ke keranjang transaksi #{}", transaksi_id);
    }
    
    fn on_item_updated(&self, transaksi_id: i32, _old_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi, _new_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[NOTIF] Keranjang transaksi #{} diperbarui", transaksi_id);
    }
    
    fn on_item_removed(&self, transaksi_id: i32, _item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[NOTIF] Item dihapus dari keranjang transaksi #{}", transaksi_id);
    }
    
    fn get_name(&self) -> &'static str { "notification" }
}

// Observer untuk inventory management
pub struct InventoryObserver;

impl TransaksiObserver for InventoryObserver {
    fn on_transaction_created(&self, _transaksi: &Transaksi) {
        // Stok sudah dikurangi saat create, tidak perlu aksi tambahan
    }
    
    fn on_transaction_updated(&self, _old: &Transaksi, _new: &Transaksi) {
        println!("[INVENTORY] Mengecek konsistensi stok setelah update transaksi");
    }
    
    fn on_transaction_completed(&self, transaksi: &Transaksi) {
        println!("[INVENTORY] Finalisasi pengurangan stok untuk transaksi #{}", transaksi.id);
    }
    
    fn on_transaction_cancelled(&self, transaksi: &Transaksi) {
        println!("[INVENTORY] Stok telah dikembalikan untuk transaksi #{}", transaksi.id);
    }
    
    fn on_item_added(&self, _transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[INVENTORY] Mengurangi stok produk {} sebanyak {}", 
                item.id_produk, item.jumlah);
    }
    
    fn on_item_updated(&self, _transaksi_id: i32, old_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi, new_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        let diff = new_item.jumlah as i32 - old_item.jumlah as i32;
        if diff > 0 {
            println!("[INVENTORY] Mengurangi stok produk {} sebanyak {}", 
                    new_item.id_produk, diff);
        } else if diff < 0 {
            println!("[INVENTORY] Menambah stok produk {} sebanyak {}", 
                    new_item.id_produk, -diff);
        }
    }
    
    fn on_item_removed(&self, _transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[INVENTORY] Mengembalikan stok produk {} sebanyak {}", 
                item.id_produk, item.jumlah);
    }
    
    fn get_name(&self) -> &'static str { "inventory" }
}

// Observer untuk analytics
pub struct AnalyticsObserver;

impl TransaksiObserver for AnalyticsObserver {
    fn on_transaction_created(&self, transaksi: &Transaksi) {
        println!("[ANALYTICS] Recording new transaction: Customer {}, Amount: {}", 
                transaksi.id_pelanggan, transaksi.total_harga);
    }
    
    fn on_transaction_updated(&self, old: &Transaksi, new: &Transaksi) {
        println!("[ANALYTICS] Transaction value changed: {} -> {}", 
                old.total_harga, new.total_harga);
    }
    
    fn on_transaction_completed(&self, transaksi: &Transaksi) {
        println!("[ANALYTICS] Completed transaction recorded: ID {}, Revenue: {}", 
                transaksi.id, transaksi.total_harga);
    }
    
    fn on_transaction_cancelled(&self, transaksi: &Transaksi) {
        println!("[ANALYTICS] Cancelled transaction recorded: ID {}, Lost revenue: {}", 
                transaksi.id, transaksi.total_harga);
    }
    
    fn on_item_added(&self, _transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[ANALYTICS] Product {} added to cart, Revenue: {}", 
                item.id_produk, item.subtotal);
    }
    
    fn on_item_updated(&self, _transaksi_id: i32, old_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi, new_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[ANALYTICS] Product {} quantity changed: {} -> {}, Revenue impact: {}", 
                new_item.id_produk, old_item.jumlah, new_item.jumlah, 
                new_item.subtotal - old_item.subtotal);
    }
    
    fn on_item_removed(&self, _transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        println!("[ANALYTICS] Product {} removed from cart, Lost revenue: {}", 
                item.id_produk, item.subtotal);
    }
    
    fn get_name(&self) -> &'static str { "analytics" }
}

// Subject (Observable) untuk manage observers
pub struct TransaksiSubject {
    observers: Vec<Arc<dyn TransaksiObserver>>,
}

impl TransaksiSubject {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }
    
    pub fn attach(&mut self, observer: Arc<dyn TransaksiObserver>) {
        self.observers.push(observer);
    }
    
    pub fn detach(&mut self, observer_name: &str) {
        self.observers.retain(|obs| obs.get_name() != observer_name);
    }
    
    pub fn notify_created(&self, transaksi: &Transaksi) {
        for observer in &self.observers {
            observer.on_transaction_created(transaksi);
        }
    }
    
    pub fn notify_updated(&self, old_transaksi: &Transaksi, new_transaksi: &Transaksi) {
        for observer in &self.observers {
            observer.on_transaction_updated(old_transaksi, new_transaksi);
        }
    }
    
    pub fn notify_completed(&self, transaksi: &Transaksi) {
        for observer in &self.observers {
            observer.on_transaction_completed(transaksi);
        }
    }
    
    pub fn notify_cancelled(&self, transaksi: &Transaksi) {
        for observer in &self.observers {
            observer.on_transaction_cancelled(transaksi);
        }
    }
    
    pub fn notify_item_added(&self, transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        for observer in &self.observers {
            observer.on_item_added(transaksi_id, item);
        }
    }
    
    pub fn notify_item_updated(&self, transaksi_id: i32, old_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi, new_item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        for observer in &self.observers {
            observer.on_item_updated(transaksi_id, old_item, new_item);
        }
    }
    
    pub fn notify_item_removed(&self, transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
        for observer in &self.observers {
            observer.on_item_removed(transaksi_id, item);
        }
    }
    
    pub fn get_observer_count(&self) -> usize {
        self.observers.len()
    }
    
    pub fn get_observer_names(&self) -> Vec<&str> {
        self.observers.iter().map(|obs| obs.get_name()).collect()
    }
}

// Singleton untuk global observer management
lazy_static::lazy_static! {
    pub static ref GLOBAL_OBSERVER_MANAGER: Arc<Mutex<TransaksiSubject>> = {
        let mut subject = TransaksiSubject::new();
        
        // Register default observers
        subject.attach(Arc::new(LoggingObserver));
        subject.attach(Arc::new(NotificationObserver));
        subject.attach(Arc::new(InventoryObserver));
        subject.attach(Arc::new(AnalyticsObserver));
        
        Arc::new(Mutex::new(subject))
    };
}

// Helper function untuk mudah akses global observer
pub fn notify_transaction_created(transaksi: &Transaksi) {
    if let Ok(subject) = GLOBAL_OBSERVER_MANAGER.lock() {
        subject.notify_created(transaksi);
    }
}

pub fn notify_transaction_updated(old_transaksi: &Transaksi, new_transaksi: &Transaksi) {
    if let Ok(subject) = GLOBAL_OBSERVER_MANAGER.lock() {
        subject.notify_updated(old_transaksi, new_transaksi);
    }
}

pub fn notify_transaction_completed(transaksi: &Transaksi) {
    if let Ok(subject) = GLOBAL_OBSERVER_MANAGER.lock() {
        subject.notify_completed(transaksi);
    }
}

pub fn notify_transaction_cancelled(transaksi: &Transaksi) {
    if let Ok(subject) = GLOBAL_OBSERVER_MANAGER.lock() {
        subject.notify_cancelled(transaksi);
    }
}

pub fn notify_item_added(transaksi_id: i32, item: &crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi) {
    if let Ok(subject) = GLOBAL_OBSERVER_MANAGER.lock() {
        subject.notify_item_added(transaksi_id, item);
    }
}