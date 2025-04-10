
use crate::main::model::payment::PaymentMethod;
use crate::main::patterns::strategy::{PaymentProcessor, CashPaymentProcessor, CreditCardPaymentProcessor, BankTransferPaymentProcessor, EWalletPaymentProcessor};

pub struct PaymentProcessorFactory;
impl PaymentProcessorFactory {
    pub fn create(method: PaymentMethod) -> Box<dyn PaymentProcessor> {
        match method {
            PaymentMethod::Cash => Box::new(CashPaymentProcessor),
            PaymentMethod::CreditCard => Box::new(CreditCardPaymentProcessor),
            PaymentMethod::BankTransfer => Box::new(BankTransferPaymentProcessor),
            PaymentMethod::EWallet => Box::new(EWalletPaymentProcessor),
        }
    }
}