use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

use crate::main::model::payment::{Payment, PaymentMethod, Installment};
use crate::main::enums::payment_status::PaymentStatus;
use crate::main::repository::payment_repository::PaymentRepository;
use crate::main::patterns::observer::PaymentSubject;
use crate::main::patterns::factory::{PaymentProcessorFactory, PaymentStateFactory};
use crate::main::service::payment_service::PaymentService;

pub struct PaymentServiceImpl {
    repository: Arc<dyn PaymentRepository>,
    subject: Arc<PaymentSubject>,
}

impl PaymentServiceImpl {
    pub fn new(repository: Arc<dyn PaymentRepository>, subject: Arc<PaymentSubject>) -> Self {
        Self {
            repository,
            subject,
        }
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

        let processor = PaymentProcessorFactory::create(method.clone());
        let payment_id = processor.process(amount, &transaction_id)?;

        let payment = Payment {
            id: payment_id,
            transaction_id: transaction_id.clone(),
            amount,
            method: processor.get_method(),
            status: PaymentStatus::Paid, 
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        let saved_payment = self.repository.save(payment)?;
        
        self.subject.notify(&saved_payment);
        
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

        if let Some(amount) = additional_amount {
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
        }

        let updated_payment = self.repository.update(payment)?;
        
        self.subject.notify(&updated_payment);
        
        Ok(updated_payment)
    }

    fn delete_payment(&self, payment_id: String) -> Result<(), String> {
        let payment = self.repository.find_by_id(&payment_id)
            .ok_or_else(|| format!("Pembayaran dengan ID {} tidak ditemukan", payment_id))?;
            
        let state = PaymentStateFactory::create(&payment.status);
        if !state.can_delete() {
            return Err(format!("Tidak dapat menghapus pembayaran dengan status {}", state.get_name()));
        }
        
        self.repository.delete(&payment_id)
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
        let mut payment = self.repository
            .find_by_id(payment_id)
            .ok_or_else(|| format!("Pembayaran dengan ID {} tidak ditemukan", payment_id))?;

        let state = PaymentStateFactory::create(&payment.status);
        state.process_payment(&mut payment, amount)?;

        let updated_payment = self.repository.update(payment)?;
        self.subject.notify(&updated_payment);
        
        Ok(updated_payment)
    }
}