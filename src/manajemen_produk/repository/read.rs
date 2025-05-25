use crate::manajemen_produk::model::Produk;
use crate::manajemen_produk::repository::dto::{get_db_pool, row_to_produk, RepositoryError};

pub async fn ambil_semua_produk() -> Result<Vec<Produk>, RepositoryError> {
    let pool = get_db_pool()?;
    
    let rows = sqlx::query("SELECT * FROM produk ORDER BY id")
        .fetch_all(pool)
        .await?;
    
    let mut products = Vec::new();
    for row in rows {
        products.push(row_to_produk(&row)?);
    }
    
    Ok(products)
}

pub async fn ambil_produk_by_id(id: i64) -> Result<Option<Produk>, RepositoryError> {
    let pool = get_db_pool()?;
    
    let row = sqlx::query("SELECT * FROM produk WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    
    match row {
        Some(row) => Ok(Some(row_to_produk(&row)?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::repository::create::tambah_produk;
    use crate::manajemen_produk::repository::delete::clear_all;
    use crate::manajemen_produk::repository::dto::init_database;
    use tokio::test;

    fn create_test_products() -> Vec<Produk> {
        vec![
            Produk::new("Laptop Gaming".to_string(), "Elektronik".to_string(), 15_000_000.0, 10, Some("Laptop dengan RTX 4060".to_string())),
            Produk::new("Cat Tembok".to_string(), "Material".to_string(), 150_000.0, 50, Some("Cat tembok anti air".to_string())),
            Produk::new("Smartphone".to_string(), "Elektronik".to_string(), 8_000_000.0, 20, Some("Smartphone dengan kamera 108MP".to_string())),
        ]
    }

    async fn setup_and_cleanup_repository() -> Result<(), RepositoryError> {
        // Always initialize database first
        init_database().await?;
        clear_all().await?;
        Ok(())
    }

    #[test]
    async fn test_ambil_semua_produk() {
        let _ = setup_and_cleanup_repository().await;
        
        let test_products = create_test_products();
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        let all_products = ambil_semua_produk().await.unwrap();
        assert_eq!(all_products.len(), test_products.len());
    }

    #[test]
    async fn test_ambil_produk_by_id() {
        let _ = setup_and_cleanup_repository().await;
        
        let produk = create_test_products()[0].clone();
        let id = tambah_produk(&produk).await.unwrap();
        
        let retrieved = ambil_produk_by_id(id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_produk = retrieved.unwrap();
        assert_eq!(retrieved_produk.id.unwrap(), id);
        assert_eq!(retrieved_produk.nama, produk.nama);
    }
}