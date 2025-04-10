use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

use crate::main::model::payment::{Payment, PaymentMethod};
use crate::main::enums::payment_status::PaymentStatus;
use crate::main::repository::payment_repository_impl::PaymentRepositoryImpl;
use crate::main::repository::payment_repository::PaymentRepository;

#[test]
fn test_save_payment() {
    let repository = PaymentRepositoryImpl::new();
    let payment_id = format!("PMT-{}", Uuid::new_v4());
    let transaction_id = format!("TRX-{}", Uuid::new_v4());
    
    let payment = Payment {
        id: payment_id.clone(),
        transaction_id: transaction_id.clone(),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Paid,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    let result = repository.save(payment.clone());
    
    assert!(result.is_ok());
    let saved_payment = result.unwrap();
    assert_eq!(saved_payment.id, payment_id);
    assert_eq!(saved_payment.transaction_id, transaction_id);
    assert_eq!(saved_payment.amount, 1000.0);
}

#[test]
fn test_find_by_id() {
    let repository = PaymentRepositoryImpl::new();
    let payment_id = format!("PMT-{}", Uuid::new_v4());
    
    let payment = Payment {
        id: payment_id.clone(),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Paid,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    repository.save(payment.clone()).unwrap();
    
    let result = repository.find_by_id(&payment_id);
    
    assert!(result.is_some());
    let found_payment = result.unwrap();
    assert_eq!(found_payment.id, payment_id);
}

#[test]
fn test_find_by_transaction_id() {
    let repository = PaymentRepositoryImpl::new();
    let transaction_id = format!("TRX-{}", Uuid::new_v4());
    
    let payment = Payment {
        id: format!("PMT-{}", Uuid::new_v4()),
        transaction_id: transaction_id.clone(),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Paid,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    repository.save(payment.clone()).unwrap();
    
    let result = repository.find_by_transaction_id(&transaction_id);
    
    assert!(result.is_some());
    let found_payment = result.unwrap();
    assert_eq!(found_payment.transaction_id, transaction_id);
}

#[test]
fn test_update_payment() {
    let repository = PaymentRepositoryImpl::new();
    let payment_id = format!("PMT-{}", Uuid::new_v4());
    
    let payment = Payment {
        id: payment_id.clone(),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Installment,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    repository.save(payment.clone()).unwrap();
    
    let mut updated_payment = payment.clone();
    updated_payment.status = PaymentStatus::Paid;
    let result = repository.update(updated_payment);
    
    assert!(result.is_ok());
    let found = repository.find_by_id(&payment_id).unwrap();
    assert_eq!(found.status, PaymentStatus::Paid);
}

#[test]
fn test_delete_payment() {
    let repository = PaymentRepositoryImpl::new();
    let payment_id = format!("PMT-{}", Uuid::new_v4());
    
    let payment = Payment {
        id: payment_id.clone(),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Paid,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    repository.save(payment.clone()).unwrap();
    
    let result = repository.delete(&payment_id);
    
    assert!(result.is_ok());
    assert!(repository.find_by_id(&payment_id).is_none());
}

#[test]
fn test_find_all_with_filters() {
    let repository = PaymentRepositoryImpl::new();
    
    let cash_payment = Payment {
        id: format!("PMT-{}", Uuid::new_v4()),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Paid,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    let credit_payment = Payment {
        id: format!("PMT-{}", Uuid::new_v4()),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 2000.0,
        method: PaymentMethod::CreditCard,
        status: PaymentStatus::Installment,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    repository.save(cash_payment.clone()).unwrap();
    repository.save(credit_payment.clone()).unwrap();
    
    let mut filters = HashMap::new();
    filters.insert("status".to_string(), "LUNAS".to_string());
    let status_results = repository.find_all(Some(filters));
    
    assert_eq!(status_results.len(), 1);
    assert_eq!(status_results[0].status, PaymentStatus::Paid);
    
    let all_results = repository.find_all(None);
    
    assert_eq!(all_results.len(), 2);
}