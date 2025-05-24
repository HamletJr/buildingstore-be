use std::collections::HashMap;
use sqlx::{Any, Pool};
use async_trait::async_trait;

use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;

#[async_trait]
pub trait PaymentService: Send + Sync {
    async fn create_payment(&self, db: Pool<Any>, transaction_id: String, amount: f64, method: PaymentMethod) -> Result<Payment, String>;
    async fn update_payment_status(&self, db: Pool<Any>, payment_id: String, new_status: PaymentStatus, additional_amount: Option<f64>) -> Result<Payment, String>;
    async fn delete_payment(&self, db: Pool<Any>, payment_id: String) -> Result<(), String>;
    async fn get_payment(&self, db: Pool<Any>, payment_id: &str) -> Result<Payment, String>;
    async fn get_payment_by_transaction(&self, db: Pool<Any>, transaction_id: &str) -> Result<Payment, String>;
    async fn get_all_payments(&self, db: Pool<Any>, filters: Option<HashMap<String, String>>) -> Result<Vec<Payment>, String>;
    async fn add_installment(&self, db: Pool<Any>, payment_id: &str, amount: f64) -> Result<Payment, String>;
}