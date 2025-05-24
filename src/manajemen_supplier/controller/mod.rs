use rocket::{fairing::AdHoc, routes};

pub mod supplier_controller;

pub fn route_stage() -> AdHoc {
    return AdHoc::on_ignite("Initializing Pelanggan controller routes...", |rocket| async {
        rocket
            .mount("/api", routes![supplier_controller::save_supplier, supplier_controller::delete_supplier,
            supplier_controller::get_supplier,supplier_controller::update_supplier])
    });
}