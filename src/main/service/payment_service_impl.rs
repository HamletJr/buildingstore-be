use std::collections::HashMap;
use std::sync::Arc;

use crate::main::model::payment::{Payment, PaymentMethod};
use crate::main::enums::payment_status::PaymentStatus;
use crate::main::repository::payment_repository::PaymentRepository;
use crate::main::patterns::observer::PaymentSubject;
use crate::main::patterns::command::{CreatePaymentCommand, UpdatePaymentStatusCommand, AddInstallmentCommand, DeletePaymentCommand};
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
        let command = CreatePaymentCommand::new(
            transaction_id,
            amount,
            method,
            self.repository.clone(),
            self.subject.clone(),
        );
        command.execute()
    }

    fn update_payment_status(
        &self,
        payment_id: String,
        new_status: PaymentStatus,
        additional_amount: Option<f64>,
    ) -> Result<Payment, String> {
        let command = UpdatePaymentStatusCommand::new(
            payment_id,
            new_status,
            additional_amount,
            self.repository.clone(),
            self.subject.clone(),
        );
        command.execute()
    }

    fn delete_payment(&self, payment_id: String) -> Result<(), String> {
        let command = DeletePaymentCommand::new(payment_id, self.repository.clone());
        command.execute()
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
        let command = AddInstallmentCommand::new(
            payment_id.to_string(),
            amount,
            self.repository.clone(),
            self.subject.clone(),
        );
        command.execute()
    }
}