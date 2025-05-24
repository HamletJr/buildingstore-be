use std::collections::HashMap;
use sqlx::{Any, pool::PoolConnection};
use crate::manajemen_pembayaran::model::payment::Payment;
use async_trait::async_trait;

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn save(&self, db: PoolConnection<Any>, payment: Payment) -> Result<Payment, sqlx::Error>;
    async fn find_by_id(&self, db: PoolConnection<Any>, id: &str) -> Result<Payment, sqlx::Error>;
    async fn find_by_transaction_id(&self, db: PoolConnection<Any>, transaction_id: &str) -> Result<Payment, sqlx::Error>;
    async fn find_all(&self, db: PoolConnection<Any>, filters: Option<HashMap<String, String>>) -> Result<Vec<Payment>, sqlx::Error>;
    async fn update(&self, db: PoolConnection<Any>, payment: Payment) -> Result<Payment, sqlx::Error>;
    async fn delete(&self, db: PoolConnection<Any>, id: &str) -> Result<(), sqlx::Error>;
}