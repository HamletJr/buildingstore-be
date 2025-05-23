use crate::manajemen_supplier::main::model::supplier::Supplier;
use::async_trait::async_trait;

#[async_trait]
pub trait SupplierNotifier: Send + Sync {
    async fn notify_supplier_saved(&self, supplier: &Supplier);
}