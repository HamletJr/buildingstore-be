///model/detail_transaksi.rs
use rocket::serde::{Serialize, Deserialize};

/// Struct representing transaction details (Detail Transaksi) in the system.
/// Contains information about products in a transaction with reference to product ID only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DetailTransaksi {
    pub id: i32,
    pub id_transaksi: i32,
    pub id_produk: i32,
    pub harga_satuan: f64,  // Price at the time of transaction
    pub jumlah: u32,
    pub subtotal: f64,
}

impl DetailTransaksi {
    /// Creates a new instance of `DetailTransaksi`. Automatically calculates subtotal.
    /// Note: harga_satuan should be fetched from the current product price when creating the transaction.
    pub fn new(
        id_transaksi: i32,
        id_produk: i32,
        harga_satuan: f64,
        jumlah: u32,
    ) -> Self {
        let subtotal = harga_satuan * jumlah as f64;
        
        DetailTransaksi {
            id: 0,
            id_transaksi,
            id_produk,
            harga_satuan,
            jumlah,
            subtotal,
        }
    }

    /// Updates the quantity and recalculates subtotal
    pub fn update_jumlah(&mut self, jumlah: u32) {
        self.jumlah = jumlah;
        self.subtotal = self.harga_satuan * jumlah as f64;
    }

    /// Updates the unit price and recalculates subtotal
    /// This should rarely be used after transaction creation
    pub fn update_harga_satuan(&mut self, harga_satuan: f64) {
        self.harga_satuan = harga_satuan;
        self.subtotal = harga_satuan * self.jumlah as f64;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_detail_transaksi() {
        let detail = DetailTransaksi::new(
            1,
            101,
            15000000.0,
            2,
        );

        assert_eq!(detail.id_transaksi, 1);
        assert_eq!(detail.id_produk, 101);
        assert_eq!(detail.harga_satuan, 15000000.0);
        assert_eq!(detail.jumlah, 2);
        assert_eq!(detail.subtotal, 30000000.0);
    }

    #[test]
    fn test_update_jumlah() {
        let mut detail = DetailTransaksi::new(
            1,
            102,
            250000.0,
            1,
        );

        detail.update_jumlah(3);
        assert_eq!(detail.jumlah, 3);
        assert_eq!(detail.subtotal, 750000.0);
    }

    #[test]
    fn test_update_harga_satuan() {
        let mut detail = DetailTransaksi::new(
            1,
            103,
            500000.0,
            2,
        );

        detail.update_harga_satuan(600000.0);
        assert_eq!(detail.harga_satuan, 600000.0);
        assert_eq!(detail.subtotal, 1200000.0);
    }
}