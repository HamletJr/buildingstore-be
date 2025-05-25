use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{Any, Pool, Error as SqlxError};
use chrono::Utc; 
use uuid::Uuid; 

use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::repository::supplier_repository::SupplierRepository;
use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::service::supplier_service::SupplierService;

pub struct SupplierServiceImpl {
    supplier_repo: Arc<dyn SupplierRepository>,
    dispatcher: Arc<dyn SupplierNotifier>,
}

impl SupplierServiceImpl {
    pub fn new(
        supplier_repo: Arc<dyn SupplierRepository>,
        dispatcher: Arc<dyn SupplierNotifier>,
    ) -> Self {
        Self { supplier_repo, dispatcher }
    }
}

#[async_trait]
impl SupplierService for SupplierServiceImpl {
    async fn save_supplier(
        &self,
        db_pool: Pool<Any>,
        name: String,
        jenis_barang: String,
        jumlah_barang: i32,
        resi: String,
    ) -> Result<Supplier, String> {
        let conn = db_pool.acquire().await
            .map_err(|e| format!("Service: Failed to acquire DB connection: {}", e))?;
        
        let supplier_to_save = Supplier {
            id: Uuid::new_v4().to_string(), 
            name,
            jenis_barang,
            jumlah_barang,
            resi,
            updated_at: Utc::now().to_rfc3339(), 
        };
        
        let saved_supplier = self.supplier_repo.save(supplier_to_save, conn).await
            .map_err(|e| format!("Service: Repository save error: {}", e))?;

        self.dispatcher.notify_supplier_saved(&saved_supplier).await;
        Ok(saved_supplier)
    }

    async fn update_supplier(
        &self,
        db_pool: Pool<Any>,
        id: String,
        name: String,
        jenis_barang: String,
        jumlah_barang: i32,
        resi: String,
    ) -> Result<(), String> { 
        let conn = db_pool.acquire().await
            .map_err(|e| format!("Service: Failed to acquire DB connection: {}", e))?;
        let supplier_to_update = Supplier {
            id,
            name,
            jenis_barang,
            jumlah_barang,
            resi,
            updated_at: Utc::now().to_rfc3339(), 
        };
        
        self.supplier_repo.update(supplier_to_update, conn).await
            .map_err(|e| match e {
                SqlxError::RowNotFound => "Service: Supplier not found for update.".to_string(),
                _ => format!("Service: Repository update error: {}", e),
            })
    }

    async fn delete_supplier(&self, db_pool: Pool<Any>, id: &str) -> Result<(), String> {
        let conn = db_pool.acquire().await
            .map_err(|e| format!("Service: Failed to acquire DB connection: {}", e))?;

        self.supplier_repo.delete(id, conn).await
            .map_err(|e| match e {
                SqlxError::RowNotFound => "Service: Supplier not found for delete.".to_string(),
                _ => format!("Service: Repository delete error: {}", e),
            })
    }

    async fn get_supplier(&self, db_pool: Pool<Any>, id: &str) -> Result<Option<Supplier>, String> {
        let conn = match db_pool.acquire().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[Service Error] Failed to acquire DB connection for get_supplier (ID: {}): {}", id, e);
                return Err(format!("Service: Failed to acquire DB connection: {}", e));
            }
        };
        match self.supplier_repo.find_by_id(id, conn).await {
            Ok(s) => Ok(Some(s)), // Repository find_by_id returns Result<Supplier, Error>
            Err(SqlxError::RowNotFound) => Ok(None),
            Err(e) => {
                eprintln!("[Service Error] Repository error fetching supplier by ID '{}': {}", id, e);
                Err(format!("Service: Repository error: {}", e))
            }
        }
    }
}