use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentMethod {
    Cash,
    CreditCard,
    BankTransfer,
    EWallet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installment {
    pub id: String,
    pub amount: f64,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub transaction_id: String,
    pub amount: f64,
    pub method: PaymentMethod,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub installments: Vec<Installment>,
    pub payment_proof: Option<String>, // Tambahkan field untuk bukti pembayaran
}

impl Payment {
    pub fn new(transaction_id: String, amount: f64, method: PaymentMethod) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            transaction_id,
            amount,
            method,
            status: PaymentStatus::Pending,
            created_at: now,
            updated_at: now,
            installments: Vec::new(),
            payment_proof: None,
        }
    }
    
    pub fn set_payment_proof(&mut self, proof: String) {
        self.payment_proof = Some(proof);
        self.updated_at = Utc::now();
    }

    pub fn add_installment(&mut self, amount: f64) -> Result<(), String> {
        if self.status != PaymentStatus::Installment {
            return Err("Pembayaran harus berstatus CICILAN untuk menambahkan cicilan".to_string());
        }
        
        let installment = Installment {
            id: Uuid::new_v4().to_string(),
            amount,
            date: Utc::now(),
        };
        
        self.installments.push(installment);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn get_paid_amount(&self) -> f64 {
        let installments_amount: f64 = self.installments.iter().map(|i| i.amount).sum();
        installments_amount
    }
    
    pub fn get_remaining_amount(&self) -> f64 {
        let paid_amount = self.get_paid_amount();
        self.amount - paid_amount
    }
    
    pub fn is_fully_paid(&self) -> bool {
        let paid_amount = self.get_paid_amount();
        paid_amount >= self.amount
    }
    
    pub fn update_status(&mut self, new_status: PaymentStatus, initial_amount: Option<f64>) -> Result<(), String> {
        match new_status {
            PaymentStatus::Paid => {
                self.status = PaymentStatus::Paid;
            },
            PaymentStatus::Installment => {
                self.status = PaymentStatus::Installment;
                
                if let Some(amount) = initial_amount {
                    let installment = Installment {
                        id: Uuid::new_v4().to_string(),
                        amount,
                        date: Utc::now(),
                    };
                    
                    self.installments.push(installment);
                }
            },
            PaymentStatus::Pending => {
                self.status = PaymentStatus::Pending;
            },
            PaymentStatus::Cancelled => {
                self.status = PaymentStatus::Cancelled;
            },
        }
        
        self.updated_at = Utc::now();
        
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_payment_status_from_string() {
        let paid_status = PaymentStatus::from_string("LUNAS");
        assert!(paid_status.is_some());
        assert_eq!(paid_status.unwrap(), PaymentStatus::Paid);

        let installment_status = PaymentStatus::from_string("CICILAN");
        assert!(installment_status.is_some());
        assert_eq!(installment_status.unwrap(), PaymentStatus::Installment);

        let lowercase_status = PaymentStatus::from_string("lunas");
        assert!(lowercase_status.is_some());
        assert_eq!(lowercase_status.unwrap(), PaymentStatus::Paid);

        let invalid_status = PaymentStatus::from_string("INVALID");
        assert!(invalid_status.is_none());
    }

    #[test]
    fn test_payment_status_to_string() {
        assert_eq!(PaymentStatus::Paid.to_string(), "LUNAS");
        assert_eq!(PaymentStatus::Installment.to_string(), "CICILAN");
    }


    #[test]
    fn test_create_payment() {
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
        
        assert_eq!(payment.id, payment_id);
        assert_eq!(payment.transaction_id, transaction_id);
        assert_eq!(payment.amount, 1000.0);
        assert_eq!(payment.method, PaymentMethod::Cash);
        assert_eq!(payment.status, PaymentStatus::Paid);
        assert!(payment.installments.is_empty());
    }

    #[test]
    fn test_add_installment() {
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        
        let mut payment = Payment {
            id: payment_id.clone(),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        let installment = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment_id.clone(),
            amount: 500.0,
            payment_date: Utc::now(),
        };
        
        payment.installments.push(installment);
        
        assert_eq!(payment.installments.len(), 1);
        assert_eq!(payment.installments[0].amount, 500.0);
    }
}