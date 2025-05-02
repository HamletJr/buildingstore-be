use std::sync::Arc;

use crate::manajemen_supplier::main::model::supplier::Supplier;
use crate::manajemen_supplier::main::service::supplier_service::SupplierService;

pub struct SupplierController {
    supplier_service: Arc<dyn SupplierService>,
}

impl SupplierController {
    pub fn new(supplier_service: Arc<dyn SupplierService>) -> Self {
        Self { supplier_service }
    }

    pub fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String> {
        self.supplier_service.save_supplier(supplier)
    }

    pub fn update_supplier(&self, supplier: Supplier) -> Result<(), String> {
        self.supplier_service.update_supplier(supplier)
    }

    pub fn delete_supplier(&self, id: &str) -> Result<(), String> {
        self.supplier_service.delete_supplier(id)
    }

    pub fn get_supplier(&self, id: &str) -> Option<Supplier> {
        self.supplier_service.get_supplier(id)
    }
}