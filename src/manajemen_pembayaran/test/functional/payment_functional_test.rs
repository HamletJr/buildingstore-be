use std::sync::Arc;
use uuid::Uuid;

use crate::main::model::payment::PaymentMethod;
use crate::main::enums::payment_status::PaymentStatus;
use crate::main::repository::payment_repository_impl::PaymentRepositoryImpl;
use crate::main::repository::payment_repository::PaymentRepository;
use crate::main::patterns::observer::{PaymentSubject, TransactionObserver, InventoryObserver, CustomerObserver, PaymentObserver};
use crate::main::service::payment_service_impl::PaymentServiceImpl;
use crate::main::service::payment_service::PaymentService;

fn setup_payment_service() -> Arc<dyn PaymentService> {
    let repository: Arc<dyn PaymentRepository> = Arc::new(PaymentRepositoryImpl::new());
    
    let mut subject = PaymentSubject::new();
    
    let transaction_observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
    let inventory_observer = Arc::new(InventoryObserver) as Arc<dyn PaymentObserver>;
    let customer_observer = Arc::new(CustomerObserver) as Arc<dyn PaymentObserver>;
    
    subject.attach(transaction_observer);
    subject.attach(inventory_observer);
    subject.attach(customer_observer);
    
    let subject = Arc::new(subject);
    
    Arc::new(PaymentServiceImpl::new(repository, subject))
}

#[test]
fn test_cash_payment_lifecycle() {
    let service = setup_payment_service();
    let transaction_id = format!("TRX-CASH-{}", Uuid::new_v4());
    
    let result = service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash);
    assert!(result.is_ok());
    let payment = result.unwrap();
    assert_eq!(payment.transaction_id, transaction_id);
    assert_eq!(payment.method, PaymentMethod::Cash);
    assert_eq!(payment.status, PaymentStatus::Paid);
    
    let found = service.get_payment_by_transaction(&transaction_id);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, payment.id);
}

#[test]
fn test_installment_payment_lifecycle() {
    let service = setup_payment_service();
    let transaction_id = format!("TRX-INST-{}", Uuid::new_v4());
    
    let result = service.create_payment(transaction_id.clone(), 3000.0, PaymentMethod::CreditCard);
    assert!(result.is_ok());
    let payment = result.unwrap();
    
    let result = service.update_payment_status(
        payment.id.clone(),
        PaymentStatus::Installment,
        Some(1000.0)
    );
    assert!(result.is_ok());
    let payment = result.unwrap();
    assert_eq!(payment.status, PaymentStatus::Installment);
    assert_eq!(payment.installments.len(), 1);
    assert_eq!(payment.installments[0].amount, 1000.0);
    
    let result = service.add_installment(&payment.id, 1000.0);
    assert!(result.is_ok());
    let payment = result.unwrap();
    assert_eq!(payment.installments.len(), 2);
    assert_eq!(payment.status, PaymentStatus::Installment);
    
    let result = service.add_installment(&payment.id, 1000.0);
    assert!(result.is_ok());
    let payment = result.unwrap();
    assert_eq!(payment.installments.len(), 3);
    assert_eq!(payment.status, PaymentStatus::Paid); 
    
    let result = service.add_installment(&payment.id, 500.0);
    assert!(result.is_err());
}

#[test]
fn test_bank_transfer_payment() {
    let service = setup_payment_service();
    let transaction_id = format!("TRX-BANK-{}", Uuid::new_v4());
    
    let result = service.create_payment(transaction_id.clone(), 5000.0, PaymentMethod::BankTransfer);
    assert!(result.is_ok());
    let payment = result.unwrap();
    assert_eq!(payment.method, PaymentMethod::BankTransfer);
    
    assert!(payment.id.starts_with("BANK-"));
}

#[test]
fn test_ewallet_payment() {
    let service = setup_payment_service();
    let transaction_id = format!("TRX-EWALLET-{}", Uuid::new_v4());
    
    let result = service.create_payment(transaction_id.clone(), 2500.0, PaymentMethod::EWallet);
    assert!(result.is_ok());
    let payment = result.unwrap();
    assert_eq!(payment.method, PaymentMethod::EWallet);
    
    assert!(payment.id.starts_with("EWALLET-"));
}

#[test]
fn test_delete_paid_payment() {
    let service = setup_payment_service();
    let transaction_id = format!("TRX-DELETE-{}", Uuid::new_v4());
    
    let payment = service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash).unwrap();
    
    let result = service.delete_payment(payment.id.clone());
    assert!(result.is_err());
    
    let found = service.get_payment(&payment.id);
    assert!(found.is_some());
}

#[test]
fn test_delete_installment_payment() {
    let service = setup_payment_service();
    let transaction_id = format!("TRX-DELETE-INST-{}", Uuid::new_v4());
    
    let payment = service.create_payment(transaction_id.clone(), 1000.0, PaymentMethod::Cash).unwrap();
    let payment = service.update_payment_status(
        payment.id.clone(),
        PaymentStatus::Installment,
        None
    ).unwrap();
    
    let result = service.delete_payment(payment.id.clone());
    
    assert!(result.is_ok());
    
    let found = service.get_payment(&payment.id);
    assert!(found.is_none());
}