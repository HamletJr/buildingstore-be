use sqlx::{Any, pool::PoolConnection, any::AnyRow, Row};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::manajemen_supplier::model::supplier_transaction::SupplierTransaction;
use crate::manajemen_supplier::repository::supplier_transaction_repository::SupplierTransactionRepository;

pub struct SupplierTransactionRepositoryImpl;

impl SupplierTransactionRepositoryImpl {
    pub fn new() -> Self {
        Self
    }

    fn parse_row_to_transaction(row: AnyRow) -> Result<SupplierTransaction, sqlx::Error> {
        let id: String = row.get("id");
        let supplier_id: String = row.get("supplier_id");
        let supplier_name: String = row.get("supplier_name");
        let jenis_barang: String = row.get("jenis_barang");
        let jumlah_barang: i32 = row.get("jumlah_barang");
        let pengiriman_info: String = row.get("pengiriman_info");
        let tanggal_transaksi_str: String = row.get("tanggal_transaksi");
        let tanggal_transaksi = DateTime::parse_from_rfc3339(&tanggal_transaksi_str)
            .map_err(|e| sqlx::Error::ColumnDecode {
                index: "tanggal_transaksi".into(),
                source: Box::new(e),
            })?
            .with_timezone(&Utc);

        Ok(SupplierTransaction {
            id,
            supplier_id,
            supplier_name,
            jenis_barang,
            jumlah_barang,
            pengiriman_info,
            tanggal_transaksi,
        })
    }
}

#[async_trait]
impl SupplierTransactionRepository for SupplierTransactionRepositoryImpl {
    async fn save(&self, transaction: SupplierTransaction, mut db: PoolConnection<Any>) -> Result<SupplierTransaction, sqlx::Error> {
        let query = "
            INSERT INTO supplier_transactions (id, supplier_id, supplier_name, jenis_barang, jumlah_barang, pengiriman_info, tanggal_transaksi)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        ";

        sqlx::query(query)
            .bind(&transaction.id)
            .bind(&transaction.supplier_id)
            .bind(&transaction.supplier_name)
            .bind(&transaction.jenis_barang)
            .bind(transaction.jumlah_barang)
            .bind(&transaction.pengiriman_info)
            .bind(transaction.tanggal_transaksi.to_rfc3339())
            .execute(&mut *db)
            .await?;

        Ok(transaction)
    }

    async fn find_by_id(&self, id: &str, mut db: PoolConnection<Any>) -> Result<SupplierTransaction, sqlx::Error> {
        let query = "SELECT * FROM supplier_transactions WHERE id = $1";

        let row = sqlx::query(query)
            .bind(id)
            .fetch_one(&mut *db)
            .await?;

        Self::parse_row_to_transaction(row)
    }

    async fn find_by_supplier_id(&self, supplier_id: &str, mut db: PoolConnection<Any>) -> Result<Vec<SupplierTransaction>, sqlx::Error> {
        let query = "SELECT * FROM supplier_transactions WHERE supplier_id = $1";

        let rows = sqlx::query(query)
            .bind(supplier_id)
            .fetch_all(&mut *db)
            .await?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(Self::parse_row_to_transaction(row)?);
        }

        Ok(transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{any::{AnyPoolOptions, install_default_drivers}};
    use chrono::Utc;
    use uuid::Uuid;
    use crate::manajemen_supplier::model::supplier::Supplier;
    use crate::manajemen_supplier::repository::supplier_transaction_repository::SupplierTransactionRepository;

    async fn setup_repository() -> (SupplierTransactionRepositoryImpl, sqlx::Pool<Any>) {
        install_default_drivers();
        let db_pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to test DB");

        // Run migrations for test (ensure migrations/test folder is available)
        sqlx::migrate!("migrations/test")
            .run(&db_pool)
            .await
            .expect("Failed to run migrations");

        (SupplierTransactionRepositoryImpl::new(), db_pool)
    }

    fn create_supplier() -> Supplier {
        Supplier {
            id: format!("SUP-{}", Uuid::new_v4()),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now(),
        }
    }

    fn create_transaction(supplier: &Supplier) -> SupplierTransaction {
        SupplierTransaction {
            id: format!("TRX-{}", Uuid::new_v4()),
            supplier_id: supplier.id.clone(),
            supplier_name: supplier.name.clone(),
            jenis_barang: supplier.jenis_barang.clone(),
            jumlah_barang: supplier.jumlah_barang,
            pengiriman_info: supplier.resi.clone(),
            tanggal_transaksi: Utc::now(),
        }
    }

    #[tokio::test]
    async fn save_supplier_transaction() {
        let (repository, db_pool) = setup_repository().await;
        let supplier = create_supplier();
        let transaksi = create_transaction(&supplier);

        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.save(transaksi.clone(), db_conn).await;
        assert!(result.is_ok());
        let saved = result.unwrap();
        assert_eq!(saved.supplier_id, supplier.id);
        assert_eq!(saved.supplier_name, supplier.name);
    }

    #[tokio::test]
    async fn test_find_supplier_transaction_by_id() {
        let (repository, db_pool) = setup_repository().await;
        let supplier = create_supplier();
        let transaksi = create_transaction(&supplier);

        let db_conn = db_pool.acquire().await.unwrap();
        repository.save(transaksi.clone(), db_conn).await.unwrap();
        
        let db_conn = db_pool.acquire().await.unwrap();
        let found = repository.find_by_id(&transaksi.id, db_conn).await;
        assert!(found.is_ok());
        assert_eq!(found.unwrap().id, transaksi.id);
    }

    #[tokio::test]
    async fn test_find_nonexistent_transaction() {
        let (repository, db_pool) = setup_repository().await;
        let db_conn = db_pool.acquire().await.unwrap();
        let result = repository.find_by_id("non-existent-id", db_conn).await;
        
        // Should return an error (RowNotFound) when transaction doesn't exist
        assert!(result.is_err());
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {}, // Expected error
            _ => panic!("Expected RowNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_find_supplier_transaction_by_supplier_id() {
        let (repository, db_pool) = setup_repository().await;
        let supplier = create_supplier();
        let transaksi1 = create_transaction(&supplier);
        let transaksi2 = create_transaction(&supplier);

        let db_conn = db_pool.acquire().await.unwrap();
        repository.save(transaksi1.clone(), db_conn).await.unwrap();
        
        let db_conn = db_pool.acquire().await.unwrap();
        repository.save(transaksi2.clone(), db_conn).await.unwrap();

        let db_conn = db_pool.acquire().await.unwrap();
        let results = repository.find_by_supplier_id(&supplier.id, db_conn).await;
        assert!(results.is_ok());
        let transactions = results.unwrap();
        assert_eq!(transactions.len(), 2);
        assert!(transactions.iter().all(|t| t.supplier_id == supplier.id));
    }

    #[tokio::test]
    async fn test_find_transactions_for_nonexistent_supplier() {
        let (repository, db_pool) = setup_repository().await;
        let db_conn = db_pool.acquire().await.unwrap();
        let results = repository.find_by_supplier_id("non-existent-supplier", db_conn).await;
        
        assert!(results.is_ok());
        let transactions = results.unwrap();
        assert!(transactions.is_empty());
    }
}