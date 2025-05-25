use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::repository::helper::{get_db_pool, row_to_produk, RepositoryError};

pub async fn ambil_semua_produk() -> Result<Vec<Produk>, RepositoryError> {
    let pool = get_db_pool()?;
    
    let rows = sqlx::query("SELECT * FROM products ORDER BY id")
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
    
    let row = sqlx::query("SELECT * FROM products WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    
    match row {
        Some(row) => Ok(Some(row_to_produk(&row)?)),
        None => Ok(None),
    }
}

pub async fn filter_produk_by_kategori(kategori: &str) -> Result<Vec<Produk>, RepositoryError> {
    let pool = get_db_pool()?;
    
    let rows = sqlx::query("SELECT * FROM products WHERE kategori = ? ORDER BY id")
        .bind(kategori)
        .fetch_all(pool)
        .await?;
    
    let mut products = Vec::new();
    for row in rows {
        products.push(row_to_produk(&row)?);
    }
    
    Ok(products)
}

pub async fn filter_produk_by_price_range(min_price: f64, max_price: f64) -> Result<Vec<Produk>, RepositoryError> {
    let pool = get_db_pool()?;
    
    let rows = sqlx::query("SELECT * FROM products WHERE harga >= ? AND harga <= ? ORDER BY id")
        .bind(min_price)
        .bind(max_price)
        .fetch_all(pool)
        .await?;
    
    let mut products = Vec::new();
    for row in rows {
        products.push(row_to_produk(&row)?);
    }
    
    Ok(products)
}

pub async fn filter_produk_by_stock_availability(min_stock: u32) -> Result<Vec<Produk>, RepositoryError> {
    let pool = get_db_pool()?;
    
    let rows = sqlx::query("SELECT * FROM products WHERE stok >= ? ORDER BY id")
        .bind(min_stock as i64)
        .fetch_all(pool)
        .await?;
    
    let mut products = Vec::new();
    for row in rows {
        products.push(row_to_produk(&row)?);
    }
    
    Ok(products)
}

pub async fn search_produk_by_name(name_query: &str) -> Result<Vec<Produk>, RepositoryError> {
    let pool = get_db_pool()?;
    let search_pattern = format!("%{}%", name_query);
    
    let rows = sqlx::query("SELECT * FROM products WHERE nama LIKE ? ORDER BY id")
        .bind(search_pattern)
        .fetch_all(pool)
        .await?;
    
    let mut products = Vec::new();
    for row in rows {
        products.push(row_to_produk(&row)?);
    }
    
    Ok(products)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manajemen_produk::produk::repository::create::tambah_produk;
    use crate::manajemen_produk::produk::repository::delete::clear_all;
    use crate::manajemen_produk::produk::repository::helper::init_database;
    use tokio::test;

    fn create_test_products() -> Vec<Produk> {
        vec![
            Produk::new("Laptop Gaming".to_string(), "Elektronik".to_string(), 15_000_000.0, 10, Some("Laptop dengan RTX 4060".to_string())),
            Produk::new("Cat Tembok".to_string(), "Material".to_string(), 150_000.0, 50, Some("Cat tembok anti air".to_string())),
            Produk::new("Smartphone".to_string(), "Elektronik".to_string(), 8_000_000.0, 20, Some("Smartphone dengan kamera 108MP".to_string())),
        ]
    }

    async fn setup_and_cleanup_repository() -> Result<(), RepositoryError> {
        let _ = init_database().await;
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

    #[test]
    async fn test_filter_by_kategori() {
        let _ = setup_and_cleanup_repository().await;
        
        let test_products = create_test_products();
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        let filtered = filter_produk_by_kategori("Elektronik").await.unwrap();
        assert_eq!(filtered.len(), 2);
        
        for product in filtered {
            assert_eq!(product.kategori, "Elektronik");
        }
    }

    #[test]
    async fn test_filter_by_price_range() {
        let _ = setup_and_cleanup_repository().await;
        
        let test_products = create_test_products();
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        let filtered = filter_produk_by_price_range(1_000_000.0, 10_000_000.0).await.unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].nama, "Smartphone");
    }

    #[test]
    async fn test_search_produk_by_name() {
        let _ = setup_and_cleanup_repository().await;
        
        let test_products = create_test_products();
        for product in &test_products {
            let _ = tambah_produk(product).await.unwrap();
        }
        
        let laptop_results = search_produk_by_name("Laptop").await.unwrap();
        assert_eq!(laptop_results.len(), 1);
        assert_eq!(laptop_results[0].nama, "Laptop Gaming");
        
        let no_results = search_produk_by_name("NonExistent").await.unwrap();
        assert!(no_results.is_empty());
    }
}