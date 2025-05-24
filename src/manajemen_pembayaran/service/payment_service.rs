use std::collections::HashMap;
use rocket::State;
use chrono::{Utc};
use uuid::Uuid;

use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::repository::payment_repository::PembayaranRepository;
use sqlx::{Any, Pool};

pub struct PaymentService;

#[derive(Debug)]
pub enum PaymentError {
    DatabaseError(String),
    NotFound(String),
    InvalidInput(String),
}

impl PaymentService {
    pub fn new() -> Self {
        PaymentService {}
    }
    
    pub async fn create_payment(&self, db: &State<Pool<Any>>, payment: Payment) -> Result<Payment, PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::create(conn, &payment).await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))
    }

    pub async fn get_payment_by_id(&self, db: &State<Pool<Any>>, id: &str) -> Result<Payment, PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::find_by_id(conn, id).await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", id)),
                _ => PaymentError::DatabaseError(e.to_string())
            })
    }

    pub async fn get_all_payments(&self, db: &State<Pool<Any>>, filters: Option<HashMap<String, String>>) -> Result<Vec<Payment>, PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::find_all(conn, filters).await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))
    }

    pub async fn update_payment(&self, db: &State<Pool<Any>>, payment: Payment) -> Result<Payment, PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::update(conn, &payment).await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", payment.id)),
                _ => PaymentError::DatabaseError(e.to_string())
            })
    }
    pub async fn update_payment_status(&self, db: &State<Pool<Any>>, payment_id: String, new_status: PaymentStatus, additional_amount: Option<f64>) -> Result<Payment, PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::update_payment_status(conn, payment_id.clone(), new_status, additional_amount).await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => PaymentError::NotFound(format!("Payment with id {} not found", payment_id)),
                _ => PaymentError::DatabaseError(e.to_string())
            })
    }
    
    pub async fn delete_payment(&self, db: &State<Pool<Any>>, payment_id: &str) -> Result<(), PaymentError> {
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        PembayaranRepository::delete(conn, payment_id).await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))
    }
    
    pub async fn add_installment(&self, db: &State<Pool<Any>>, payment_id: &str, amount: f64) -> Result<Payment, PaymentError> {
        let payment: Payment = self.get_payment_by_id(db, payment_id).await?;
        
        if payment.status != PaymentStatus::Installment {
            return Err(PaymentError::InvalidInput("Cannot add installment to a payment that is not in INSTALLMENT status".to_string()));
        }
        
        let installment = Installment {
            id: format!("INST-{}", Uuid::new_v4()),
            payment_id: payment_id.to_string(),
            amount,
            payment_date: Utc::now(),
        };
        
        let conn = db.acquire().await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        
        let mut updated_payment = payment.clone();
        updated_payment.installments.push(installment);
        
        PembayaranRepository::update(conn, &updated_payment).await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))
    }
    
    pub fn generate_payment_id(&self) -> String {
        format!("PMT-{}", Uuid::new_v4())
    }
    
    pub fn parse_payment_method(&self, method_str: &str) -> Result<PaymentMethod, PaymentError> {
        match method_str.to_uppercase().as_str() {
            "CASH" => Ok(PaymentMethod::Cash),
            "CREDIT_CARD" => Ok(PaymentMethod::CreditCard),
            "BANK_TRANSFER" => Ok(PaymentMethod::BankTransfer),
            "E_WALLET" => Ok(PaymentMethod::EWallet),
            _ => Err(PaymentError::InvalidInput(format!("Invalid payment method: {}", method_str))),
        }
    }
    
    pub fn parse_payment_status(&self, status_str: &str) -> Result<PaymentStatus, PaymentError> {
        PaymentStatus::from_string(status_str)
            .ok_or_else(|| PaymentError::InvalidInput(format!("Invalid payment status: {}", status_str)))
    }
}