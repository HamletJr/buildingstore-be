use uuid::Uuid;
use chrono::Utc;

use crate::main::model::payment::{Payment, PaymentStatus, Installment};

pub trait PaymentState: Send + Sync {
    fn process_payment(&self, payment: &mut Payment, amount: f64) -> Result<(), String>;
    fn can_delete(&self) -> bool;
    fn get_name(&self) -> String;
}

pub struct PaidState;
impl PaymentState for PaidState {
    fn process_payment(&self, _payment: &mut Payment, _amount: f64) -> Result<(), String> {
        Err("Pembayaran sudah lunas, tidak dapat menambahkan pembayaran lagi".to_string())
    }
    
    fn can_delete(&self) -> bool {
        false
    }

    fn get_name(&self) -> String {
        "LUNAS".to_string()
    }
}

pub struct InstallmentState;
impl PaymentState for InstallmentState {
    fn process_payment(&self, payment: &mut Payment, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Jumlah cicilan harus lebih dari 0".to_string());
        }
        
        let installment = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment.id.clone(),
            amount,
            payment_date: Utc::now(),
        };
        payment.installments.push(installment);

        let total_paid: f64 = payment.installments.iter().map(|i| i.amount).sum();
        if total_paid >= payment.amount {
            payment.status = PaymentStatus::Paid;
        }

        Ok(())
    }
    
    fn can_delete(&self) -> bool {
        true
    }

    fn get_name(&self) -> String {
        "CICILAN".to_string()
    }
}