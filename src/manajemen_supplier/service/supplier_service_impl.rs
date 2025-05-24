use std::sync::Arc;
use async_trait::async_trait;
use sqlx::{Any, Pool, Error as SqlxError};

use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::repository::supplier_repository::SupplierRepository;
use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::service::supplier_service::SupplierService;

pub struct SupplierServiceImpl {
    supplier_repo: Arc<dyn SupplierRepository>,
    dispatcher: Arc<dyn SupplierNotifier>,
    db_pool: Pool<Any>,
}

impl SupplierServiceImpl {
    pub fn new(
        supplier_repo: Arc<dyn SupplierRepository>,
        dispatcher: Arc<dyn SupplierNotifier>,
        db_pool: Pool<Any>,
    ) -> Self {
        Self { supplier_repo, dispatcher, db_pool }
    }
}

#[async_trait]
impl SupplierService for SupplierServiceImpl {
    async fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String> {
        let conn = self.db_pool.acquire().await
            .map_err(|e| format!("Service Error: Failed to acquire DB connection for save: {}", e))?;
        
        let saved_supplier_from_repo = self.supplier_repo.save(supplier, conn).await
            .map_err(|e| format!("Service Error: Repository save failed: {}", e))?;

        self.dispatcher.notify_supplier_saved(&saved_supplier_from_repo).await;
        Ok(saved_supplier_from_repo)
    }

    async fn update_supplier(&self, supplier: Supplier) -> Result<(), String> {
        let conn = self.db_pool.acquire().await
            .map_err(|e| format!("Service Error: Failed to acquire DB connection for update: {}", e))?;
        
        self.supplier_repo.update(supplier, conn).await
            .map_err(|e| match e {
                SqlxError::RowNotFound => "Service Error: Supplier not found for update.".to_string(),
                _ => format!("Service Error: Repository update failed: {}", e),
            })
    }

    async fn delete_supplier(&self, id: &str) -> Result<(), String> {
        let conn = self.db_pool.acquire().await
            .map_err(|e| format!("Service Error: Failed to acquire DB connection for delete: {}", e))?;

        self.supplier_repo.delete(id, conn).await
            .map_err(|e| match e {
                SqlxError::RowNotFound => "Service Error: Supplier not found for delete.".to_string(),
                _ => format!("Service Error: Repository delete failed: {}", e),
            })
    }

    async fn get_supplier(&self, id: &str) -> Option<Supplier> {
        let conn = match self.db_pool.acquire().await {
            Ok(c) => c,
            Err(e) => {
                // TODO: Replace with a robust logging framework
                eprintln!("[Service Error] Failed to acquire DB connection for get_supplier (ID: {}): {}", id, e);
                return None;
            }
        };
        match self.supplier_repo.find_by_id(id, conn).await {
            Ok(supplier) => Some(supplier),
            Err(SqlxError::RowNotFound) => None,
            Err(e) => {
                // TODO: Replace with a robust logging framework
                eprintln!("[Service Error] Repository error fetching supplier by ID '{}': {}", id, e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use chrono::Utc;
    use async_trait::async_trait;
    use mockall::predicate::*;
    use sqlx::any::{AnyPoolOptions, Any};

    use crate::manajemen_supplier::repository::supplier_repository::MockSupplierRepository;
    use crate::manajemen_supplier::model::supplier::Supplier;
    use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;

    struct MockNotifier {}

    #[async_trait]
    impl SupplierNotifier for MockNotifier {
        async fn notify_supplier_saved(&self, supplier: &Supplier) {
            // In a real test, you might add state to MockNotifier to assert this was called.
            println!("[Test MockNotifier] notify_supplier_saved called for supplier ID: {}", supplier.id);
        }
    }

    fn create_sample_supplier_for_service(id: &str, name: &str) -> Supplier {
        Supplier {
            id: id.to_string(),
            name: name.to_string(),
            jenis_barang: "Service Test Item".to_string(),
            jumlah_barang: 50,
            resi: "SRVRESI456".to_string(),
            updated_at: Utc::now(),
        }
    }

    async fn create_dummy_pool_for_service() -> Pool<Any> {
        sqlx::any::install_default_drivers();
        AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create dummy pool for service tests")
    }

    struct TestServiceSetup {
        service: SupplierServiceImpl,
        mock_repo: Arc<MockSupplierRepository>,
        mock_notifier: Arc<MockNotifier>,
    }

    async fn setup_service_for_test() -> TestServiceSetup {
        let repo_arc = Arc::new(MockSupplierRepository::new());
        let notifier_arc = Arc::new(MockNotifier{});
        let dummy_pool = create_dummy_pool_for_service().await;
        let service = SupplierServiceImpl::new(repo_arc.clone(), notifier_arc.clone(), dummy_pool);
        TestServiceSetup {
            service,
            mock_repo: repo_arc,
            mock_notifier: notifier_arc,
        }
    }

    #[tokio::test]
    async fn svc_save_supplier_success() {
        let mut setup = setup_service_for_test().await;
        let input_supplier = create_sample_supplier_for_service("SVC-SAVE-001", "Save Me");
        let returned_supplier_from_repo = input_supplier.clone();

        setup.mock_repo.expect_save()
            .with(eq(input_supplier.clone()), always())
            .times(1)
            .returning(move |_, _| Ok(returned_supplier_from_repo.clone()));

        let result = setup.service.save_supplier(input_supplier.clone()).await;
        
        assert!(result.is_ok());
        let service_result_supplier = result.unwrap();
        assert_eq!(service_result_supplier.id, "SVC-SAVE-001");
    }

    #[tokio::test]
    async fn svc_save_supplier_repo_error() {
        let mut setup = setup_service_for_test().await;
        let input_supplier = create_sample_supplier_for_service("SVC-SAVE-ERR", "Save Error");

        setup.mock_repo.expect_save()
            .with(eq(input_supplier.clone()), always())
            .times(1)
            .returning(|_, _| Err(SqlxError::Protocol("DB Save Error".to_string())));

        let result = setup.service.save_supplier(input_supplier.clone()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Service Error: Repository save failed: DB Save Error");
    }

    #[tokio::test]
    async fn svc_get_supplier_found() {
        let mut setup = setup_service_for_test().await;
        let supplier_id = "SVC-GET-FOUND";
        let expected_supplier = create_sample_supplier_for_service(supplier_id, "I Am Found");

        setup.mock_repo.expect_find_by_id()
            .with(eq(supplier_id), always())
            .times(1)
            .returning({
                let s = expected_supplier.clone();
                move |_, _| Ok(s.clone())
            });

        let result = setup.service.get_supplier(supplier_id).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, supplier_id);
    }

    #[tokio::test]
    async fn svc_get_supplier_repo_returns_not_found() {
        let mut setup = setup_service_for_test().await;
        let supplier_id = "SVC-GET-404";

        setup.mock_repo.expect_find_by_id()
            .with(eq(supplier_id), always())
            .times(1)
            .returning(|_, _| Err(SqlxError::RowNotFound));

        let result = setup.service.get_supplier(supplier_id).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn svc_update_supplier_success() {
        let mut setup = setup_service_for_test().await;
        let supplier_to_update = create_sample_supplier_for_service("SVC-UPD-001", "Updated Name");

        setup.mock_repo.expect_update()
            .with(eq(supplier_to_update.clone()), always())
            .times(1)
            .returning(|_, _| Ok(()));
        
        let result = setup.service.update_supplier(supplier_to_update).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn svc_update_supplier_repo_returns_not_found() {
        let mut setup = setup_service_for_test().await;
        let supplier_to_update = create_sample_supplier_for_service("SVC-UPD-404", "No Update");

        setup.mock_repo.expect_update()
            .with(eq(supplier_to_update.clone()), always())
            .times(1)
            .returning(|_, _| Err(SqlxError::RowNotFound));
            
        let result = setup.service.update_supplier(supplier_to_update).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Service Error: Supplier not found for update.");
    }

    #[tokio::test]
    async fn svc_delete_supplier_success() {
        let mut setup = setup_service_for_test().await;
        let supplier_id_to_delete = "SVC-DEL-001";

        setup.mock_repo.expect_delete()
            .with(eq(supplier_id_to_delete), always())
            .times(1)
            .returning(|_, _| Ok(()));
        
        let result = setup.service.delete_supplier(supplier_id_to_delete).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn svc_delete_supplier_repo_returns_not_found() {
        let mut setup = setup_service_for_test().await;
        let supplier_id_to_delete = "SVC-DEL-404";

        setup.mock_repo.expect_delete()
            .with(eq(supplier_id_to_delete), always())
            .times(1)
            .returning(|_, _| Err(SqlxError::RowNotFound));
            
        let result = setup.service.delete_supplier(supplier_id_to_delete).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Service Error: Supplier not found for delete.");
    }
}