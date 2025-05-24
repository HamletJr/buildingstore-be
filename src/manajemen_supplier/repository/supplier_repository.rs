use async_trait::async_trait;
use crate::manajemen_supplier::model::supplier::Supplier;

#[async_trait]
pub trait SupplierRepository: Send + Sync {
    async fn save(&self, supplier: Supplier) -> Result<Supplier, String>;
    async fn find_by_id(&self, id: &str) -> Option<Supplier>;
    async fn update(&self, supplier: Supplier) -> Result<(), String>;
    async fn delete(&self, id: &str) -> Result<(), String>;
}
