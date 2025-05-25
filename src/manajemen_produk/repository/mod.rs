pub mod dto;
pub mod create;
pub mod read;
pub mod update;
pub mod delete;

pub struct ProdukRepository;
pub use dto::{RepositoryError, init_database, get_db_pool};
pub use create::{tambah_produk};
pub use read::*;
pub use update::{update_produk, update_stok, update_harga};
pub use delete::{hapus_produk, clear_all};