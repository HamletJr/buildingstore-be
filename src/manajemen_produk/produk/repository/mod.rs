pub mod helper;
pub mod create;
pub mod read;
pub mod update;
pub mod delete;

pub struct ProdukRepository;
pub use helper::{RepositoryError, get_next_id, PRODUCT_STORE, ID_COUNTER};
pub use create::tambah_produk;
pub use read::*;
pub use update::update_produk;
pub use delete::{hapus_produk, clear_all};