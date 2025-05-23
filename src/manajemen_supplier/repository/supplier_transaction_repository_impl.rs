use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use crate::manajemen_supplier::model::supplier_transaction::SupplierTransaction;
use crate::manajemen_supplier::repository::supplier_transaction_repository::SupplierTransactionRepository;

#[derive(Clone, Default)]
pub struct SupplierTransactionRepositoryImpl {
    storage: Arc<Mutex<HashMap<String, SupplierTransaction>>>,
}

impl SupplierTransactionRepositoryImpl {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SupplierTransactionRepository for SupplierTransactionRepositoryImpl {
    async fn save(&self, transaction: SupplierTransaction) -> Result<SupplierTransaction, String> {
        self.storage
            .lock()
            .map_err(|_| "Failed to acquire lock".to_string())
            .map(|mut store| {
                store.insert(transaction.id.clone(), transaction.clone());
                transaction
            })
    }

    async fn find_by_id(&self, id: &str) -> Option<SupplierTransaction> {
        self.storage
            .lock()
            .ok()
            .and_then(|store| store.get(id).cloned())
    }

    async fn find_by_supplier_id(&self, supplier_id: &str) -> Vec<SupplierTransaction> {
        self.storage
            .lock()
            .map(|store| {
                store
                    .values()
                    .filter(|trx| trx.supplier_id == supplier_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use crate::manajemen_supplier::model::supplier::Supplier;

    fn create_supplier() -> Supplier {
        Supplier {
            id: format!("SUP-{}", Uuid::new_v4()),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now(),
        }
    }

    fn create_transaction(supplier: &Supplier) -> SupplierTransaction {
        SupplierTransaction {
            id: format!("TRX-{}", Uuid::new_v4()),
            supplier_id: supplier.id.clone(),
            supplier_name: supplier.name.clone(),
            jenis_barang: supplier.jenis_barang.clone(),
            jumlah_barang: supplier.jumlah_barang,
            pengiriman_info: supplier.resi.clone(),
            tanggal_transaksi: Utc::now(),
        }
    }

    #[tokio::test]
    async fn save_supplier_transaction() {
        let repository = SupplierTransactionRepositoryImpl::new();
        let supplier = create_supplier();
        let transaksi = create_transaction(&supplier);

        let result = repository.save(transaksi.clone()).await;
        assert!(result.is_ok());
        let saved = result.unwrap();
        assert_eq!(saved.supplier_id, supplier.id);
        assert_eq!(saved.supplier_name, supplier.name);
    }

    #[tokio::test]
    async fn test_find_supplier_transaction_by_id() {
        let repository = SupplierTransactionRepositoryImpl::new();
        let supplier = create_supplier();
        let transaksi = create_transaction(&supplier);

        repository.save(transaksi.clone()).await.unwrap();
        let found = repository.find_by_id(&transaksi.id).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, transaksi.id);
    }

    #[tokio::test]
    async fn test_find_supplier_transaction_by_supplier_id() {
        let repository = SupplierTransactionRepositoryImpl::new();
        let supplier = create_supplier();
        let transaksi = create_transaction(&supplier);

        repository.save(transaksi.clone()).await.unwrap();
        let results = repository.find_by_supplier_id(&supplier.id).await;
        assert!(!results.is_empty());
        assert_eq!(results[0].supplier_id, supplier.id);
    }
}
