pub mod create;
pub mod read;
pub mod update;
pub mod delete;
pub mod filter;
pub mod dto;

use rocket::Route;

// Re-export untuk kemudahan akses
pub use dto::*;

// Fungsi untuk mendaftarkan semua routes
pub fn routes() -> Vec<Route> {
    let mut all_routes = Vec::new();
    
    // Tambahkan routes dari setiap modul
    all_routes.extend(create::routes());
    all_routes.extend(read::routes());
    all_routes.extend(update::routes());
    all_routes.extend(delete::routes());
    all_routes.extend(filter::routes());
    
    all_routes
}