use sqlx::any::AnyRow;
use sqlx::{Any, pool::PoolConnection};
use sqlx::Row;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use async_trait::async_trait;
use sqlx::types::chrono::{DateTime as SqlxDateTime, Utc as SqlxUtc};

use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::repository::payment_repository::PaymentRepository;

pub struct PaymentRepositoryImpl;

impl PaymentRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
    
    fn parse_row_to_payment(row: AnyRow) -> Result<Payment, sqlx::Error> {
        let id: String = row.get("id");
        let transaction_id: String = row.get("transaction_id");
        let amount: f64 = row.get("amount");
        let method_str: String = row.get("payment_method");
        let status_str: String = row.get("status");
        let payment_date: DateTime<Utc> = row.get("payment_date");
        let due_date: Option<DateTime<Utc>> = row.try_get("due_date").unwrap_or(None);
        
        // Parse method and status from strings
        let method = match method_str.as_str() {
            "CASH" => PaymentMethod::Cash,
            "CREDIT_CARD" => PaymentMethod::CreditCard,
            "TRANSFER" => PaymentMethod::Transfer,
            _ => PaymentMethod::Cash // Default fallback
        };
        
        let status = PaymentStatus::from_string(&status_str).unwrap_or(PaymentStatus::Pending);
        
        // Create payment object
        let payment = Payment {
            id,
            transaction_id,
            amount,
            method,
            status,
            payment_date,
            installments: Vec::new(),
            due_date,
        };
        
        Ok(payment)
    }
}

#[async_trait]
impl PaymentRepository for PaymentRepositoryImpl {
    async fn save(&self, mut db: PoolConnection<Any>, payment: Payment) -> Result<Payment, sqlx::Error> {
        let payment_id = if payment.id.is_empty() {
            format!("PMT-{}", Uuid::new_v4())
        } else {
            payment.id
        };
        
        let payment_method_str = match payment.method {
            PaymentMethod::Cash => "CASH",
            PaymentMethod::CreditCard => "CREDIT_CARD",
            PaymentMethod::Transfer => "TRANSFER",
        };
        
        let status_str = payment.status.to_string();
        
        let result = sqlx::query("
            INSERT INTO payments (id, transaction_id, amount, payment_method, status, payment_date, due_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, transaction_id, amount, payment_method, status, payment_date, due_date
        ")
        .bind(&payment_id)
        .bind(&payment.transaction_id)
        .bind(payment.amount)
        .bind(payment_method_str)
        .bind(&status_str)
        .bind(payment.payment_date)
        .bind(payment.due_date)
        .fetch_one(&mut *db)
        .await?;
        
        Self::parse_row_to_payment(result)
    }

    async fn find_by_id(&self, mut db: PoolConnection<Any>, id: &str) -> Result<Payment, sqlx::Error> {
        let result = sqlx::query("
            SELECT id, transaction_id, amount, payment_method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ")
        .bind(id)
        .fetch_one(&mut *db)
        .await?;
        
        Self::parse_row_to_payment(result)
    }

    async fn find_by_transaction_id(&self, mut db: PoolConnection<Any>, transaction_id: &str) -> Result<Payment, sqlx::Error> {
        let result = sqlx::query("
            SELECT id, transaction_id, amount, payment_method, status, payment_date, due_date
            FROM payments
            WHERE transaction_id = $1
        ")
        .bind(transaction_id)
        .fetch_one(&mut *db)
        .await?;
        
        Self::parse_row_to_payment(result)
    }

    async fn find_all(&self, mut db: PoolConnection<Any>, filters: Option<std::collections::HashMap<String, String>>) -> Result<Vec<Payment>, sqlx::Error> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT id, transaction_id, amount, payment_method, status, payment_date, due_date FROM payments"
        );
        
        if let Some(filter_map) = &filters {
            let mut conditions = false;
            
            // Filter by status
            if let Some(status_str) = filter_map.get("status") {
                if !conditions {
                    query_builder.push(" WHERE ");
                    conditions = true;
                } else {
                    query_builder.push(" AND ");
                }
                
                query_builder.push("status = ");
                query_builder.push_bind(status_str);
            }
            
            // Filter by method
            if let Some(method) = filter_map.get("method") {
                if !conditions {
                    query_builder.push(" WHERE ");
                    conditions = true;
                } else {
                    query_builder.push(" AND ");
                }
                
                query_builder.push("payment_method = ");
                query_builder.push_bind(method.to_uppercase());
            }
        }
        
        let query = query_builder.build();
        let rows = query.fetch_all(&mut *db).await?;
        
        let mut payments = Vec::with_capacity(rows.len());
        for row in rows {
            let payment = Self::parse_row_to_payment(row)?;
            payments.push(payment);
        }
        
        Ok(payments)
    }

    async fn update(&self, mut db: PoolConnection<Any>, payment: Payment) -> Result<Payment, sqlx::Error> {
        let payment_method_str = match payment.method {
            PaymentMethod::Cash => "CASH",
            PaymentMethod::CreditCard => "CREDIT_CARD",
            PaymentMethod::Transfer => "TRANSFER",
        };
        
        let status_str = payment.status.to_string();
        
        let result = sqlx::query("
            UPDATE payments
            SET transaction_id = $1, amount = $2, payment_method = $3, status = $4, payment_date = $5, due_date = $6
            WHERE id = $7
            RETURNING id, transaction_id, amount, payment_method, status, payment_date, due_date
        ")
        .bind(&payment.transaction_id)
        .bind(payment.amount)
        .bind(payment_method_str)
        .bind(&status_str)
        .bind(payment.payment_date)
        .bind(payment.due_date)
        .bind(&payment.id)
        .fetch_one(&mut *db)
        .await?;
        
        Self::parse_row_to_payment(result)
    }

    async fn delete(&self, mut db: PoolConnection<Any>, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM payments WHERE id = $1")
            .bind(id)
            .execute(&mut *db)
            .await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use sqlx::any::{AnyPoolOptions, install_default_drivers};
    use rocket::async_test;
    use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};

    async fn setup() -> sqlx::Pool<Any> {
        install_default_drivers();
        let db = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        
        // Create payments table for testing
        sqlx::query("
            CREATE TABLE IF NOT EXISTS payments (
                id TEXT PRIMARY KEY,
                transaction_id TEXT NOT NULL,
                amount REAL NOT NULL,
                payment_method TEXT NOT NULL,
                status TEXT NOT NULL,
                payment_date TIMESTAMP NOT NULL,
                due_date TIMESTAMP
            )
        ")
        .execute(&db)
        .await
        .unwrap();
        
        db
    }

    #[async_test]
    async fn test_save_payment() {
        let db = setup().await;
        let repository = PaymentRepositoryImpl::new();
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
        
        let result = repository.save(db.acquire().await.unwrap(), payment.clone()).await;
        
        assert!(result.is_ok());
        let saved_payment = result.unwrap();
        assert_eq!(saved_payment.id, payment_id);
        assert_eq!(saved_payment.transaction_id, transaction_id);
        assert_eq!(saved_payment.amount, 1000.0);
    }

    #[async_test]
    async fn test_find_by_id() {
        let db = setup().await;
        let repository = PaymentRepositoryImpl::new();
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        
        let payment = Payment {
            id: payment_id.clone(),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        repository.save(db.acquire().await.unwrap(), payment.clone()).await.unwrap();
        
        let result = repository.find_by_id(db.acquire().await.unwrap(), &payment_id).await;
        
        assert!(result.is_ok());
        let found_payment = result.unwrap();
        assert_eq!(found_payment.id, payment_id);
    }

    #[async_test]
    async fn test_find_by_transaction_id() {
        let db = setup().await;
        let repository = PaymentRepositoryImpl::new();
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        repository.save(db.acquire().await.unwrap(), payment.clone()).await.unwrap();
        
        let result = repository.find_by_transaction_id(db.acquire().await.unwrap(), &transaction_id).await;
        
        assert!(result.is_ok());
        let found_payment = result.unwrap();
        assert_eq!(found_payment.transaction_id, transaction_id);
    }

    #[async_test]
    async fn test_update_payment() {
        let db = setup().await;
        let repository = PaymentRepositoryImpl::new();
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        
        let payment = Payment {
            id: payment_id.clone(),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        repository.save(db.acquire().await.unwrap(), payment.clone()).await.unwrap();
        
        let mut updated_payment = payment.clone();
        updated_payment.status = PaymentStatus::Paid;
        
        let result = repository.update(db.acquire().await.unwrap(), updated_payment).await;
        
        assert!(result.is_ok());
        let found = repository.find_by_id(db.acquire().await.unwrap(), &payment_id).await.unwrap();
        assert_eq!(found.status, PaymentStatus::Paid);
    }

    #[async_test]
    async fn test_delete_payment() {
        let db = setup().await;
        let repository = PaymentRepositoryImpl::new();
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        
        let payment = Payment {
            id: payment_id.clone(),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        repository.save(db.acquire().await.unwrap(), payment.clone()).await.unwrap();
        
        let result = repository.delete(db.acquire().await.unwrap(), &payment_id).await;
        
        assert!(result.is_ok());
        
        let find_result = repository.find_by_id(db.acquire().await.unwrap(), &payment_id).await;
        assert!(find_result.is_err());
    }

    #[async_test]
    async fn test_find_all_with_filters() {
        let db = setup().await;
        let repository = PaymentRepositoryImpl::new();
        
        let cash_payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Paid,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        let credit_payment = Payment {
            id: format!("PMT-{}", Uuid::new_v4()),
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 2000.0,
            method: PaymentMethod::CreditCard,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        repository.save(db.acquire().await.unwrap(), cash_payment.clone()).await.unwrap();
        repository.save(db.acquire().await.unwrap(), credit_payment.clone()).await.unwrap();
        
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "LUNAS".to_string());
        
        let status_results = repository.find_all(db.acquire().await.unwrap(), Some(filters)).await.unwrap();
        
        assert_eq!(status_results.len(), 1);
        assert_eq!(status_results[0].status, PaymentStatus::Paid);
        
        let all_results = repository.find_all(db.acquire().await.unwrap(), None).await.unwrap();
        
        assert_eq!(all_results.len(), 2);
    }
}