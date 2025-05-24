use std::sync::Arc;
use async_trait::async_trait;

use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::patterns::factory::SupplierTransactionFactory;
use crate::manajemen_supplier::service::supplier_observer::SupplierObserver;
use crate::manajemen_supplier::repository::supplier_transaction_repository::SupplierTransactionRepository;

#[derive(Clone)]
pub struct SupplierTransactionLogger {
    trx_repo: Arc<dyn SupplierTransactionRepository + Send + Sync>,
}

impl SupplierTransactionLogger {
    pub fn new(trx_repo: Arc<dyn SupplierTransactionRepository + Send + Sync>) -> Self {
        Self { trx_repo }
    }
}

#[async_trait]
impl SupplierObserver for SupplierTransactionLogger {
    async fn on_supplier_saved(&self, supplier: &Supplier) {
        let trx = SupplierTransactionFactory::create_from_supplier(supplier);
        if let Err(err) = self.trx_repo.save(trx).await {
            eprintln!("Failed to log transaction: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use chrono::Utc;
    use async_trait::async_trait;

    use crate::manajemen_supplier::model::supplier::Supplier;
    use crate::manajemen_supplier::model::supplier_transaction::SupplierTransaction;
    use crate::manajemen_supplier::repository::supplier_transaction_repository::SupplierTransactionRepository;
    use crate::manajemen_supplier::service::supplier_observer::SupplierObserver;
    use super::SupplierTransactionLogger;

    struct MockSupplierTransactionRepository {
        saved_transaction: Arc<Mutex<Option<SupplierTransaction>>>,
    }

    #[async_trait]
    impl SupplierTransactionRepository for MockSupplierTransactionRepository {
        async fn save(&self, trx: SupplierTransaction) -> Result<SupplierTransaction, String> {
            *self.saved_transaction.lock().unwrap() = Some(trx.clone());
            Ok(trx)
        }

        async fn find_by_id(&self, _id: &str) -> Option<SupplierTransaction> {
            None
        }

        async fn find_by_supplier_id(&self, _supplier_id: &str) -> Vec<SupplierTransaction> {
            vec![]
        }
    }

    fn sample_supplier() -> Supplier {
        Supplier {
            id: "SUP-123".to_string(),
            name: "PT. Pt".to_string(),
            jenis_barang: "Ayam".to_string(),
            jumlah_barang: 10,
            resi: "RESI123".to_string(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_on_supplier_saved_logs_transaction() {
        let saved_trx = Arc::new(Mutex::new(None));
        let mock_repo = Arc::new(MockSupplierTransactionRepository {
            saved_transaction: Arc::clone(&saved_trx),
        });
        let logger = SupplierTransactionLogger::new(mock_repo);
        let supplier = sample_supplier();

        logger.on_supplier_saved(&supplier).await;

        let logged_trx = saved_trx.lock().unwrap();
        let logged_trx = logged_trx.as_ref().expect("Transaction was not logged");

        assert_eq!(logged_trx.supplier_id, "SUP-123");
        assert_eq!(logged_trx.supplier_name, "PT. Pt");
        assert_eq!(logged_trx.jenis_barang, "Ayam");
        assert_eq!(logged_trx.jumlah_barang, 10);
        assert_eq!(logged_trx.pengiriman_info, "RESI123");
    }
}
