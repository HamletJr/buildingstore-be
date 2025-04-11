use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::main::model::payment::Payment;
use crate::main::enums::payment_status::PaymentStatus;
use crate::main::repository::payment_repository::PaymentRepository;

pub struct PaymentRepositoryImpl {
    payments: Arc<Mutex<HashMap<String, Payment>>>,
}

impl PaymentRepositoryImpl {
    pub fn new() -> Self {
        Self {
            payments: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl PaymentRepository for PaymentRepositoryImpl {
    fn save(&self, payment: Payment) -> Result<Payment, String> {
        let mut payments = self.payments.lock().unwrap();
        let payment_clone = payment.clone();
        payments.insert(payment.id.clone(), payment);
        Ok(payment_clone)
    }

    fn find_by_id(&self, id: &str) -> Option<Payment> {
        let payments = self.payments.lock().unwrap();
        payments.get(id).cloned()
    }

    fn find_by_transaction_id(&self, transaction_id: &str) -> Option<Payment> {
        let payments = self.payments.lock().unwrap();
        payments
            .values()
            .find(|p| p.transaction_id == transaction_id)
            .cloned()
    }

    fn find_all(&self, filters: Option<HashMap<String, String>>) -> Vec<Payment> {
        let payments = self.payments.lock().unwrap();
        let mut result: Vec<Payment> = payments.values().cloned().collect();

        if let Some(filter_map) = filters {
            // Filter by status
            if let Some(status_str) = filter_map.get("status") {
                if let Some(status) = PaymentStatus::from_string(status_str) {
                    result = result
                        .into_iter()
                        .filter(|p| p.status == status)
                        .collect();
                }
            }
            
            // Filter by method
            if let Some(method) = filter_map.get("method") {
                result = result
                    .into_iter()
                    .filter(|p| format!("{:?}", p.method).to_uppercase() == method.to_uppercase())
                    .collect();
            }
        }

        result
    }

    fn update(&self, payment: Payment) -> Result<Payment, String> {
        let mut payments = self.payments.lock().unwrap();
        if !payments.contains_key(&payment.id) {
            return Err(format!("Pembayaran dengan ID {} tidak ditemukan", payment.id));
        }
        let payment_clone = payment.clone();
        payments.insert(payment.id.clone(), payment);
        Ok(payment_clone)
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let mut payments = self.payments.lock().unwrap();
        if payments.remove(id).is_none() {
            return Err(format!("Pembayaran dengan ID {} tidak ditemukan", id));
        }
        Ok(())
    }
}