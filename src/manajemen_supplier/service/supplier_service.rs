use crate::manajemen_supplier::model::supplier::Supplier;
use::async_trait::async_trait;

#[async_trait]
pub trait SupplierService: Send + Sync {
    async fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String>;
    async fn update_supplier(&self, supplier: Supplier) -> Result<(), String>;
    async fn delete_supplier(&self, id: &str) -> Result<(), String>;
    async fn get_supplier(&self, id: &str) -> Option<Supplier>;
}
