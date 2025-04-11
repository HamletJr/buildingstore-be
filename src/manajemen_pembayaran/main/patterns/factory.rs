use crate::main::model::payment::{PaymentMethod, PaymentStatus};
use crate::main::patterns::strategy::{PaymentProcessor, CashPaymentProcessor, CreditCardPaymentProcessor, BankTransferPaymentProcessor, EWalletPaymentProcessor};
use crate::main::patterns::state::{PaymentState, PaidState, InstallmentState};

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

pub struct PaymentStateFactory;
impl PaymentStateFactory {
    pub fn create(status: &PaymentStatus) -> Box<dyn PaymentState> {
        match status {
            PaymentStatus::Paid => Box::new(PaidState),
            PaymentStatus::Installment => Box::new(InstallmentState),
        }
    }
}