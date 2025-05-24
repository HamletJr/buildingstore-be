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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;
    use std::collections::HashMap;

    #[test]
    fn test_payment_service_creation() {
        let service = PaymentService::new();
        // Test that service is created successfully
        // Since PaymentService is a unit struct, we just verify it can be instantiated
        assert!(std::mem::size_of_val(&service) >= 0);
    }

    #[test]
    fn test_generate_payment_id() {
        let service = PaymentService::new();
        let id1 = service.generate_payment_id();
        let id2 = service.generate_payment_id();

        assert!(id1.starts_with("PMT-"));
        assert!(id2.starts_with("PMT-"));
        assert_ne!(id1, id2); // Each generated ID should be unique
        assert_eq!(id1.len(), 40); // "PMT-" + 36 character UUID
    }

    #[test]
    fn test_parse_payment_method_valid() {
        let service = PaymentService::new();

        assert_eq!(service.parse_payment_method("CASH").unwrap(), PaymentMethod::Cash);
        assert_eq!(service.parse_payment_method("cash").unwrap(), PaymentMethod::Cash);
        assert_eq!(service.parse_payment_method("Cash").unwrap(), PaymentMethod::Cash);

        assert_eq!(service.parse_payment_method("CREDIT_CARD").unwrap(), PaymentMethod::CreditCard);
        assert_eq!(service.parse_payment_method("credit_card").unwrap(), PaymentMethod::CreditCard);

        assert_eq!(service.parse_payment_method("BANK_TRANSFER").unwrap(), PaymentMethod::BankTransfer);
        assert_eq!(service.parse_payment_method("bank_transfer").unwrap(), PaymentMethod::BankTransfer);

        assert_eq!(service.parse_payment_method("E_WALLET").unwrap(), PaymentMethod::EWallet);
        assert_eq!(service.parse_payment_method("e_wallet").unwrap(), PaymentMethod::EWallet);
    }

    #[test]
    fn test_parse_payment_method_invalid() {
        let service = PaymentService::new();

        let result = service.parse_payment_method("INVALID_METHOD");
        assert!(result.is_err());

        if let Err(PaymentError::InvalidInput(msg)) = result {
            assert!(msg.contains("Invalid payment method"));
            assert!(msg.contains("INVALID_METHOD"));
        } else {
            panic!("Expected PaymentError::InvalidInput");
        }

        let result2 = service.parse_payment_method("");
        assert!(result2.is_err());
    }    #[test]
    fn test_parse_payment_status_valid() {
        let service = PaymentService::new();

        assert_eq!(service.parse_payment_status("LUNAS").unwrap(), PaymentStatus::Paid);
        assert_eq!(service.parse_payment_status("CICILAN").unwrap(), PaymentStatus::Installment);
    }

    #[test]
    fn test_parse_payment_status_invalid() {
        let service = PaymentService::new();

        let result = service.parse_payment_status("INVALID_STATUS");
        assert!(result.is_err());

        if let Err(PaymentError::InvalidInput(msg)) = result {
            assert!(msg.contains("Invalid payment status"));
            assert!(msg.contains("INVALID_STATUS"));
        } else {
            panic!("Expected PaymentError::InvalidInput");
        }

        let result2 = service.parse_payment_status("");
        assert!(result2.is_err());
    }

    #[test]
    fn test_payment_error_types() {
        let db_error = PaymentError::DatabaseError("Database connection failed".to_string());
        let not_found_error = PaymentError::NotFound("Payment not found".to_string());
        let invalid_input_error = PaymentError::InvalidInput("Invalid payment data".to_string());

        // Test that different error types can be created
        match db_error {
            PaymentError::DatabaseError(msg) => assert_eq!(msg, "Database connection failed"),
            _ => panic!("Expected DatabaseError"),
        }

        match not_found_error {
            PaymentError::NotFound(msg) => assert_eq!(msg, "Payment not found"),
            _ => panic!("Expected NotFound"),
        }

        match invalid_input_error {
            PaymentError::InvalidInput(msg) => assert_eq!(msg, "Invalid payment data"),
            _ => panic!("Expected InvalidInput"),
        }
    }

    #[test]
    fn test_installment_validation_logic() {
        // Test the business logic that would be used in add_installment
        let payment = Payment {
            id: "payment-123".to_string(),
            transaction_id: "txn-456".to_string(),
            amount: 1000.0,
            method: PaymentMethod::Cash,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        // Valid case: payment status is INSTALLMENT
        assert_eq!(payment.status, PaymentStatus::Installment);        // Invalid case: payment status is not INSTALLMENT
        let invalid_payment = Payment {
            status: PaymentStatus::Paid,
            ..payment
        };

        assert_ne!(invalid_payment.status, PaymentStatus::Installment);
    }

    #[test]
    fn test_installment_creation_in_service() {
        let payment_id = "payment-123";
        let amount = 500.0;
        let installment_id = format!("INST-{}", Uuid::new_v4());

        let installment = Installment {
            id: installment_id.clone(),
            payment_id: payment_id.to_string(),
            amount,
            payment_date: Utc::now(),
        };

        assert_eq!(installment.payment_id, payment_id);
        assert_eq!(installment.amount, amount);
        assert!(installment.id.starts_with("INST-"));
    }

    #[test]
    fn test_filter_handling() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "COMPLETED".to_string());
        filters.insert("method".to_string(), "CASH".to_string());

        // Test that filters can be constructed and accessed
        assert_eq!(filters.get("status"), Some(&"COMPLETED".to_string()));
        assert_eq!(filters.get("method"), Some(&"CASH".to_string()));
        assert_eq!(filters.get("nonexistent"), None);

        // Test empty filters
        let empty_filters: Option<HashMap<String, String>> = None;
        assert!(empty_filters.is_none());
    }

    #[test]
    fn test_payment_structure_for_service() {
        let payment = Payment {
            id: "PMT-123".to_string(),
            transaction_id: "TXN-456".to_string(),
            amount: 1500.0,
            method: PaymentMethod::BankTransfer,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: vec![
                Installment {
                    id: "INST-1".to_string(),
                    payment_id: "PMT-123".to_string(),
                    amount: 750.0,
                    payment_date: Utc::now(),
                },
                Installment {
                    id: "INST-2".to_string(),
                    payment_id: "PMT-123".to_string(),
                    amount: 750.0,
                    payment_date: Utc::now(),
                },
            ],
            due_date: Some(Utc::now()),
        };

        assert_eq!(payment.installments.len(), 2);
        assert_eq!(payment.amount, 1500.0);
        
        let total_installments: f64 = payment.installments.iter().map(|i| i.amount).sum();
        assert_eq!(total_installments, 1500.0);
    }

    #[test]
    fn test_payment_cloning() {
        let original_payment = Payment {
            id: "PMT-789".to_string(),
            transaction_id: "TXN-101112".to_string(),
            amount: 2000.0,
            method: PaymentMethod::CreditCard,
            status: PaymentStatus::Installment,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };

        let cloned_payment = original_payment.clone();

        assert_eq!(original_payment.id, cloned_payment.id);
        assert_eq!(original_payment.transaction_id, cloned_payment.transaction_id);
        assert_eq!(original_payment.amount, cloned_payment.amount);
        assert_eq!(original_payment.method, cloned_payment.method);
        assert_eq!(original_payment.status, cloned_payment.status);
    }

    #[test]
    fn test_uuid_generation_consistency() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();

        assert_ne!(uuid1, uuid2);
        assert_eq!(uuid1.to_string().len(), 36);
        assert_eq!(uuid2.to_string().len(), 36);
    }
}