// File: src/manajemen_produk/produk/repository.rs
use rocket_db_pools::Connection;
use rocket_db_pools::sqlx::{self, PgPool, Row, Error};
use crate::BuildingStoreDB;
use super::model::Produk;

pub struct ProdukRepository;

impl ProdukRepository {
    pub async fn tambah_produk(db: &PgPool, produk: &Produk) -> Result<i64, Error> {
        let result = sqlx::query(
            "INSERT INTO produk (nama, kategori, harga, stok, deskripsi) VALUES ($1, $2, $3, $4, $5) RETURNING id"
        )
        .bind(&produk.nama)
        .bind(&produk.kategori)
        .bind(produk.harga)
        .bind(produk.stok as i32)
        .bind(&produk.deskripsi)
        .fetch_one(db)
        .await?;

        Ok(result.get("id"))
    }
    
    pub async fn ambil_semua_produk(db: &PgPool) -> Result<Vec<Produk>, Error> {
        let rows = sqlx::query(
            "SELECT nama, kategori, harga, stok, deskripsi FROM produk"
        )
        .fetch_all(db)
        .await?;
        
        let produk_list = rows.iter().map(|row| {
            Produk {
                nama: row.get("nama"),
                kategori: row.get("kategori"),
                harga: row.get("harga"),
                stok: row.get("stok"),
                deskripsi: row.get("deskripsi"),
            }
        }).collect();
        
        Ok(produk_list)
    }
    
    pub async fn ambil_produk_by_id(db: &PgPool, id: i64) -> Result<Option<Produk>, Error> {
        let maybe_row = sqlx::query(
            "SELECT nama, kategori, harga, stok, deskripsi FROM produk WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(db)
        .await?;
        
        match maybe_row {
            Some(row) => Ok(Some(Produk {
                nama: row.get("nama"),
                kategori: row.get("kategori"),
                harga: row.get("harga"),
                stok: row.get("stok"),
                deskripsi: row.get("deskripsi"),
            })),
            None => Ok(None)
        }
    }
    
    pub async fn filter_produk_by_kategori(db: &PgPool, kategori: &str) -> Result<Vec<Produk>, Error> {
        let rows = sqlx::query(
            "SELECT nama, kategori, harga, stok, deskripsi FROM produk WHERE kategori = $1"
        )
        .bind(kategori)
        .fetch_all(db)
        .await?;
        
        let produk_list = rows.iter().map(|row| {
            Produk {
                nama: row.get("nama"),
                kategori: row.get("kategori"),
                harga: row.get("harga"),
                stok: row.get("stok"),
                deskripsi: row.get("deskripsi"),
            }
        }).collect();
        
        Ok(produk_list)
    }
}