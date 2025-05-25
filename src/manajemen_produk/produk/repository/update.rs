use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::repository::dto::{get_db_pool, validate_produk, RepositoryError};

pub async fn update_produk(id: i64, produk: &Produk) -> Result<bool, RepositoryError> {
    // Validasi input
    validate_produk(produk)?;
    
    let pool = get_db_pool()?;
    
    let result = sqlx::query(
        r#"
        UPDATE produk 
        SET nama = ?, kategori = ?, harga = ?, stok = ?, deskripsi = ?
        WHERE id = ?
        "#
    )
    .bind(&produk.nama)
    .bind(&produk.kategori)
    .bind(produk.harga)
    .bind(produk.stok as i64)
    .bind(&produk.deskripsi)
    .bind(id)
    .execute(pool)
    .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}

pub async fn update_stok(id: i64, new_stok: u32) -> Result<bool, RepositoryError> {
    let pool = get_db_pool()?;
    
    let result = sqlx::query("UPDATE produk SET stok = ? WHERE id = ?")
        .bind(new_stok as i64)
        .bind(id)
        .execute(pool)
        .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}

pub async fn update_harga(id: i64, new_harga: f64) -> Result<bool, RepositoryError> {
    if new_harga < 0.0 {
        return Err(RepositoryError::ValidationError("Harga tidak boleh negatif".to_string()));
    }
    
    let pool = get_db_pool()?;
    
    let result = sqlx::query("UPDATE produk SET harga = ? WHERE id = ?")
        .bind(new_harga)
        .bind(id)
        .execute(pool)
        .await?;
    
    if result.rows_affected() == 0 {
        Err(RepositoryError::NotFound)
    } else {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::produk::repository::create::tambah_produk;
    use crate::manajemen_produk::produk::repository::read::ambil_produk_by_id;
    use crate::manajemen_produk::produk::repository::delete::clear_all;
    use crate::manajemen_produk::produk::repository::dto::init_database;
    use tokio::test;

    fn create_test_product() -> Produk {
        Produk::new(
            "Laptop Gaming".to_string(),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some("Laptop dengan RTX 4060".to_string()),
        )
    }

    async fn cleanup_repository() -> Result<(), RepositoryError> {
        // Always initialize database first
        init_database().await?;
        clear_all().await
    }

    #[test]
    async fn test_update_produk() {
        let _ = cleanup_repository().await;
        
        let mut produk = create_test_product();
        let id = tambah_produk(&produk).await.unwrap();
        
        produk.nama = "Updated Laptop".to_string();
        produk.harga = 17_000_000.0;
        produk.stok = 12;
        
        let update_result = update_produk(id, &produk).await.unwrap();
        assert!(update_result);
        
        let updated = ambil_produk_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.nama, "Updated Laptop");
        assert_eq!(updated.harga, 17_000_000.0);
        assert_eq!(updated.stok, 12);
    }

    #[test]
    async fn test_update_produk_tidak_ada() {
        let _ = cleanup_repository().await;
        
        let produk = create_test_product();
        let update_result = update_produk(9999, &produk).await;
        
        assert!(update_result.is_err());
        match update_result {
            Err(RepositoryError::NotFound) => {},
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    async fn test_update_stok() {
        let _ = cleanup_repository().await;
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await.unwrap();
        
        let new_stok = 25;
        let update_result = update_stok(id, new_stok).await.unwrap();
        assert!(update_result);
        
        let updated = ambil_produk_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.stok, new_stok);
        assert_eq!(updated.nama, produk.nama); // Other fields unchanged
    }

    #[test]
    async fn test_update_harga() {
        let _ = cleanup_repository().await;
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await.unwrap();
        
        let new_harga = 18_000_000.0;
        let update_result = update_harga(id, new_harga).await.unwrap();
        assert!(update_result);
        
        let updated = ambil_produk_by_id(id).await.unwrap().unwrap();
        assert_eq!(updated.harga, new_harga);
        assert_eq!(updated.nama, produk.nama); // Other fields unchanged
    }

    #[test]
    async fn test_update_harga_negatif() {
        let _ = cleanup_repository().await;
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await.unwrap();
        
        let update_result = update_harga(id, -1000.0).await;
        
        assert!(update_result.is_err());
        match update_result {
            Err(RepositoryError::ValidationError(msg)) => {
                assert_eq!(msg, "Harga tidak boleh negatif");
            },
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    async fn test_update_with_invalid_data() {
        let _ = cleanup_repository().await;
        
        let produk = create_test_product();
        let id = tambah_produk(&produk).await.unwrap();
        
        let invalid_produk = Produk::new("".to_string(), "Elektronik".to_string(), 15_000_000.0, 10, None);
        
        let update_result = update_produk(id, &invalid_produk).await;
        
        assert!(update_result.is_err());
        match update_result {
            Err(RepositoryError::ValidationError(_)) => {},
            _ => panic!("Expected ValidationError"),
        }
        
        // Original product should remain unchanged
        let original = ambil_produk_by_id(id).await.unwrap().unwrap();
        assert_eq!(original.nama, produk.nama);
    }
}