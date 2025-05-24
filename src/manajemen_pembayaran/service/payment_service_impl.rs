use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::repository::payment_repository::PaymentRepository;
use crate::manajemen_pembayaran::service::payment_service::PaymentService;

pub struct PaymentServiceImpl {
    repository: Arc<dyn PaymentRepository>,
}

impl PaymentServiceImpl {
    pub fn new(repository: Arc<dyn PaymentRepository>) -> Self {
        Self {
            repository,
        }
    }
    
    fn process_payment(&self, amount: f64, transaction_id: &str, method: &PaymentMethod) -> Result<String, String> {
        println!("Memproses pembayaran {:?} sebesar Rp{} untuk transaksi {}", method, amount, transaction_id);
        
        if amount <= 0.0 {
            return Err("Jumlah pembayaran harus lebih dari 0".to_string());
        }
        
        let payment_id = match method {
            PaymentMethod::Cash => format!("CASH-{}", Uuid::new_v4()),
            PaymentMethod::CreditCard => format!("CC-{}", Uuid::new_v4()),
            PaymentMethod::BankTransfer => format!("BANK-{}", Uuid::new_v4()),
            PaymentMethod::EWallet => format!("EWALLET-{}", Uuid::new_v4()),
        };
        
        Ok(payment_id)
    }
}

impl PaymentService for PaymentServiceImpl {
    fn create_payment(
        &self,
        transaction_id: String,
        amount: f64,
        method: PaymentMethod,
    ) -> Result<Payment, String> {
        if self.repository.find_by_transaction_id(&transaction_id).is_some() {
            return Err(format!("Transaksi {} sudah memiliki pembayaran", transaction_id));
        }

        let payment_id = self.process_payment(amount, &transaction_id, &method)?;

        let payment = Payment {
            id: payment_id,
            transaction_id: transaction_id,
            amount,
            method,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        let saved_payment = self.repository.save(payment)?;
        
        println!("Payment created for transaction ID {}: {:?}", saved_payment.transaction_id, saved_payment.status);
        
        Ok(saved_payment)
    }
    
    fn update_payment_status(
        &self,
        payment_id: String,
        new_status: PaymentStatus,
        additional_amount: Option<f64>,
    ) -> Result<Payment, String> {
        let mut payment = self.repository
            .find_by_id(&payment_id)
            .ok_or_else(|| format!("Pembayaran dengan ID {} tidak ditemukan", payment_id))?;

        payment.status = new_status.clone();
        
        if let Some(additional) = additional_amount {
            if additional <= 0.0 {
                return Err("Jumlah tambahan harus lebih dari 0".to_string());
            }
            payment.amount += additional;
            
            if new_status == PaymentStatus::Installment {
                let installment = Installment {
                    id: Uuid::new_v4().to_string(),
                    payment_id: payment_id.to_string(),
                    amount: additional,
                    payment_date: Utc::now(),
                };
                payment.installments.push(installment);
            }
        }
        
        let updated_payment = self.repository.update(payment)?;
        
        println!("Payment status updated for ID {}: {:?}", updated_payment.id, updated_payment.status);
        
        Ok(updated_payment)
    }

    fn delete_payment(&self, payment_id: String) -> Result<(), String> {
        let payment = self.repository
            .find_by_id(&payment_id)
            .ok_or_else(|| format!("Pembayaran dengan ID {} tidak ditemukan", payment_id))?;
            
        if payment.status == PaymentStatus::Paid {
            return Err(format!("Pembayaran dengan ID {} sudah lunas dan tidak dapat dihapus", payment_id));
        }
        
        self.repository.delete(&payment_id)?;
        
        println!("Payment with ID {} deleted", payment_id);
        
        Ok(())
    }

    fn get_payment(&self, payment_id: &str) -> Option<Payment> {
        self.repository.find_by_id(payment_id)
    }

    fn get_payment_by_transaction(&self, transaction_id: &str) -> Option<Payment> {
        self.repository.find_by_transaction_id(transaction_id)
    }

    fn get_all_payments(&self, filters: Option<HashMap<String, String>>) -> Vec<Payment> {
        self.repository.find_all(filters)
    }

    fn add_installment(&self, payment_id: &str, amount: f64) -> Result<Payment, String> {
        if amount <= 0.0 {
            return Err("Jumlah cicilan harus lebih dari 0".to_string());
        }
        
        let mut payment = self.repository
            .find_by_id(payment_id)
            .ok_or_else(|| format!("Pembayaran dengan ID {} tidak ditemukan", payment_id))?;
            
        if payment.status != PaymentStatus::Installment {
            return Err(format!("Pembayaran dengan ID {} bukan cicilan", payment_id));
        }
        
        let installment = Installment {
            id: Uuid::new_v4().to_string(),
            payment_id: payment_id.to_string(),
            amount,
            payment_date: Utc::now(),
        };
        
        payment.installments.push(installment);
        
        let total_installments: f64 = payment.installments.iter().map(|i| i.amount).sum();
        
        if total_installments >= payment.amount {
            payment.status = PaymentStatus::Paid;
        }
        
        let updated_payment = self.repository.update(payment)?;
        
        println!("Added installment to payment ID {}, new status: {:?}", payment_id, updated_payment.status);
        
        Ok(updated_payment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::collections::HashMap;
    
    use crate::manajemen_pembayaran::repository::payment_repository_impl::PaymentRepositoryImpl;

    #[test]
    fn test_create_payment() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone()));
        
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
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone()));
        
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash).unwrap();
        
        let result = service.create_payment(transaction_id.clone(), 2000.0, PaymentMethod::CreditCard);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_update_payment_status() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone()));
        
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
    }

    #[test]
    fn test_add_installment() {
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone()));
        
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
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone()));
        
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
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone()));
        
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
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone()));
        
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
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository.clone()));
        
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        let payment = service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash).unwrap();
        
        let result = service.delete_payment(payment.id.clone());
        
        assert!(result.is_err());
        assert!(service.get_payment(&payment.id).is_some());
    }
}
