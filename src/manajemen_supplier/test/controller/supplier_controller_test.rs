#[cfg(test)]
mod supplier_controller_test {
    use std::sync::Arc;
    use crate::manajemen_supplier::main::controller::supplier_controller::SupplierController;
    use crate::manajemen_supplier::main::repository::supplier_repository_impl::SupplierRepositoryImpl;
    use crate::manajemen_supplier::main::service::supplier_notifier::SupplierNotifier;
    use crate::manajemen_supplier::main::model::supplier::Supplier;
    use crate::manajemen_supplier::main::service::supplier_service_impl::SupplierServiceImpl;
    use chrono::Utc;

    struct MockNotifier;

    impl SupplierNotifier for MockNotifier {
        fn notify_supplier_saved(&self, _supplier: &Supplier) {
            // No-op for testing
        }
    }

    fn setup_controller() -> SupplierController {
        let repo = Arc::new(SupplierRepositoryImpl::new());
        let notifier = Arc::new(MockNotifier);
        let service = Arc::new(SupplierServiceImpl::new(repo.clone(), notifier));
        SupplierController::new(service)
    }

    fn sample_supplier(id: &str) -> Supplier {
        Supplier {
            id: id.to_string(),
            name: "PT. Pt".to_string(),
            jenis_barang: "Ayam".to_string(),
            jumlah_barang: 10,
            resi: "RESI123".to_string(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_save_supplier_success() {
        let controller = setup_controller();
        let supplier = sample_supplier("SUP1");

        let result = controller.save_supplier(supplier.clone());

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "SUP1");
    }

    #[test]
    fn test_get_supplier_existing() {
        let controller = setup_controller();
        let supplier = sample_supplier("SUP2");
        controller.save_supplier(supplier.clone()).unwrap();

        let fetched = controller.get_supplier("SUP2");

        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "PT. Pt");
    }

    #[test]
    fn test_get_supplier_non_existing() {
        let controller = setup_controller();
        let fetched = controller.get_supplier("NON_EXISTENT");
        assert!(fetched.is_none());
    }

    #[test]
    fn test_update_supplier_success() {
        let controller = setup_controller();
        let mut supplier = sample_supplier("SUP3");
        controller.save_supplier(supplier.clone()).unwrap();

        supplier.name = "Updated Supplier".to_string();
        let result = controller.update_supplier(supplier.clone());

        assert!(result.is_ok());

        let updated = controller.get_supplier("SUP3").unwrap();
        assert_eq!(updated.name, "Updated Supplier");
    }

    #[test]
    fn test_update_supplier_fail_not_found() {
        let controller = setup_controller();
        let supplier = sample_supplier("SUP-UNKNOWN");

        let result = controller.update_supplier(supplier);

        assert!(result.is_err());
    }

    #[test]
    fn test_delete_supplier_success() {
        let controller = setup_controller();
        let supplier = sample_supplier("SUP4");
        controller.save_supplier(supplier.clone()).unwrap();

        let result = controller.delete_supplier("SUP4");
        assert!(result.is_ok());

        let after_delete = controller.get_supplier("SUP4");
        assert!(after_delete.is_none());
    }

    #[test]
    fn test_delete_supplier_fail_not_found() {
        let controller = setup_controller();
        let result = controller.delete_supplier("SUP-UNKNOWN");
        assert!(result.is_err());
    }
}