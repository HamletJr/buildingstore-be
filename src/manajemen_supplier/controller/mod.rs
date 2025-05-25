use rocket::fairing::AdHoc;
use rocket::routes; 
use sqlx::{Any, Pool}; 
use std::sync::Arc;  
use crate::manajemen_supplier::repository::supplier_repository_impl::SupplierRepositoryImpl;
use crate::manajemen_supplier::service::supplier_dispatcher::SupplierDispatcher;
use crate::manajemen_supplier::service::supplier_service_impl::SupplierServiceImpl;
use crate::manajemen_supplier::repository::supplier_repository::SupplierRepository;
use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::service::supplier_service::SupplierService;
pub mod supplier_controller;

pub fn route_stage() -> AdHoc {
    AdHoc::on_ignite("Supplier Module: Manage Dependencies & Init Routes", |rocket| async {
        let _db_pool = match rocket.state::<Pool<Any>>() {
            Some(pool) => pool.clone(),
            None => {
                panic!("Critical dependency Pool<Any> not found for Supplier module. Halting startup.");
            }
        };
        let supplier_repository_instance: Arc<dyn SupplierRepository> =
            Arc::new(SupplierRepositoryImpl::new()); 

        let supplier_notifier_instance: Arc<dyn SupplierNotifier> =
            Arc::new(SupplierDispatcher::new()); 
        let supplier_service_instance: Arc<dyn SupplierService> =
            Arc::new(SupplierServiceImpl::new(
                supplier_repository_instance,      
                supplier_notifier_instance.clone(),
            ));
        rocket
            .manage(supplier_service_instance)
            .manage(supplier_notifier_instance)
            .mount("/api", routes![
                supplier_controller::save_supplier,
                supplier_controller::delete_supplier,
                supplier_controller::get_supplier,
                supplier_controller::update_supplier
            ])
    })
}