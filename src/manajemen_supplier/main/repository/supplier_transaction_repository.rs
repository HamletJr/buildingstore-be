use async_trait::async_trait;
use crate::manajemen_supplier::main::model::supplier_transaction::SupplierTransaction;

#[async_trait]
pub trait SupplierTransactionRepository: Send + Sync {
    async fn save(&self, transaction: SupplierTransaction) -> Result<SupplierTransaction, String>;
    async fn find_by_id(&self, id: &str) -> Option<SupplierTransaction>;
    async fn find_by_supplier_id(&self, supplier_id: &str) -> Vec<SupplierTransaction>;
}
