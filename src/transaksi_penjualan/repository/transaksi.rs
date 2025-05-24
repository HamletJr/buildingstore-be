// transaksi/repository/transaksi.rs

use sqlx::any::AnyRow;
use sqlx::{Any, pool::PoolConnection};
use sqlx::Row;
use chrono::NaiveDateTime;

use crate::transaksi_penjualan::model::transaksi::Transaksi;
use crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi;
use crate::transaksi_penjualan::enums::status_transaksi::StatusTransaksi;

pub struct TransaksiRepository;

impl TransaksiRepository {
    pub async fn create_transaksi(mut db: PoolConnection<Any>, transaksi: &Transaksi) -> Result<Transaksi, sqlx::Error> {
        let result = sqlx::query("
                INSERT INTO transaksi (id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan
            ")
            .bind(transaksi.id_pelanggan)
            .bind(&transaksi.nama_pelanggan)
            .bind(transaksi.tanggal_transaksi.to_string())
            .bind(transaksi.total_harga)
            .bind(transaksi.status.to_string())
            .bind(&transaksi.catatan)
            .fetch_one(&mut *db)
            .await?;
        
        let transaksi = Self::parse_row_to_transaksi(result);
        Ok(transaksi)
    }

    pub async fn get_transaksi_by_id(mut db: PoolConnection<Any>, id: i32) -> Result<Transaksi, sqlx::Error> {
        let result = sqlx::query("
                SELECT id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan
                FROM transaksi
                WHERE id = $1
            ")
            .bind(id)
            .fetch_one(&mut *db)
            .await?;
        
        let transaksi = Self::parse_row_to_transaksi(result);
        Ok(transaksi)
    }

    pub async fn update_transaksi(mut db: PoolConnection<Any>, transaksi: &Transaksi) -> Result<Transaksi, sqlx::Error> {
        let result = sqlx::query("
                UPDATE transaksi
                SET id_pelanggan = $1, nama_pelanggan = $2, tanggal_transaksi = $3, 
                    total_harga = $4, status = $5, catatan = $6
                WHERE id = $7
                RETURNING id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan
            ")
            .bind(transaksi.id_pelanggan)
            .bind(&transaksi.nama_pelanggan)
            .bind(transaksi.tanggal_transaksi.to_string())
            .bind(transaksi.total_harga)
            .bind(transaksi.status.to_string())
            .bind(&transaksi.catatan)
            .bind(transaksi.id)
            .fetch_one(&mut *db)
            .await?;
        
        let transaksi = Self::parse_row_to_transaksi(result);
        Ok(transaksi)
    }

    pub async fn delete_transaksi(mut db: PoolConnection<Any>, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM transaksi WHERE id = $1")
            .bind(id)
            .execute(&mut *db)
            .await?;
        
        Ok(())
    }

    pub async fn get_all_transaksi(mut db: PoolConnection<Any>) -> Result<Vec<Transaksi>, sqlx::Error> {
        let rows = sqlx::query("
                SELECT id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan
                FROM transaksi
                ORDER BY tanggal_transaksi DESC
            ")
            .fetch_all(&mut *db)
            .await?;
        
        let mut transaksi_list = Vec::new();
        for row in rows {
            let transaksi = Self::parse_row_to_transaksi(row);
            transaksi_list.push(transaksi);
        }
        
        Ok(transaksi_list)
    }

    pub async fn get_transaksi_by_pelanggan(mut db: PoolConnection<Any>, id_pelanggan: i32) -> Result<Vec<Transaksi>, sqlx::Error> {
        let rows = sqlx::query("
                SELECT id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan
                FROM transaksi
                WHERE id_pelanggan = $1
                ORDER BY tanggal_transaksi DESC
            ")
            .bind(id_pelanggan)
            .fetch_all(&mut *db)
            .await?;
        
        let mut transaksi_list = Vec::new();
        for row in rows {
            let transaksi = Self::parse_row_to_transaksi(row);
            transaksi_list.push(transaksi);
        }
        
        Ok(transaksi_list)
    }

    pub async fn get_transaksi_by_status(mut db: PoolConnection<Any>, status: &StatusTransaksi) -> Result<Vec<Transaksi>, sqlx::Error> {
        let rows = sqlx::query("
                SELECT id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan
                FROM transaksi
                WHERE status = $1
                ORDER BY tanggal_transaksi DESC
            ")
            .bind(status.to_string())
            .fetch_all(&mut *db)
            .await?;
        
        let mut transaksi_list = Vec::new();
        for row in rows {
            let transaksi = Self::parse_row_to_transaksi(row);
            transaksi_list.push(transaksi);
        }
        
        Ok(transaksi_list)
    }

    pub async fn create_detail_transaksi(mut db: PoolConnection<Any>, detail: &DetailTransaksi) -> Result<DetailTransaksi, sqlx::Error> {
        let result = sqlx::query("
                INSERT INTO detail_transaksi (id_transaksi, id_produk, harga_satuan, jumlah, subtotal)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, id_transaksi, id_produk, harga_satuan, jumlah, subtotal
            ")
            .bind(detail.id_transaksi)
            .bind(detail.id_produk)
            .bind(detail.harga_satuan)
            .bind(detail.jumlah as i32)
            .bind(detail.subtotal)
            .fetch_one(&mut *db)
            .await?;
        
        let detail = Self::parse_row_to_detail_transaksi(result);
        Ok(detail)
    }

    pub async fn get_detail_by_transaksi_id(mut db: PoolConnection<Any>, id_transaksi: i32) -> Result<Vec<DetailTransaksi>, sqlx::Error> {
        let rows = sqlx::query("
                SELECT id, id_transaksi, id_produk, harga_satuan, jumlah, subtotal
                FROM detail_transaksi
                WHERE id_transaksi = $1
            ")
            .bind(id_transaksi)
            .fetch_all(&mut *db)
            .await?;
        
        let mut detail_list = Vec::new();
        for row in rows {
            let detail = Self::parse_row_to_detail_transaksi(row);
            detail_list.push(detail);
        }
        
        Ok(detail_list)
    }

    pub async fn update_detail_transaksi(mut db: PoolConnection<Any>, detail: &DetailTransaksi) -> Result<DetailTransaksi, sqlx::Error> {
        let result = sqlx::query("
                UPDATE detail_transaksi
                SET id_produk = $1, harga_satuan = $2, jumlah = $3, subtotal = $4
                WHERE id = $5
                RETURNING id, id_transaksi, id_produk, harga_satuan, jumlah, subtotal
            ")
            .bind(detail.id_produk)
            .bind(detail.harga_satuan)
            .bind(detail.jumlah as i32)
            .bind(detail.subtotal)
            .bind(detail.id)
            .fetch_one(&mut *db)
            .await?;
        
        let detail = Self::parse_row_to_detail_transaksi(result);
        Ok(detail)
    }

    pub async fn delete_detail_transaksi(mut db: PoolConnection<Any>, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM detail_transaksi WHERE id = $1")
            .bind(id)
            .execute(&mut *db)
            .await?;
        
        Ok(())
    }

    pub async fn delete_detail_by_transaksi_id(mut db: PoolConnection<Any>, id_transaksi: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM detail_transaksi WHERE id_transaksi = $1")
            .bind(id_transaksi)
            .execute(&mut *db)
            .await?;
        
        Ok(())
    }

    fn parse_row_to_transaksi(row: AnyRow) -> Transaksi {
        let status_str: String = row.get("status");
        let status = StatusTransaksi::from_string(&status_str).unwrap_or(StatusTransaksi::MasihDiproses);

        Transaksi {
            id: row.get("id"),
            id_pelanggan: row.get("id_pelanggan"),
            nama_pelanggan: row.get("nama_pelanggan"),
            tanggal_transaksi: NaiveDateTime::parse_from_str(&row.get::<String, _>("tanggal_transaksi"), "%Y-%m-%d %H:%M:%S%.f").unwrap(),
            total_harga: row.get("total_harga"),
            status,
            catatan: row.get("catatan"),
        }
    }

    fn parse_row_to_detail_transaksi(row: AnyRow) -> DetailTransaksi {
        DetailTransaksi {
            id: row.get("id"),
            id_transaksi: row.get("id_transaksi"),
            id_produk: row.get("id_produk"),
            harga_satuan: row.get("harga_satuan"),
            jumlah: row.get::<i32, _>("jumlah") as u32,
            subtotal: row.get("subtotal"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::any::install_default_drivers;
    use sqlx::{Any, Pool};
    use sqlx::any::AnyPoolOptions;
    use rocket::async_test;

    async fn setup() -> Pool<Any> {
        install_default_drivers();
        let db = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("migrations/test")
            .run(&db)
            .await
            .unwrap();
        db
    }

    #[async_test]
    async fn test_create_transaksi() {
        let db = setup().await;

        let transaksi = Transaksi::new(
            1,
            "Castorice".to_string(),
            150000.0,
            Some("Test transaction".to_string()),
        );
        let created_transaksi = TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();

        assert_eq!(created_transaksi.id_pelanggan, 1);
        assert_eq!(created_transaksi.nama_pelanggan, "Castorice");
        assert_eq!(created_transaksi.total_harga, 150000.0);
        assert_eq!(created_transaksi.status, StatusTransaksi::MasihDiproses);
    }

    #[async_test]
    async fn test_get_transaksi_by_id() {
        let db = setup().await;

        let transaksi = Transaksi::new(
            2,
            "Tribbie".to_string(),
            200000.0,
            None,
        );
        let created_transaksi = TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();

        let fetched_transaksi = TransaksiRepository::get_transaksi_by_id(db.acquire().await.unwrap(), created_transaksi.id).await.unwrap();

        assert_eq!(fetched_transaksi.id_pelanggan, 2);
        assert_eq!(fetched_transaksi.nama_pelanggan, "Tribbie");
        assert_eq!(fetched_transaksi.total_harga, 200000.0);
    }

    #[async_test]
    async fn test_create_detail_transaksi() {
        let db = setup().await;

        let transaksi = Transaksi::new(
            1,
            "Hyacine".to_string(),
            500000.0,
            None,
        );
        let created_transaksi = TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();

        let detail = DetailTransaksi::new(
            created_transaksi.id,
            101,
            15000000.0,
            1,
        );
        let created_detail = TransaksiRepository::create_detail_transaksi(db.acquire().await.unwrap(), &detail).await.unwrap();

        assert_eq!(created_detail.id_transaksi, created_transaksi.id);
        assert_eq!(created_detail.id_produk, 101);
        assert_eq!(created_detail.subtotal, 15000000.0);
    }
}