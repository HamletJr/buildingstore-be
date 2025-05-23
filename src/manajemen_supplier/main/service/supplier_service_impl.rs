use std::sync::Arc;

use async_trait::async_trait;
use crate::manajemen_supplier::main::model::supplier::Supplier;
use crate::manajemen_supplier::main::repository::supplier_repository::SupplierRepository;
use crate::manajemen_supplier::main::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::main::service::supplier_service::SupplierService;

pub struct SupplierServiceImpl {
    pub supplier_repo: Arc<dyn SupplierRepository>,
    pub dispatcher: Arc<dyn SupplierNotifier>,
}

impl SupplierServiceImpl {
    pub fn new(
        supplier_repo: Arc<dyn SupplierRepository>,
        dispatcher: Arc<dyn SupplierNotifier>,
    ) -> Self {
        SupplierServiceImpl {
            supplier_repo,
            dispatcher,
        }
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