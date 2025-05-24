use rocket::fairing::AdHoc;
use rocket::routes;
use std::sync::Arc;

use crate::manajemen_pembayaran::repository::payment_repository_impl::PaymentRepositoryImpl;
use crate::manajemen_pembayaran::patterns::observer::PaymentSubject;
use crate::manajemen_pembayaran::service::payment_service_impl::PaymentServiceImpl;
use crate::manajemen_pembayaran::controller::payment_controller::{PaymentController, get_routes, get_catchers};

pub mod payment_controller;

pub fn route_stage() -> AdHoc {
    AdHoc::on_ignite("Payments Controller Routes", |rocket| async {
        // Setup repository
        let repository = Arc::new(PaymentRepositoryImpl::new());
        
        // Setup observer subject
        let subject = Arc::new(PaymentSubject::new());
        
        // Setup service
        let service = Arc::new(PaymentServiceImpl::new(repository, subject));
        
        // Setup controller
        let controller = PaymentController::new(service);
        
        // Return rocket instance with routes and catchers
        rocket
            .manage(controller)
            .mount("/api/payments", get_routes())
            .register("/api/payments", get_catchers())
    })
}