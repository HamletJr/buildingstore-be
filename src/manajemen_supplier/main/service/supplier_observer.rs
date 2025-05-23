use crate::manajemen_supplier::main::model::supplier::Supplier;
use::async_trait::async_trait;

#[async_trait]
pub trait SupplierObserver: Send + Sync {
    async fn on_supplier_saved(&self, supplier: &Supplier);
}