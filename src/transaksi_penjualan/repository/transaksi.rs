use sqlx::any::AnyRow;
use sqlx::{Any, pool::PoolConnection};
use sqlx::Row;
use chrono::{DateTime, NaiveDateTime, Utc};

use crate::transaksi_penjualan::model::transaksi::Transaksi;
use crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi;
use crate::transaksi_penjualan::enums::status_transaksi::StatusTransaksi;

pub struct TransaksiRepository;

impl TransaksiRepository {
    pub async fn create_transaksi(mut db: PoolConnection<Any>, transaksi: &Transaksi) -> Result<Transaksi, sqlx::Error> {
        let result = sqlx::query("
                INSERT INTO transaksi (id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan
            ")
            .bind(transaksi.id_pelanggan)
            .bind(&transaksi.nama_pelanggan)
            .bind(transaksi.tanggal_transaksi.to_string())
            .bind(transaksi.total_harga)
            .bind(transaksi.status.to_string())
            .bind(transaksi.catatan.as_ref().map(|s| s.as_str()).unwrap_or(""))
            .bind(DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap().to_string())
            .bind(DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap().to_string())
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
                    total_harga = $4, status = $5, catatan = $6, updated_at = $7
                WHERE id = $8
                RETURNING id, id_pelanggan, nama_pelanggan, tanggal_transaksi, total_harga, status, catatan
            ")
            .bind(transaksi.id_pelanggan)
            .bind(&transaksi.nama_pelanggan)
            .bind(transaksi.tanggal_transaksi.to_string())
            .bind(transaksi.total_harga)
            .bind(transaksi.status.to_string())
            .bind(transaksi.catatan.as_ref().map(|s| s.as_str()).unwrap_or(""))
            .bind(DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap().to_string())
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
                INSERT INTO detail_transaksi (id_transaksi, id_produk, harga_satuan, jumlah, subtotal, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, id_transaksi, id_produk, harga_satuan, jumlah, subtotal
            ")
            .bind(detail.id_transaksi)
            .bind(detail.id_produk)
            .bind(detail.harga_satuan)
            .bind(detail.jumlah as i32)
            .bind(detail.subtotal)
            .bind(DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap().to_string())
            .bind(DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap().to_string())
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
                SET id_produk = $1, harga_satuan = $2, jumlah = $3, subtotal = $4, updated_at = $5
                WHERE id = $6
                RETURNING id, id_transaksi, id_produk, harga_satuan, jumlah, subtotal
            ")
            .bind(detail.id_produk)
            .bind(detail.harga_satuan)
            .bind(detail.jumlah as i32)
            .bind(detail.subtotal)
            .bind(DateTime::from_timestamp(Utc::now().timestamp(), 0).unwrap().to_string())
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

        let mut transaksi = Transaksi::new(
            row.get("id_pelanggan"),
            row.get("nama_pelanggan"),
            row.get("total_harga"),
            row.get("catatan"),
        );

        transaksi.id = row.get("id");
        transaksi.tanggal_transaksi = NaiveDateTime::parse_from_str(
            &row.get::<String, _>("tanggal_transaksi"), 
            "%Y-%m-%d %H:%M:%S%.f"
        ).unwrap();
        transaksi.status = status;

        transaksi
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
        
        sqlx::migrate!("./migrations/test")
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

    #[async_test]
    async fn test_get_all_transaksi() {
        let db = setup().await;

        let transaksi1 = Transaksi::new(1, "Alice".to_string(), 100000.0, None);
        let transaksi2 = Transaksi::new(2, "Bob".to_string(), 200000.0, None);

        TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi1).await.unwrap();
        TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi2).await.unwrap();

        let all_transaksi = TransaksiRepository::get_all_transaksi(db.acquire().await.unwrap()).await.unwrap();
        
        assert_eq!(all_transaksi.len(), 2);
        assert!(all_transaksi.iter().any(|t| t.nama_pelanggan == "Alice"));
        assert!(all_transaksi.iter().any(|t| t.nama_pelanggan == "Bob"));
    }

    #[async_test]
    async fn test_get_transaksi_by_status() {
        let db = setup().await;

        let mut transaksi = Transaksi::new(1, "Test User".to_string(), 100000.0, None);
        let created = TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();
        
        transaksi.id = created.id;
        transaksi.update_status(StatusTransaksi::Selesai);
        TransaksiRepository::update_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();

        let completed_transaksi = TransaksiRepository::get_transaksi_by_status(
            db.acquire().await.unwrap(), 
            &StatusTransaksi::Selesai
        ).await.unwrap();
        
        assert_eq!(completed_transaksi.len(), 1);
        assert_eq!(completed_transaksi[0].status, StatusTransaksi::Selesai);
    }

    #[async_test]
    async fn test_update_transaksi() {
        let db = setup().await;

        let transaksi = Transaksi::new(1, "Original Name".to_string(), 100000.0, None);
        let created = TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();

        let mut updated_transaksi = created.clone();
        updated_transaksi.total_harga = 200000.0;
        
        let result = TransaksiRepository::update_transaksi(db.acquire().await.unwrap(), &updated_transaksi).await.unwrap();
        
        assert_eq!(result.total_harga, 200000.0);
        assert_eq!(result.id, created.id);
    }

    #[async_test]
    async fn test_delete_transaksi() {
        let db = setup().await;

        let transaksi = Transaksi::new(1, "To Delete".to_string(), 100000.0, None);
        let created = TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();

        TransaksiRepository::delete_transaksi(db.acquire().await.unwrap(), created.id).await.unwrap();

        let result = TransaksiRepository::get_transaksi_by_id(db.acquire().await.unwrap(), created.id).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_detail_transaksi_operations() {
        let db = setup().await;

        let transaksi = Transaksi::new(1, "Detail Test".to_string(), 0.0, None);
        let created_transaksi = TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();

        let detail1 = DetailTransaksi::new(created_transaksi.id, 1, 100000.0, 2);
        let detail2 = DetailTransaksi::new(created_transaksi.id, 2, 50000.0, 3);

        let created_detail1 = TransaksiRepository::create_detail_transaksi(db.acquire().await.unwrap(), &detail1).await.unwrap();
        let created_detail2 = TransaksiRepository::create_detail_transaksi(db.acquire().await.unwrap(), &detail2).await.unwrap();

        let details = TransaksiRepository::get_detail_by_transaksi_id(db.acquire().await.unwrap(), created_transaksi.id).await.unwrap();
        assert_eq!(details.len(), 2);

        let mut updated_detail1 = created_detail1.clone();
        updated_detail1.jumlah = 5;
        updated_detail1.subtotal = 500000.0;
        
        let result = TransaksiRepository::update_detail_transaksi(db.acquire().await.unwrap(), &updated_detail1).await.unwrap();
        assert_eq!(result.jumlah, 5);
        assert_eq!(result.subtotal, 500000.0);

        TransaksiRepository::delete_detail_transaksi(db.acquire().await.unwrap(), created_detail2.id).await.unwrap();
        
        let remaining_details = TransaksiRepository::get_detail_by_transaksi_id(db.acquire().await.unwrap(), created_transaksi.id).await.unwrap();
        assert_eq!(remaining_details.len(), 1);
        assert_eq!(remaining_details[0].id, created_detail1.id);
    }

    #[async_test]
    async fn test_state_pattern_integration() {
        let db = setup().await;

        let transaksi = Transaksi::new(
            1,
            "State Test".to_string(),
            100000.0,
            None,
        );

        assert!(transaksi.can_be_modified());
        assert!(transaksi.can_be_completed());

        let created_transaksi = TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();
        
        let mut fetched_transaksi = TransaksiRepository::get_transaksi_by_id(db.acquire().await.unwrap(), created_transaksi.id).await.unwrap();
        assert!(fetched_transaksi.can_be_modified());
        
        fetched_transaksi.complete().unwrap();
        assert!(!fetched_transaksi.can_be_modified());
        
        let updated_transaksi = TransaksiRepository::update_transaksi(db.acquire().await.unwrap(), &fetched_transaksi).await.unwrap();
        assert_eq!(updated_transaksi.status, StatusTransaksi::Selesai);
    }

    #[async_test]
    async fn test_get_transaksi_by_pelanggan() {
        let db = setup().await;

        let transaksi1 = Transaksi::new(1, "Customer 1".to_string(), 100000.0, None);
        let transaksi2 = Transaksi::new(1, "Customer 1 Again".to_string(), 200000.0, None);
        let transaksi3 = Transaksi::new(2, "Customer 2".to_string(), 150000.0, None);

        TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi1).await.unwrap();
        TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi2).await.unwrap();
        TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi3).await.unwrap();

        let customer1_transaksi = TransaksiRepository::get_transaksi_by_pelanggan(
            db.acquire().await.unwrap(), 
            1
        ).await.unwrap();
        
        assert_eq!(customer1_transaksi.len(), 2);
        assert!(customer1_transaksi.iter().all(|t| t.id_pelanggan == 1));

        let customer2_transaksi = TransaksiRepository::get_transaksi_by_pelanggan(
            db.acquire().await.unwrap(), 
            2
        ).await.unwrap();
        
        assert_eq!(customer2_transaksi.len(), 1);
        assert_eq!(customer2_transaksi[0].id_pelanggan, 2);
    }

    #[async_test]
    async fn test_delete_detail_by_transaksi_id() {
        let db = setup().await;

        let transaksi = Transaksi::new(1, "Batch Delete Test".to_string(), 0.0, None);
        let created_transaksi = TransaksiRepository::create_transaksi(db.acquire().await.unwrap(), &transaksi).await.unwrap();

        let detail1 = DetailTransaksi::new(created_transaksi.id, 1, 100000.0, 2);
        let detail2 = DetailTransaksi::new(created_transaksi.id, 2, 50000.0, 3);
        let detail3 = DetailTransaksi::new(created_transaksi.id, 3, 75000.0, 1);

        TransaksiRepository::create_detail_transaksi(db.acquire().await.unwrap(), &detail1).await.unwrap();
        TransaksiRepository::create_detail_transaksi(db.acquire().await.unwrap(), &detail2).await.unwrap();
        TransaksiRepository::create_detail_transaksi(db.acquire().await.unwrap(), &detail3).await.unwrap();

        let details_before = TransaksiRepository::get_detail_by_transaksi_id(db.acquire().await.unwrap(), created_transaksi.id).await.unwrap();
        assert_eq!(details_before.len(), 3);

        TransaksiRepository::delete_detail_by_transaksi_id(db.acquire().await.unwrap(), created_transaksi.id).await.unwrap();

        let details_after = TransaksiRepository::get_detail_by_transaksi_id(db.acquire().await.unwrap(), created_transaksi.id).await.unwrap();
        assert_eq!(details_after.len(), 0);
    }
}