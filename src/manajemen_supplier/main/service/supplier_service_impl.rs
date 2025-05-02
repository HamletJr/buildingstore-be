pub struct SupplierServiceImpl {
}

impl SupplierServiceImpl {
}

impl SupplierService for SupplierServiceImpl {
    fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String> {
    }

    fn update_supplier(&self, supplier: Supplier) -> Result<(), String> {
    }

    fn delete_supplier(&self, id: &str) -> Result<(), String> {
    }

    fn get_supplier(&self, id: &str) -> Option<Supplier> {
    }
}