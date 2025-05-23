use std::sync::{Arc, Mutex};
use futures::future::join_all;
use async_trait::async_trait;
use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::service::supplier_observer::SupplierObserver;

#[derive(Clone)]
pub struct SupplierDispatcher {
    observers: Arc<Mutex<Vec<Arc<dyn SupplierObserver + Send + Sync>>>>,
}

impl SupplierDispatcher {
    pub fn new() -> Self {
        Self {
            observers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register(&self, observer: Arc<dyn SupplierObserver + Send + Sync>) {
        self.observers.lock().unwrap().push(observer);
    }

pub async fn notify_all_supplier_saved(&self, supplier: &Supplier) {
        let observers_to_notify = self.observers.lock().unwrap().clone();
        let futures = observers_to_notify.into_iter()
            .map(|observer_arc| {
                async move {
                    observer_arc.on_supplier_saved(supplier).await;
                }
            });
        join_all(futures).await;
    }
}

#[async_trait]
impl SupplierNotifier for SupplierDispatcher {
    async fn notify_supplier_saved(&self, supplier: &Supplier) {
        self.notify_all_supplier_saved(supplier).await;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use chrono::Utc;
    use async_trait::async_trait;
    use crate::manajemen_supplier::model::supplier::Supplier;
    use crate::manajemen_supplier::service::supplier_dispatcher::SupplierDispatcher;
    use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;
    use crate::manajemen_supplier::service::supplier_observer::SupplierObserver;

    struct MockObserver {
        pub called: Arc<Mutex<bool>>,
        pub last_supplier_name: Arc<Mutex<Option<String>>>,
    }

    #[async_trait]
    impl SupplierObserver for MockObserver {
        async fn on_supplier_saved(&self, supplier: &Supplier) {
            let mut called_lock = self.called.lock().unwrap();
            let mut name_lock = self.last_supplier_name.lock().unwrap();
            *called_lock = true;
            *name_lock = Some(supplier.name.clone());
        }
    }

    fn sample_supplier() -> Supplier {
        Supplier {
            id: "SUP-001".to_string(),
            name: "Test Supplier".to_string(),
            jenis_barang: "Elektronik".to_string(),
            jumlah_barang: 100,
            resi: "RESI123".to_string(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test] 
    async fn test_register_and_notify_observer() {
        let dispatcher = SupplierDispatcher::new();

        let called_flag = Arc::new(Mutex::new(false));
        let last_name = Arc::new(Mutex::new(None::<String>));
        let observer = Arc::new(MockObserver {
            called: called_flag.clone(),
            last_supplier_name: last_name.clone(),
        });

        dispatcher.register(observer.clone());
        let supplier = sample_supplier();
        dispatcher.notify_supplier_saved(&supplier).await;

        assert_eq!(*called_flag.lock().unwrap(), true, "Observer was not called");
        assert_eq!(
            last_name.lock().unwrap().as_deref(),
            Some("Test Supplier"),
            "Supplier name was not correctly recorded by observer"
        );
    }


    #[tokio::test]
    async fn test_supplier_notifier_trait_works() {
        let dispatcher = SupplierDispatcher::new();
        let notifier: Arc<dyn SupplierNotifier> = Arc::new(dispatcher.clone()); 

        let called_flag = Arc::new(Mutex::new(false));
        let last_name = Arc::new(Mutex::new(None::<String>));

        let observer = Arc::new(MockObserver {
            called: called_flag.clone(),
            last_supplier_name: last_name.clone(),
        });

        dispatcher.register(observer);
        let supplier = sample_supplier();
        notifier.notify_supplier_saved(&supplier).await;

        assert_eq!(*called_flag.lock().unwrap(), true, "Observer was not called via trait");
        assert_eq!(
            last_name.lock().unwrap().as_deref(),
            Some("Test Supplier"),
            "Supplier name was not correctly recorded by observer via trait"
        );
    }
}