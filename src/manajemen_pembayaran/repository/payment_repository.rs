use sqlx::any::AnyRow;
use sqlx::{Any, pool::PoolConnection};
use sqlx::Row;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};

pub struct PembayaranRepository;

impl PembayaranRepository {
    pub async fn create(mut db: PoolConnection<Any>, payment: &Payment) -> Result<Payment, sqlx::Error>{
        let result = sqlx::query("
            INSERT INTO payments (id, transaction_id, amount, payment_method, status, payment_date, due_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, transaction_id, amount, payment_method, status, payment_date, due_date
        ")
            .bind(&payment.id)
            .bind(&payment.transaction_id)
            .bind(payment.amount)
            .bind(payment.method.to_string())
            .bind(payment.status.to_string())
            .bind(payment.payment_date.naive_utc().to_string())
            .bind(payment.due_date.map(|d| d.naive_utc().to_string()))
            .fetch_one(&mut *db)
            .await?;

        let mut created_payment = Self::parse_row_to_payment(result)?;
        
        if !payment.installments.is_empty() {
            for installment in &payment.installments {
                Self::add_installment(&mut db, installment).await?;
            }
            
            created_payment = Self::load_payment_with_installments(&mut db, &created_payment.id).await?;
        }

        Ok(created_payment)
    }    pub async fn find_by_id(mut db: PoolConnection<Any>, id: &str) -> Result<Payment, sqlx::Error>{
        let payment_with_installments = Self::load_payment_with_installments(&mut db, id).await?;

        Ok(payment_with_installments)
    }    pub async fn find_all(mut db: PoolConnection<Any>, filters: Option<HashMap<String, String>>) -> Result<Vec<Payment>, sqlx::Error> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT id, transaction_id, amount, payment_method, status, payment_date, due_date FROM payments"
        );
        
        if let Some(filter_map) = &filters {
            let mut add_where = true;
            if let Some(status_str) = filter_map.get("status") {
                query_builder.push(" WHERE status = ");
                query_builder.push_bind(status_str);
                add_where = false;
            }            if let Some(method) = filter_map.get("method") {
                if add_where {
                    query_builder.push(" WHERE payment_method = ");
                    add_where = false;
                } else {
                    query_builder.push(" AND payment_method = ");
                }
                query_builder.push_bind(method.to_uppercase());
            }
            if let Some(transaction_id) = filter_map.get("transaction_id") {
                if add_where {
                    query_builder.push(" WHERE transaction_id = ");
                } else {
                    query_builder.push(" AND transaction_id = ");
                }
                query_builder.push_bind(transaction_id);
            }
        }
        
        let query = query_builder.build();
        let rows = query.fetch_all(&mut *db).await?;
        let mut payments = Vec::with_capacity(rows.len());
        for row in rows {
            let payment_id: String = row.get("id");
            let payment_with_installments = Self::load_payment_with_installments(&mut db, &payment_id).await?;
            payments.push(payment_with_installments);
        }
        
        Ok(payments)
    }   
    
    pub async fn update(mut db: PoolConnection<Any>, payment: &Payment) -> Result<Payment, sqlx::Error>{
        let payment_method_str = payment.method.to_string();
        let status_str = payment.status.to_string();
        
        let result = sqlx::query("
            UPDATE payments
            SET transaction_id = $1, amount = $2, payment_method = $3, status = $4, payment_date = $5, due_date = $6
            WHERE id = $7
            RETURNING id, transaction_id, amount, payment_method, status, payment_date, due_date
        ")
        .bind(&payment.transaction_id)
        .bind(payment.amount)
        .bind(&payment_method_str)
        .bind(&status_str)
        .bind(payment.payment_date.naive_utc().to_string())
        .bind(payment.due_date.map(|d| d.naive_utc().to_string()))
        .bind(&payment.id)
        .fetch_one(&mut *db)
        .await?;
        
        let updated_payment = Self::parse_row_to_payment(result)?;
        
        let payment_with_installments = Self::load_payment_with_installments(&mut db, &updated_payment.id).await?;

        Ok(payment_with_installments)
    }

    pub async fn update_payment_status(mut db: PoolConnection<Any>, payment_id: String, new_status: PaymentStatus, additional_amount: Option<f64>) -> Result<Payment, sqlx::Error> {
        let payment_result = sqlx::query("
            SELECT id, transaction_id, amount, payment_method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ")
        .bind(&payment_id)        .fetch_one(&mut *db)
        .await?;
        
        let mut payment = Self::parse_row_to_payment(payment_result)?;
        
        payment.status = new_status;
        
        if let Some(amount) = additional_amount {
            let installment = Installment {
                id: format!("INST-{}", Uuid::new_v4()),
                payment_id: payment_id.clone(),
                amount,
                payment_date: Utc::now(),
            };
            
            Self::add_installment(&mut db, &installment).await?;
        }
        
        Self::update(db, &payment).await
    }

    pub async fn delete(mut db: PoolConnection<Any>, id: &str) -> Result<(), sqlx::Error>{
        sqlx::query("DELETE FROM installments WHERE payment_id = $1")
            .bind(id)
            .execute(&mut *db)
            .await?;
        
        sqlx::query("DELETE FROM payments WHERE id = $1")
            .bind(id)
            .execute(&mut *db)
            .await?;
        
        Ok(())
    }
    
    pub async fn add_installment(db: &mut PoolConnection<Any>, installment: &Installment) -> Result<(), sqlx::Error> {
        sqlx::query("
            INSERT INTO installments (id, payment_id, amount, payment_date)
            VALUES ($1, $2, $3, $4)
        ")
        .bind(&installment.id)
        .bind(&installment.payment_id)
        .bind(installment.amount)
        .bind(installment.payment_date.naive_utc().to_string())
        .execute(&mut **db)
        .await?;
        
        Ok(())
    }
    pub async fn load_payment_with_installments(db: &mut PoolConnection<Any>, payment_id: &str) -> Result<Payment, sqlx::Error> {
        let payment_row = sqlx::query("
            SELECT id, transaction_id, amount, payment_method, status, payment_date, due_date
            FROM payments
            WHERE id = $1
        ")
        .bind(payment_id)
        .fetch_one(&mut **db)
        .await?;
        
        let mut payment = Self::parse_row_to_payment(payment_row)?;
        
        let installment_rows = sqlx::query("
            SELECT id, payment_id, amount, payment_date
            FROM installments
            WHERE payment_id = $1
            ORDER BY payment_date ASC
        ")
        .bind(payment_id)
        .fetch_all(&mut **db)
        .await?;
        
        let mut installments = Vec::with_capacity(installment_rows.len());
        for row in installment_rows {
            let installment = Self::parse_row_to_installment(row)?;
            installments.push(installment);
        }
        
        payment.installments = installments;
        
        Ok(payment)
    }
    
    fn parse_row_to_payment(row: AnyRow) -> Result<Payment, sqlx::Error> {
        let id: String = row.get("id");
        let transaction_id: String = row.get("transaction_id");
        let amount: f64 = row.get("amount");
        let payment_method_str: String = row.get("payment_method");
        let status_str: String = row.get("status");
        let payment_date_str: String = row.get("payment_date");
        let due_date_str: Option<String> = row.get("due_date");
        
        let payment_method = match payment_method_str.as_str() {
            "CASH" => PaymentMethod::Cash,
            "CREDIT_CARD" => PaymentMethod::CreditCard,
            "BANK_TRANSFER" => PaymentMethod::BankTransfer,
            "E_WALLET" => PaymentMethod::EWallet,
            _ => return Err(sqlx::Error::RowNotFound),
        };
        
        let payment_status = PaymentStatus::from_string(&status_str)
            .ok_or(sqlx::Error::RowNotFound)?;
        
        let payment_date = DateTime::parse_from_rfc3339(&payment_date_str)
            .map_err(|_| sqlx::Error::RowNotFound)?
            .with_timezone(&Utc);
            
        let due_date = due_date_str
            .map(|d| DateTime::parse_from_rfc3339(&d))
            .transpose()
            .map_err(|_| sqlx::Error::RowNotFound)?
            .map(|dt| dt.with_timezone(&Utc));
        
        Ok(Payment {
            id,
            transaction_id,
            amount,
            method: payment_method,
            status: payment_status,
            payment_date,
            installments: Vec::new(), 
            due_date,
        })
    }
    
    fn parse_row_to_installment(row: AnyRow) -> Result<Installment, sqlx::Error> {
        let id: String = row.get("id");
        let payment_id: String = row.get("payment_id");
        let amount: f64 = row.get("amount");
        let payment_date_str: String = row.get("payment_date");
        
        let payment_date = DateTime::parse_from_rfc3339(&payment_date_str)
            .map_err(|_| sqlx::Error::RowNotFound)?
            .with_timezone(&Utc);
        
        Ok(Installment {
            id,
            payment_id,
            amount,
            payment_date,
        })
    }
}


