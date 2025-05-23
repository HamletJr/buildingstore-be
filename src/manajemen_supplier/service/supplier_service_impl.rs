use std::sync::Arc;
use async_trait::async_trait;

use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::repository::supplier_repository::SupplierRepository;
use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::service::supplier_service::SupplierService;

pub struct SupplierServiceImpl {
    supplier_repo: Arc<dyn SupplierRepository>,
    dispatcher: Arc<dyn SupplierNotifier>,
}

impl SupplierServiceImpl {
    pub fn new(
        supplier_repo: Arc<dyn SupplierRepository>,
        dispatcher: Arc<dyn SupplierNotifier>,
    ) -> Self {
        Self { supplier_repo, dispatcher }
    }
}

#[async_trait]
impl SupplierService for SupplierServiceImpl {
    async fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String> {
        let saved = self.supplier_repo.save(supplier.clone()).await?;
        self.dispatcher.notify_supplier_saved(&supplier).await;
        Ok(saved)
    }

    async fn update_supplier(&self, supplier: Supplier) -> Result<(), String> {
        self.supplier_repo.update(supplier).await
    }

    async fn delete_supplier(&self, id: &str) -> Result<(), String> {
        self.supplier_repo.delete(id).await
    }

    async fn get_supplier(&self, id: &str) -> Option<Supplier> {
        self.supplier_repo.find_by_id(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use chrono::Utc;
    use async_trait::async_trait;
    use crate::manajemen_supplier::repository::supplier_repository_impl::SupplierRepositoryImpl;

    struct MockNotifier;

    #[async_trait]
    impl SupplierNotifier for MockNotifier {
        async fn notify_supplier_saved(&self, _supplier: &Supplier) {}
    }

    fn create_sample_supplier() -> Supplier {
        Supplier {
            id: "SUP-123".to_string(),
            name: "PT. pt".to_string(),
            jenis_barang: "Ayam".to_string(),
            jumlah_barang: 10,
            resi: "2306206282".to_string(),
            updated_at: Utc::now(),
        }
    }

    fn setup_service() -> (Arc<SupplierRepositoryImpl>, SupplierServiceImpl) {
        let repo = Arc::new(SupplierRepositoryImpl::new());
        let notifier = Arc::new(MockNotifier);
        let service = SupplierServiceImpl::new(repo.clone(), notifier);
        (repo, service)
    }

    #[tokio::test]
    async fn test_save_supplier_success() {
        let (repo, service) = setup_service();
        let supplier = create_sample_supplier();
        let result = service.save_supplier(supplier.clone()).await;
        assert!(result.is_ok());
        let stored = repo.find_by_id("SUP-123").await.unwrap();
        assert_eq!(stored.name, supplier.name);
    }

    #[tokio::test]
    async fn test_update_supplier_success() {
        let (repo, service) = setup_service();
        let mut supplier = create_sample_supplier();
        service.save_supplier(supplier.clone()).await.unwrap();
        supplier.name = "Updated Supplier".to_string();
        let update_result = service.update_supplier(supplier.clone()).await;
        assert!(update_result.is_ok());
        let updated = repo.find_by_id("SUP-123").await.unwrap();
        assert_eq!(updated.name, "Updated Supplier");
    }

    #[tokio::test]
    async fn test_update_supplier_failure() {
        let (_, service) = setup_service();
        let supplier = create_sample_supplier();
        let result = service.update_supplier(supplier).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Supplier not found".to_string());
    }

    #[tokio::test]
    async fn test_delete_supplier_success() {
        let (repo, service) = setup_service();
        let supplier = create_sample_supplier();
        service.save_supplier(supplier.clone()).await.unwrap();
        let delete_result = service.delete_supplier(&supplier.id).await;
        assert!(delete_result.is_ok());
        let found = repo.find_by_id(&supplier.id).await;
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_supplier_failure() {
        let (_, service) = setup_service();
        let result = service.delete_supplier("UNKNOWN-ID").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Supplier not found".to_string());
    }

    #[tokio::test]
    async fn test_get_supplier_success() {
        let (_repo, service) = setup_service();
        let supplier = create_sample_supplier();
        service.save_supplier(supplier.clone()).await.unwrap();
        let result = service.get_supplier(&supplier.id).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, supplier.name);
    }

    #[tokio::test]
    async fn test_get_supplier_not_found() {
        let (_, service) = setup_service();
        let result = service.get_supplier("UNKNOWN").await;
        assert!(result.is_none());
    }
}
