// src/manajemen_supplier/service/supplier_service.rs
use crate::manajemen_supplier::model::supplier::Supplier;
use async_trait::async_trait;
use mockall::automock;
use sqlx::{Any, Pool};
// chrono::DateTime and Utc are no longer needed in the trait signature for these methods

#[async_trait]
#[automock]
pub trait SupplierService: Send + Sync {
    async fn save_supplier(
        &self,
        db_pool: Pool<Any>,
        // id: String, // Removed - Service will generate
        name: String,
        jenis_barang: String,
        jumlah_barang: i32,
        resi: String,
        // created_at: DateTime<Utc>, // Removed - Service will set
    ) -> Result<Supplier, String>;

    async fn update_supplier(
        &self,
        db_pool: Pool<Any>,
        id: String, // Still needed to identify the supplier to update
        name: String,
        jenis_barang: String,
        jumlah_barang: i32,
        resi: String,
        // updated_at: DateTime<Utc>, // Removed - Service will set
    ) -> Result<(), String>; // Consider returning Result<Supplier, String> to get the updated entity

    // These remain the same
    async fn delete_supplier(&self, db_pool: Pool<Any>, id: &str) -> Result<(), String>;
    async fn get_supplier(&self, db_pool: Pool<Any>, id: &str) -> Result<Option<Supplier>, String>;
}