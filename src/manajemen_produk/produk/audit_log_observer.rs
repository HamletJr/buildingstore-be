use crate::manajemen_produk::produk::events::ProdukObserver;
use crate::manajemen_produk::produk::model::Produk;
use rocket_db_pools::sqlx::{self, PgPool};

pub struct AuditLogObserver {
    pub db: PgPool,
}

impl ProdukObserver for AuditLogObserver {
    fn on_stock_changed(&self, produk: &Produk, old_stock: u32) {
        let _ = sqlx::query!(
            "INSERT INTO produk_audit_log (produk_id, field, old_value, new_value) VALUES ($1, $2, $3, $4)",
            produk.id.unwrap(),
            "stok",
            old_stock as i32,
            produk.stok as i32
        )
        .execute(&self.db);
    }
}