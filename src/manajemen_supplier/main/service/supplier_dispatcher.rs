use std::sync::{Arc, Mutex};
use futures::future::join_all;
use async_trait::async_trait;
use crate::manajemen_supplier::main::model::supplier::Supplier;
use crate::manajemen_supplier::main::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::main::service::supplier_observer::SupplierObserver;

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
