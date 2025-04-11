use rocket::fairing::AdHoc;

pub mod pelanggan;

pub fn route_stage() -> AdHoc {
    return AdHoc::on_ignite("Initializing controller routes...", |rocket| async {
        rocket
            .mount("/api/pelanggan", routes![])
    });
}
