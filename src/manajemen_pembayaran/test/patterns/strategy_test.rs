use uuid::Uuid;

use crate::main::model::payment::PaymentMethod;
use crate::main::patterns::strategy::{PaymentProcessor, CashPaymentProcessor, CreditCardPaymentProcessor, BankTransferPaymentProcessor, EWalletPaymentProcessor};

#[test]
fn test_cash_payment_processor() {
    let processor = CashPaymentProcessor;
    let transaction_id = format!("TRX-{}", Uuid::new_v4());
    
    // Test process method
    let result = processor.process(1000.0, &transaction_id);
    assert!(result.is_ok());
    let payment_id = result.unwrap();
    assert!(payment_id.starts_with("CASH-"));
    
    // Test get_method
    assert_eq!(processor.get_method(), PaymentMethod::Cash);
}

#[test]
fn test_credit_card_payment_processor() {
    let processor = CreditCardPaymentProcessor;
    let transaction_id = format!("TRX-{}", Uuid::new_v4());
    
    // Test process method
    let result = processor.process(1000.0, &transaction_id);
    assert!(result.is_ok());
    let payment_id = result.unwrap();
    assert!(payment_id.starts_with("CC-"));
    
    // Test get_method
    assert_eq!(processor.get_method(), PaymentMethod::CreditCard);
}

#[test]
fn test_bank_transfer_payment_processor() {
    let processor = BankTransferPaymentProcessor;
    let transaction_id = format!("TRX-{}", Uuid::new_v4());
    
    // Test process method
    let result = processor.process(1000.0, &transaction_id);
    assert!(result.is_ok());
    let payment_id = result.unwrap();
    assert!(payment_id.starts_with("BANK-"));
    
    // Test get_method
    assert_eq!(processor.get_method(), PaymentMethod::BankTransfer);
}

#[test]
fn test_e_wallet_payment_processor() {
    let processor = EWalletPaymentProcessor;
    let transaction_id = format!("TRX-{}", Uuid::new_v4());
    
    // Test process method
    let result = processor.process(1000.0, &transaction_id);
    assert!(result.is_ok());
    let payment_id = result.unwrap();
    assert!(payment_id.starts_with("EWALLET-"));
    
    // Test get_method
    assert_eq!(processor.get_method(), PaymentMethod::EWallet);
}

#[test]
fn test_invalid_amount() {
    let processor = CashPaymentProcessor;
    let transaction_id = format!("TRX-{}", Uuid::new_v4());
    
    // Test with invalid amount
    let result = processor.process(-100.0, &transaction_id);
    assert!(result.is_err());
}