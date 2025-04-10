use chrono::Utc;
use uuid::Uuid;

use crate::main::model::payment::{Payment, PaymentMethod};
use crate::main::enums::payment_status::PaymentStatus;
use crate::main::patterns::state::{PaymentState, PaidState, InstallmentState};

#[test]
fn test_paid_state_process_payment() {
    let state = PaidState;
    let mut payment = Payment {
        id: format!("PMT-{}", Uuid::new_v4()),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Paid,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    let result = state.process_payment(&mut payment, 500.0);
    
    assert!(result.is_err());
    assert_eq!(payment.installments.len(), 0);
}

#[test]
fn test_installment_state_process_payment() {
    let state = InstallmentState;
    let mut payment = Payment {
        id: format!("PMT-{}", Uuid::new_v4()),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Installment,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    let result = state.process_payment(&mut payment, 400.0);
    
    assert!(result.is_ok()); 
    assert_eq!(payment.installments.len(), 1);
    assert_eq!(payment.installments[0].amount, 400.0);
    assert_eq!(payment.status, PaymentStatus::Installment);
    
    let result = state.process_payment(&mut payment, 600.0);
    
    assert!(result.is_ok());
    assert_eq!(payment.installments.len(), 2);
    assert_eq!(payment.status, PaymentStatus::Paid);
}

#[test]
fn test_installment_state_invalid_amount() {
    let state = InstallmentState;
    let mut payment = Payment {
        id: format!("PMT-{}", Uuid::new_v4()),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Installment,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    let result = state.process_payment(&mut payment, -100.0);
    
    assert!(result.is_err());
    assert_eq!(payment.installments.len(), 0);
}

#[test]
fn test_state_can_delete() {
    let paid_state = PaidState;
    assert!(!paid_state.can_delete());
    
    let installment_state = InstallmentState;
    assert!(installment_state.can_delete());
}