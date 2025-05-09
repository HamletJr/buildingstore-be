use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::manajemen_pembayaran::repository::payment_repository::PaymentRepository;
use crate::manajemen_pembayaran::service::payment_service::PaymentService;
use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::patterns::observer::PaymentSubject;

pub struct PaymentServiceImpl {
    repository: Arc<dyn PaymentRepository>,
    observer_subject: Arc<PaymentSubject>,
}

impl PaymentServiceImpl {
    pub fn new(repository: Arc<dyn PaymentRepository>, observer_subject: Arc<PaymentSubject>) -> Self {
        Self {
            repository,
            observer_subject,
        }
    }
}

impl PaymentService for PaymentServiceImpl {
    fn create_payment(&self, transaction_id: String, amount: f64, method: PaymentMethod) -> Result<Payment, String> {
        if amount <= 0.0 {
            return Err("Jumlah pembayaran harus lebih dari 0".to_string());
        }
        
        if let Some(_) = self.get_payment_by_transaction(&transaction_id) {
            return Err(format!("Transaksi dengan ID {} sudah memiliki pembayaran", transaction_id));
        }
        
        let payment = Payment::new(transaction_id, amount, method);
        
        self.repository.save(payment.clone());
        
        let subject = Arc::clone(&self.observer_subject);
        subject.notify_payment_created(&payment);
        
        Ok(payment)
    }
    
    fn update_payment_status(&self, payment_id: String, new_status: PaymentStatus, initial_amount: Option<f64>) -> Result<Payment, String> {
        let mut payment = match self.repository.find_by_id(&payment_id) {
            Some(p) => p,
            None => return Err(format!("Pembayaran dengan ID {} tidak ditemukan", payment_id)),
        };
        
        payment.update_status(new_status.clone(), initial_amount)?;
        
        self.repository.update(payment.clone());
        
        let subject = Arc::clone(&self.observer_subject);
        subject.notify_payment_status_changed(&payment);
        
        Ok(payment)
    }
    
    fn add_installment(&self, payment_id: &str, amount: f64) -> Result<Payment, String> {
        let mut payment = match self.repository.find_by_id(payment_id) {
            Some(p) => p,
            None => return Err(format!("Pembayaran dengan ID {} tidak ditemukan", payment_id)),
        };
        
        if amount <= 0.0 {
            return Err("Jumlah cicilan harus lebih dari 0".to_string());
        }
        
        payment.add_installment(amount)?;
        
        if payment.is_fully_paid() {
            payment.update_status(PaymentStatus::Paid, None)?;
        }
        
        self.repository.update(payment.clone());
        
        let subject = Arc::clone(&self.observer_subject);
        subject.notify_installment_added(&payment);
        
        Ok(payment)
    }
    
    fn get_payment(&self, payment_id: &str) -> Option<Payment> {
        self.repository.find_by_id(payment_id)
    }
    
    fn get_payment_by_transaction(&self, transaction_id: &str) -> Option<Payment> {
        self.repository.find_by_transaction_id(transaction_id)
    }
    
    fn get_all_payments(&self, filters: Option<HashMap<String, String>>) -> Vec<Payment> {
        let mut payments = self.repository.find_all();
        
        if let Some(filters) = filters {
            if let Some(status) = filters.get("status") {
                let status = match status.to_uppercase().as_str() {
                    "LUNAS" => Some(PaymentStatus::Paid),
                    "CICILAN" => Some(PaymentStatus::Installment),
                    "PENDING" => Some(PaymentStatus::Pending),
                    "CANCELLED" => Some(PaymentStatus::Cancelled),
                    _ => None,
                };
                
                if let Some(status) = status {
                    payments = payments.into_iter().filter(|p| p.status == status).collect();
                }
            }
            
            if let Some(method) = filters.get("method") {
                let method = match method.to_uppercase().as_str() {
                    "CASH" => Some(PaymentMethod::Cash),
                    "CARD" | "CREDIT_CARD" => Some(PaymentMethod::CreditCard),
                    "TRANSFER" | "BANK_TRANSFER" => Some(PaymentMethod::BankTransfer),
                    "E_WALLET" => Some(PaymentMethod::EWallet),
                    _ => None,
                };
                
                if let Some(method) = method {
                    payments = payments.into_iter().filter(|p| p.method == method).collect();
                }
            }
        }
        
        payments
    }
    
    fn delete_payment(&self, payment_id: String) -> Result<(), String> {
        let payment = match self.repository.find_by_id(&payment_id) {
            Some(p) => p,
            None => return Err(format!("Pembayaran dengan ID {} tidak ditemukan", payment_id)),
        };
        
        self.repository.delete(&payment_id);
        
        let subject = Arc::clone(&self.observer_subject);
        subject.notify_payment_deleted(&payment);
        
        Ok(())
    }
    
    fn update_payment_proof(&self, payment_id: &str, proof: String) -> Result<Payment, String> {
        let mut payment = match self.repository.find_by_id(payment_id) {
            Some(p) => p,
            None => return Err(format!("Pembayaran dengan ID {} tidak ditemukan", payment_id)),
        };
        
        payment.set_payment_proof(proof);
        
        self.repository.update(payment.clone());
        
        Ok(payment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::collections::HashMap;
    
    use crate::manajemen_pembayaran::repository::payment_repository_impl::PaymentRepositoryImpl;
    use crate::manajemen_pembayaran::patterns::observer::{PaymentSubject, TransactionObserver, PaymentObserver};

    #[test]
    fn test_create_payment() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
        subject.attach(observer);
        let subject = Arc::new(subject);
        
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone(), subject.clone()));
        
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let result = service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash);
        
        assert!(result.is_ok());
        let payment = result.unwrap();
        assert_eq!(payment.transaction_id, transaction_id);
        assert_eq!(payment.amount, 1000.0);
        assert_eq!(payment.method, PaymentMethod::Cash);
        assert_eq!(payment.status, PaymentStatus::Paid);
    }

    #[test]
    fn test_create_payment_with_existing_transaction() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
        subject.attach(observer);
        let subject = Arc::new(subject);
        
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone(), subject.clone()));
        
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash).unwrap();
        
        let result = service.create_payment(transaction_id.clone(), 2000.0, PaymentMethod::CreditCard);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_update_payment_status() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
        subject.attach(observer);
        let subject = Arc::new(subject);
        
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone(), subject.clone()));
        
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        let payment = service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash).unwrap();
        
        let result = service.update_payment_status(
            payment.id.clone(),
            PaymentStatus::Installment,
            Some(500.0)
        );
        
        assert!(result.is_ok());
        let updated_payment = result.unwrap();
        assert_eq!(updated_payment.status, PaymentStatus::Installment);
        assert_eq!(updated_payment.installments.len(), 1);
        assert_eq!(updated_payment.installments[0].amount, 500.0);
    }

    #[test]
    fn test_add_installment() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
        subject.attach(observer);
        let subject = Arc::new(subject);
        
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone(), subject.clone()));
        
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        let payment = service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash).unwrap();
        
        let payment = service.update_payment_status(
            payment.id.clone(),
            PaymentStatus::Installment,
            None
        ).unwrap();
        
        let result = service.add_installment(&payment.id, 400.0);
        
        assert!(result.is_ok());
        let payment_with_installment = result.unwrap();
        assert_eq!(payment_with_installment.installments.len(), 1);
        
        let result = service.add_installment(&payment.id, 600.0);
        
        assert!(result.is_ok());
        let completed_payment = result.unwrap();
        assert_eq!(completed_payment.installments.len(), 2);
        assert_eq!(completed_payment.status, PaymentStatus::Paid);
        
        let result = service.add_installment(&payment.id, 100.0);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_get_payment() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
        subject.attach(observer);
        let subject = Arc::new(subject);
        
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone(), subject.clone()));
        
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        let payment = service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash).unwrap();
        
        let result = service.get_payment(&payment.id);
        
        assert!(result.is_some());
        let found_payment = result.unwrap();
        assert_eq!(found_payment.id, payment.id);
        
        let result = service.get_payment_by_transaction(&transaction_id);
        
        assert!(result.is_some());
        let found_payment = result.unwrap();
        assert_eq!(found_payment.transaction_id, transaction_id);
    }

    #[test]
    fn test_get_all_payments() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
        subject.attach(observer);
        let subject = Arc::new(subject);
        
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone(), subject.clone()));
        
        service.create_payment(format!("TRX-1-{}", Uuid::new_v4()), 1000.0, PaymentMethod::Cash).unwrap();
        
        let p2 = service.create_payment(format!("TRX-2-{}", Uuid::new_v4()), 2000.0, PaymentMethod::CreditCard).unwrap();
        service.update_payment_status(p2.id, PaymentStatus::Installment, None).unwrap();
        
        let all_payments = service.get_all_payments(None);
        
        assert_eq!(all_payments.len(), 2);
        
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "LUNAS".to_string());
        let paid_payments = service.get_all_payments(Some(filters));
        
        assert_eq!(paid_payments.len(), 1);
        assert_eq!(paid_payments[0].status, PaymentStatus::Paid);
    }

    #[test]
    fn test_delete_payment() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
        subject.attach(observer);
        let subject = Arc::new(subject);
        
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone(), subject.clone()));
        
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        let payment = service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash).unwrap();
        let payment = service.update_payment_status(payment.id.clone(), PaymentStatus::Installment, None).unwrap();
        
        let result = service.delete_payment(payment.id.clone());
        
        assert!(result.is_ok());
        assert!(service.get_payment(&payment.id).is_none());
    }

    #[test]
    fn test_delete_paid_payment() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
        subject.attach(observer);
        let subject = Arc::new(subject);
        
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone(), subject.clone()));
        
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        let payment = service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash).unwrap();
        
        let result = service.delete_payment(payment.id.clone());
        
        assert!(result.is_err());
        assert!(service.get_payment(&payment.id).is_some());
    }
}
