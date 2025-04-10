use chrono::{DateTime, Utc};
use crate::main::enums::payment_status::PaymentStatus;

// Enum untuk metode pembayaran
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentMethod {
    Cash,
    CreditCard,
    BankTransfer,
    EWallet,
}

// Struct untuk pembayaran
#[derive(Debug, Clone)]
pub struct Payment {
    pub id: String,
    pub transaction_id: String,
    pub amount: f64,
    pub method: PaymentMethod,
    pub status: PaymentStatus,
    pub payment_date: DateTime<Utc>,
    pub installments: Vec<Installment>,
    pub due_date: Option<DateTime<Utc>>,
}

// Struct untuk cicilan
#[derive(Debug, Clone)]
pub struct Installment {
    pub id: String,
    pub payment_id: String,
    pub amount: f64,
    pub payment_date: DateTime<Utc>,
}