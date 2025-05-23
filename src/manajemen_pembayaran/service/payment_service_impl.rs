use std::collections::HashMap;
use std::sync::Arc;
use sqlx::{Any, Pool};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use async_trait::async_trait;

use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::repository::payment_repository::PaymentRepository;
use crate::manajemen_pembayaran::patterns::observer::PaymentSubject;
use crate::manajemen_pembayaran::service::payment_service::PaymentService;

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

#[async_trait]
impl PaymentService for PaymentServiceImpl {
    async fn create_payment(
        &self,
        db: Pool<Any>,
        transaction_id: String,
        amount: f64,
        method: PaymentMethod,
    ) -> Result<Payment, String> {
        // Ambil koneksi database
        let conn = match db.acquire().await {
            Ok(conn) => conn,
            Err(e) => return Err(format!("Database connection error: {}", e)),
        };
        
        // Buat payment baru
        let payment_id = format!("PMT-{}", Uuid::new_v4());
        let payment = Payment {
            id: payment_id,
            transaction_id,
            amount,
            method,
            status: PaymentStatus::Pending,
            payment_date: Utc::now(),
            installments: Vec::new(),
            due_date: None,
        };
        
        // Simpan ke database menggunakan repository
        match self.repository.save(conn, payment.clone()).await {
            Ok(saved_payment) => {
                // Notifikasi observer
                self.subject.notify(&saved_payment);
                Ok(saved_payment)
            },
            Err(e) => Err(format!("Failed to create payment: {}", e))
        }
    }

    async fn update_payment_status(
        &self,
        db: Pool<Any>,
        payment_id: String,
        new_status: PaymentStatus,
        additional_amount: Option<f64>,
    ) -> Result<Payment, String> {
        // Ambil koneksi database
        let conn = match db.acquire().await {
            Ok(conn) => conn,
            Err(e) => return Err(format!("Database connection error: {}", e)),
        };
        
        // Temukan payment yang ada
        let payment = match self.repository.find_by_id(conn, &payment_id).await {
            Ok(payment) => payment,
            Err(_) => return Err(format!("Payment with ID {} not found", payment_id)),
        };
        
        // Verifikasi bahwa perubahan status diizinkan
        if payment.status == PaymentStatus::Paid && new_status != PaymentStatus::Paid {
            return Err("Cannot change status of a paid payment".to_string());
        }
        
        // Dapatkan koneksi baru untuk update
        let conn = match db.acquire().await {
            Ok(conn) => conn,
            Err(e) => return Err(format!("Database connection error: {}", e)),
        };
        
        // Perbarui payment dan tambahkan installment jika diperlukan
        let mut updated_payment = payment.clone();
        updated_payment.status = new_status;
        
        if let Some(amount) = additional_amount {
            let installment = Installment {
                date: Utc::now(),
                amount,
            };
            updated_payment.installments.push(installment);
        }
        
        // Update di database
        match self.repository.update(conn, updated_payment.clone()).await {
            Ok(saved_payment) => {
                // Notifikasi observer
                self.subject.notify(&saved_payment);
                Ok(saved_payment)
            },
            Err(e) => Err(format!("Failed to update payment: {}", e))
        }
    }

    async fn delete_payment(&self, db: Pool<Any>, payment_id: String) -> Result<(), String> {
        // Ambil koneksi database
        let conn = match db.acquire().await {
            Ok(conn) => conn,
            Err(e) => return Err(format!("Database connection error: {}", e)),
        };
        
        // Temukan payment yang ada
        let payment = match self.repository.find_by_id(conn, &payment_id).await {
            Ok(payment) => payment,
            Err(_) => return Err(format!("Payment with ID {} not found", payment_id)),
        };
        
        // Verifikasi aturan bisnis (pembayaran lunas tidak boleh dihapus)
        if payment.status == PaymentStatus::Paid {
            return Err("Cannot delete a paid payment".to_string());
        }
        
        // Dapatkan koneksi baru untuk delete
        let conn = match db.acquire().await {
            Ok(conn) => conn,
            Err(e) => return Err(format!("Database connection error: {}", e)),
        };
        
        // Hapus dari database
        match self.repository.delete(conn, &payment_id).await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to delete payment: {}", e))
        }
    }

    async fn get_payment(&self, db: Pool<Any>, payment_id: &str) -> Result<Payment, String> {
        // Ambil koneksi database
        let conn = match db.acquire().await {
            Ok(conn) => conn,
            Err(e) => return Err(format!("Database connection error: {}", e)),
        };
        
        // Temukan payment berdasarkan ID
        match self.repository.find_by_id(conn, payment_id).await {
            Ok(payment) => Ok(payment),
            Err(_) => Err(format!("Payment with ID {} not found", payment_id))
        }
    }

    async fn get_payment_by_transaction(&self, db: Pool<Any>, transaction_id: &str) -> Result<Payment, String> {
        // Ambil koneksi database
        let conn = match db.acquire().await {
            Ok(conn) => conn,
            Err(e) => return Err(format!("Database connection error: {}", e)),
        };
        
        // Temukan payment berdasarkan transaction ID
        match self.repository.find_by_transaction_id(conn, transaction_id).await {
            Ok(payment) => Ok(payment),
            Err(_) => Err(format!("Payment with transaction ID {} not found", transaction_id))
        }
    }

    async fn get_all_payments(&self, db: Pool<Any>, filters: Option<HashMap<String, String>>) -> Result<Vec<Payment>, String> {
        // Ambil koneksi database
        let conn = match db.acquire().await {
            Ok(conn) => conn,
            Err(e) => return Err(format!("Database connection error: {}", e)),
        };
        
        // Dapatkan semua payment dengan filter opsional
        match self.repository.find_all(conn, filters).await {
            Ok(payments) => Ok(payments),
            Err(e) => Err(format!("Failed to get payments: {}", e))
        }
    }

    async fn add_installment(&self, db: Pool<Any>, payment_id: &str, amount: f64) -> Result<Payment, String> {
        // Cari payment untuk memastikan bahwa itu valid
        let payment = match self.get_payment(db.clone(), payment_id).await {
            Ok(payment) => payment,
            Err(e) => return Err(e),
        };
        
        // Verifikasi aturan bisnis (hanya pembayaran cicilan yang boleh ditambahkan installment)
        if payment.status != PaymentStatus::Installment {
            return Err("Can only add installments to a payment with Installment status".to_string());
        }
        
        // Tambahkan installment dengan update_payment_status
        self.update_payment_status(db, payment_id.to_string(), PaymentStatus::Installment, Some(amount)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::any::{AnyPoolOptions, install_default_drivers};
    use rocket::async_test;
    use crate::manajemen_pembayaran::repository::payment_repository_impl::PaymentRepositoryImpl;
    use crate::manajemen_pembayaran::patterns::observer::{PaymentObserver, Observer};
    
    struct TestObserver {
        payment_id: Option<String>,
    }
    
    impl TestObserver {
        fn new() -> Self {
            Self { payment_id: None }
        }
    }
    
    impl Observer for TestObserver {
        fn update(&mut self, payment: &Payment) {
            self.payment_id = Some(payment.id.clone());
        }
    }
    
    async fn setup() -> (PaymentServiceImpl, Pool<Any>) {
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
        
        let repository = Arc::new(PaymentRepositoryImpl::new());
        let subject = Arc::new(PaymentSubject::new());
        let service = PaymentServiceImpl::new(repository, subject);
        
        (service, db)
    }
    
    #[async_test]
    async fn test_create_payment() {
        let (service, db) = setup().await;
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let result = service.create_payment(
            db.clone(), 
            transaction_id.clone(), 
            1000.0, 
            PaymentMethod::Cash
        ).await;
        
        assert!(result.is_ok());
        let payment = result.unwrap();
        
        assert_eq!(payment.transaction_id, transaction_id);
        assert_eq!(payment.amount, 1000.0);
        assert_eq!(payment.method, PaymentMethod::Cash);
        assert_eq!(payment.status, PaymentStatus::Pending);
    }
    
    #[async_test]
    async fn test_update_payment_status() {
        let (service, db) = setup().await;
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        // Create a payment first
        let payment = service.create_payment(
            db.clone(), 
            transaction_id.clone(), 
            1000.0, 
            PaymentMethod::Cash
        ).await.unwrap();
        
        // Update status to Installment
        let result = service.update_payment_status(
            db.clone(),
            payment.id.clone(),
            PaymentStatus::Installment,
            Some(300.0)
        ).await;
        
        assert!(result.is_ok());
        let updated_payment = result.unwrap();
        
        assert_eq!(updated_payment.status, PaymentStatus::Installment);
        assert_eq!(updated_payment.installments.len(), 1);
        assert_eq!(updated_payment.installments[0].amount, 300.0);
        
        // Add another installment
        let result = service.add_installment(
            db.clone(),
            &payment.id,
            400.0
        ).await;
        
        assert!(result.is_ok());
        let payment_with_installment = result.unwrap();
        
        assert_eq!(payment_with_installment.installments.len(), 2);
        assert_eq!(payment_with_installment.installments[1].amount, 400.0);
    }
    
    #[async_test]
    async fn test_get_payment() {
        let (service, db) = setup().await;
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        // Create a payment first
        let payment = service.create_payment(
            db.clone(), 
            transaction_id.clone(), 
            1000.0, 
            PaymentMethod::Cash
        ).await.unwrap();
        
        // Get the payment by ID
        let result = service.get_payment(db.clone(), &payment.id).await;
        
        assert!(result.is_ok());
        let found_payment = result.unwrap();
        
        assert_eq!(found_payment.id, payment.id);
        assert_eq!(found_payment.transaction_id, transaction_id);
    }
    
    #[async_test]
    async fn test_get_all_payments() {
        let (service, db) = setup().await;
        
        // Create two payments
        let p1 = service.create_payment(
            db.clone(), 
            format!("TRX-1-{}", Uuid::new_v4()), 
            1000.0, 
            PaymentMethod::Cash
        ).await.unwrap();
        
        let p2 = service.create_payment(
            db.clone(), 
            format!("TRX-2-{}", Uuid::new_v4()), 
            2000.0, 
            PaymentMethod::CreditCard
        ).await.unwrap();
        
        // Set p1 to Paid status
        service.update_payment_status(
            db.clone(),
            p1.id.clone(),
            PaymentStatus::Paid,
            None
        ).await.unwrap();
        
        // Get all payments
        let result = service.get_all_payments(db.clone(), None).await;
        
        assert!(result.is_ok());
        let all_payments = result.unwrap();
        
        assert_eq!(all_payments.len(), 2);
        
        // Filter for only paid payments
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "LUNAS".to_string());
        
        let result = service.get_all_payments(db.clone(), Some(filters)).await;
        
        assert!(result.is_ok());
        let paid_payments = result.unwrap();
        
        assert_eq!(paid_payments.len(), 1);
        assert_eq!(paid_payments[0].status, PaymentStatus::Paid);
    }
    
    #[async_test]
    async fn test_delete_payment() {
        let (service, db) = setup().await;
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        // Create a payment first
        let payment = service.create_payment(
            db.clone(), 
            transaction_id.clone(), 
            1000.0, 
            PaymentMethod::Cash
        ).await.unwrap();
        
        // Delete the payment
        let result = service.delete_payment(db.clone(), payment.id.clone()).await;
        
        assert!(result.is_ok());
        
        // Attempt to get the deleted payment
        let result = service.get_payment(db.clone(), &payment.id).await;
        
        assert!(result.is_err());
    }
    
    #[async_test]
    async fn test_delete_paid_payment() {
        let (service, db) = setup().await;
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        // Create a payment first
        let payment = service.create_payment(
            db.clone(), 
            transaction_id.clone(), 
            1000.0, 
            PaymentMethod::Cash
        ).await.unwrap();
        
        // Set status to Paid
        service.update_payment_status(
            db.clone(),
            payment.id.clone(),
            PaymentStatus::Paid,
            None
        ).await.unwrap();
        
        // Try to delete the paid payment
        let result = service.delete_payment(db.clone(), payment.id.clone()).await;
        
        // Should fail because you can't delete paid payments
        assert!(result.is_err());
    }
    
    #[async_test]
    async fn test_observer_pattern() {
        let (service, db) = setup().await;
        
        let test_observer = Arc::new(std::sync::Mutex::new(TestObserver::new()));
        service.subject.attach(test_observer.clone());
        
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        // Create a payment
        let payment = service.create_payment(
            db.clone(), 
            transaction_id.clone(), 
            1000.0, 
            PaymentMethod::Cash
        ).await.unwrap();
        
        // Observer should be notified
        let observer = test_observer.lock().unwrap();
        assert_eq!(observer.payment_id, Some(payment.id));
    }
}
