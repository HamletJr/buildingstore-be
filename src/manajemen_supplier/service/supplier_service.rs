use crate::manajemen_supplier::model::supplier::Supplier;
use async_trait::async_trait;
use mockall::automock;
use sqlx::{Any, Pool};

#[async_trait]
#[automock]
pub trait SupplierService: Send + Sync {
    async fn save_supplier(
        &self,
        db_pool: Pool<Any>,
        name: String,
        jenis_barang: String,
        jumlah_barang: i32,
        resi: String,
    ) -> Result<Supplier, String>;

    async fn update_supplier(
        &self,
        db_pool: Pool<Any>,
        id: String, 
        name: String,
        jenis_barang: String,
        jumlah_barang: i32,
        resi: String,
    ) -> Result<(), String>;

    // These remain the same
    async fn delete_supplier(&self, db_pool: Pool<Any>, id: &str) -> Result<(), String>;
    async fn get_supplier(&self, db_pool: Pool<Any>, id: &str) -> Result<Option<Supplier>, String>;
}