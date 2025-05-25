pub mod create;
pub mod read;
pub mod update;
pub mod delete;
pub mod dto;

use rocket::{Route, routes};

// Re-export untuk kemudahan akses
pub use dto::*;

// Fungsi untuk mendaftarkan semua routes
pub fn routes() -> Vec<Route> {
    routes![
        create::tambah_produk,
        read::list_produk,
        read::detail_produk,
        update::update_produk,
        delete::hapus_produk
    ]
}