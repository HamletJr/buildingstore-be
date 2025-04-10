use std::sync::Arc;
use crate::main::model::payment::Payment;

pub trait PaymentObserver: Send + Sync {
    fn payment_updated(&self, payment: &Payment);
}

pub struct TransactionObserver;
impl PaymentObserver for TransactionObserver {
    fn payment_updated(&self, payment: &Payment) {
        println!("Observer: Memperbarui status transaksi untuk ID {}", payment.transaction_id);
    }
}

pub struct InventoryObserver;
impl PaymentObserver for InventoryObserver {
    fn payment_updated(&self, payment: &Payment) {
        println!("Observer: Memperbarui stok untuk transaksi ID {}", payment.transaction_id);
    }
}

pub struct CustomerObserver;
impl PaymentObserver for CustomerObserver {
    fn payment_updated(&self, payment: &Payment) {
        println!("Observer: Memperbarui riwayat pelanggan untuk transaksi ID {}", payment.transaction_id);
    }
}

pub struct PaymentSubject {
    observers: Vec<Arc<dyn PaymentObserver>>,
}

impl PaymentSubject {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn attach(&mut self, observer: Arc<dyn PaymentObserver>) {
        self.observers.push(observer);
    }

    pub fn notify(&self, payment: &Payment) {
        for observer in &self.observers {
            observer.payment_updated(payment);
        }
    }
}