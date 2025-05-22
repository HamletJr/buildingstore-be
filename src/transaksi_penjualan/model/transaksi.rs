///model/transaksi.rs
use chrono::{Utc, NaiveDateTime};
use rocket::serde::{Serialize, Deserialize};
use crate::transaksi_penjualan::enums::status_transaksi::StatusTransaksi;

/// Struct representing a transaction (Transaksi) in the system.
/// Contains fields for transaction details, customer info, and payment status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Transaksi {
    pub id: i32,
    pub id_pelanggan: i32,
    pub nama_pelanggan: String,
    pub tanggal_transaksi: NaiveDateTime,
    pub total_harga: f64,
    pub status: StatusTransaksi,
    pub catatan: Option<String>,
}

impl Transaksi {
    /// Creates a new instance of `Transaksi`. Automatically initializes the `id` to 0
    /// and sets the `tanggal_transaksi` to the current datetime.
    pub fn new(
        id_pelanggan: i32,
        nama_pelanggan: String,
        total_harga: f64,
        catatan: Option<String>,
    ) -> Self {
        Transaksi {
            id: 0,
            id_pelanggan,
            nama_pelanggan,
            tanggal_transaksi: Utc::now().naive_utc(),
            total_harga,
            status: StatusTransaksi::MasihDiproses,
            catatan,
        }
    }

    /// Updates the transaction status
    pub fn update_status(&mut self, status: StatusTransaksi) {
        self.status = status;
    }

    /// Updates the total price
    pub fn update_total_harga(&mut self, total_harga: f64) {
        self.total_harga = total_harga;
    }

    /// Checks if transaction can be modified (only if status is MasihDiproses)
    pub fn can_be_modified(&self) -> bool {
        self.status == StatusTransaksi::MasihDiproses
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_transaksi() {
        let transaksi = Transaksi::new(
            1,
            "Castorice".to_string(),
            150000.0,
            Some("Pembelian produk elektronik".to_string()),
        );

        assert_eq!(transaksi.id_pelanggan, 1);
        assert_eq!(transaksi.nama_pelanggan, "Castorice");
        assert_eq!(transaksi.total_harga, 150000.0);
        assert_eq!(transaksi.status, StatusTransaksi::MasihDiproses);
        assert!(transaksi.can_be_modified());
    }

    #[test]
    fn test_update_status() {
        let mut transaksi = Transaksi::new(
            1,
            "Tribbie".to_string(),
            200000.0,
            None,
        );

        transaksi.update_status(StatusTransaksi::Selesai);
        assert_eq!(transaksi.status, StatusTransaksi::Selesai);
        assert!(!transaksi.can_be_modified());
    }

    #[test]
    fn test_update_total_harga() {
        let mut transaksi = Transaksi::new(
            2,
            "Hyacine".to_string(),
            100000.0,
            None,
        );

        transaksi.update_total_harga(175000.0);
        assert_eq!(transaksi.total_harga, 175000.0);
    }
}