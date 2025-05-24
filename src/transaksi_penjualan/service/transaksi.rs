use sqlx::{Any, Pool};
use std::sync::Arc;
use crate::transaksi_penjualan::model::transaksi::Transaksi;
use crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi;
use crate::transaksi_penjualan::repository::transaksi::TransaksiRepository;
use crate::transaksi_penjualan::enums::status_transaksi::StatusTransaksi;

use crate::transaksi_penjualan::patterns::strategy::sorting_strategy::{SortingStrategyFactory, FilteringStrategyFactory};
use crate::transaksi_penjualan::patterns::observer::transaksi_observer::{
    notify_transaction_created, notify_transaction_updated, notify_transaction_completed, 
    notify_transaction_cancelled, notify_item_added, GLOBAL_OBSERVER_MANAGER
};
use crate::transaksi_penjualan::patterns::state::transaksi_state::TransaksiStateFactory;

pub struct TransaksiService;

#[derive(Debug)]
pub struct TransaksiSearchParams {
    pub sort: Option<String>,
    pub filter: Option<String>,
    pub keyword: Option<String>,
    pub status: Option<String>,
    pub id_pelanggan: Option<i32>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TransaksiSearchResult {
    pub data: Vec<Transaksi>,
    pub total_count: usize,
    pub page: usize,
    pub limit: usize,
    pub total_pages: usize,
    pub applied_sort: Option<String>,
    pub applied_filter: Option<String>,
}

impl TransaksiSearchResult {
    pub fn empty() -> Self {
        Self {
            data: vec![],
            total_count: 0,
            page: 1,
            limit: 10,
            total_pages: 0,
            applied_sort: None,
            applied_filter: None,
        }
    }
}

impl TransaksiService {
    pub async fn create_transaksi(db: Pool<Any>, transaksi: &Transaksi) -> Result<Transaksi, sqlx::Error> {
        let db_connection = db.acquire().await?;
        let created_transaksi = TransaksiRepository::create_transaksi(db_connection, transaksi).await?;
        
        notify_transaction_created(&created_transaksi);
        
        Ok(created_transaksi)
    }

    pub async fn create_transaksi_with_details(
        db: Pool<Any>, 
        request: &crate::transaksi_penjualan::dto::transaksi_request::CreateTransaksiRequest
    ) -> Result<Transaksi, sqlx::Error> {
        // Validasi request
        if let Err(_err_msg) = request.validate() {
            return Err(sqlx::Error::RowNotFound);
        }

        // Validasi stok sebelum membuat transaksi
        if let Err(_err_msg) = Self::validate_product_stock(&request.detail_transaksi).await {
            return Err(sqlx::Error::RowNotFound);
        }

        // Ambil harga produk dari sistem (simulasi)
        let product_prices = Self::fetch_product_prices(&request.detail_transaksi).await?;
        
        let total_harga = request.calculate_total(&product_prices);

        let transaksi = Transaksi::new(
            request.id_pelanggan,
            request.nama_pelanggan.clone(),
            total_harga,
            request.catatan.clone(),
        );

        // Buat transaksi dalam database
        let db_connection = db.acquire().await?;
        let created_transaksi = TransaksiRepository::create_transaksi(db_connection, &transaksi).await?;

        // Buat detail transaksi dan kurangi stok
        for detail_request in &request.detail_transaksi {
            let harga_satuan = product_prices.get(&detail_request.id_produk).unwrap_or(&0.0);
            let detail = detail_request.to_detail_transaksi(created_transaksi.id, *harga_satuan);
            
            let db_connection = db.acquire().await?;
            let created_detail = TransaksiRepository::create_detail_transaksi(db_connection, &detail).await?;
            
            // Kurangi stok produk (simulasi - seharusnya memanggil service produk)
            Self::reduce_product_stock(detail_request.id_produk, detail_request.jumlah).await?;
            
            // OBSERVER PATTERN: Notify item added
            notify_item_added(created_transaksi.id, &created_detail);
        }

        // OBSERVER PATTERN: Notify transaction created
        notify_transaction_created(&created_transaksi);

        Ok(created_transaksi)
    }

    // Simulasi pengurangan stok produk
    async fn reduce_product_stock(product_id: i32, quantity: u32) -> Result<(), sqlx::Error> {
        // Di implementasi nyata, ini akan memanggil ProductService::reduce_stock()
        println!("Mengurangi stok produk ID {} sebanyak {}", product_id, quantity);
        Ok(())
    }

    // Simulasi pengembalian stok produk
    async fn restore_product_stock(product_id: i32, quantity: u32) -> Result<(), sqlx::Error> {
        // Di implementasi nyata, ini akan memanggil ProductService::increase_stock()
        println!("Mengembalikan stok produk ID {} sebanyak {}", product_id, quantity);
        Ok(())
    }

    pub async fn validate_product_stock(
        detail_requests: &[crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest]
    ) -> Result<(), String> {
        for detail in detail_requests {
            // Simulasi pengecekan stok (di implementasi nyata akan query ke database produk)
            let available_stock = Self::get_product_stock(detail.id_produk).await;

            if available_stock == 0 {
                return Err(format!("Produk dengan ID {} tidak ditemukan atau stok habis", detail.id_produk));
            }

            if detail.jumlah > available_stock {
                return Err(format!(
                    "Stok produk '{}' tidak mencukupi. Tersedia: {}, Diminta: {}", 
                    detail.nama_produk, available_stock, detail.jumlah
                ));
            }
        }

        Ok(())
    }

    // Simulasi mendapatkan stok produk
    async fn get_product_stock(product_id: i32) -> u32 {
        match product_id {
            1 => 100,
            2 => 50,
            3 => 25,
            4 => 15,
            5 => 30,
            _ => 0,
        }
    }

    async fn fetch_product_prices(
        detail_requests: &[crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest]
    ) -> Result<std::collections::HashMap<i32, f64>, sqlx::Error> {
        let mut prices = std::collections::HashMap::new();
        
        for detail in detail_requests {
            // Simulasi harga produk (di implementasi nyata akan query ke database produk)
            let price = match detail.id_produk {
                1 => 100000.0,   // Produk A
                2 => 250000.0,   // Produk B
                3 => 500000.0,   // Produk C
                4 => 75000.0,    // Produk D
                5 => 150000.0,   // Produk E
                _ => 50000.0,    // Default
            };
            prices.insert(detail.id_produk, price);
        }

        Ok(prices)
    }

    pub async fn get_transaksi_by_id(db: Pool<Any>, id: i32) -> Result<Transaksi, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::get_transaksi_by_id(db_connection, id).await
    }

    pub async fn update_transaksi(db: Pool<Any>, transaksi: &Transaksi) -> Result<Transaksi, sqlx::Error> {
        // Get old transaction for observer pattern
        let old_transaksi = Self::get_transaksi_by_id(db.clone(), transaksi.id).await?;
        
        // STATE PATTERN: Check if can be modified
        let temp_transaksi = old_transaksi.clone();
        if !temp_transaksi.can_be_modified() {
            return Err(sqlx::Error::RowNotFound);
        }

        let db_connection = db.acquire().await?;
        let updated_transaksi = TransaksiRepository::update_transaksi(db_connection, transaksi).await?;
        
        // OBSERVER PATTERN: Notify observers
        notify_transaction_updated(&old_transaksi, &updated_transaksi);
        
        Ok(updated_transaksi)
    }

    pub async fn delete_transaksi(db: Pool<Any>, id: i32) -> Result<(), sqlx::Error> {
        let existing_transaksi = Self::get_transaksi_by_id(db.clone(), id).await?;
        
        // STATE PATTERN: Check if can be cancelled
        if !existing_transaksi.can_be_cancelled() {
            return Err(sqlx::Error::RowNotFound); 
        }

        // Kembalikan stok produk sebelum menghapus transaksi
        let details = Self::get_detail_by_transaksi_id(db.clone(), id).await?;
        for detail in &details {
            Self::restore_product_stock(detail.id_produk, detail.jumlah).await?;
        }

        let db_connection_detail = db.acquire().await?;
        TransaksiRepository::delete_detail_by_transaksi_id(db_connection_detail, id).await?;

        let db_connection = db.acquire().await?;
        TransaksiRepository::delete_transaksi(db_connection, id).await?;
        
        // OBSERVER PATTERN: Notify observers
        notify_transaction_cancelled(&existing_transaksi);
        
        Ok(())
    }

    pub async fn get_all_transaksi(db: Pool<Any>) -> Result<Vec<Transaksi>, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::get_all_transaksi(db_connection).await
    }

    pub async fn get_transaksi_by_pelanggan(db: Pool<Any>, id_pelanggan: i32) -> Result<Vec<Transaksi>, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::get_transaksi_by_pelanggan(db_connection, id_pelanggan).await
    }

    pub async fn get_transaksi_by_status(db: Pool<Any>, status: &StatusTransaksi) -> Result<Vec<Transaksi>, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::get_transaksi_by_status(db_connection, status).await
    }

    pub async fn complete_transaksi(db: Pool<Any>, id: i32) -> Result<Transaksi, sqlx::Error> {
        let mut transaksi = Self::get_transaksi_by_id(db.clone(), id).await?;
        
        // STATE PATTERN: Use state machine for completion
        if let Err(_) = transaksi.complete() {
            return Err(sqlx::Error::RowNotFound);
        }

        let updated_transaksi = Self::update_transaksi(db, &transaksi).await?;
        
        // OBSERVER PATTERN: Notify completion
        notify_transaction_completed(&updated_transaksi);
        
        Ok(updated_transaksi)
    }

    pub async fn cancel_transaksi(db: Pool<Any>, id: i32) -> Result<Transaksi, sqlx::Error> {
        let mut transaksi = Self::get_transaksi_by_id(db.clone(), id).await?;
        
        // STATE PATTERN: Use state machine for cancellation
        if let Err(_) = transaksi.cancel() {
            return Err(sqlx::Error::RowNotFound);
        }

        // Kembalikan stok produk sebelum membatalkan transaksi
        let details = Self::get_detail_by_transaksi_id(db.clone(), id).await?;
        for detail in details {
            Self::restore_product_stock(detail.id_produk, detail.jumlah).await?;
        }

        let updated_transaksi = Self::update_transaksi(db, &transaksi).await?;
        
        // OBSERVER PATTERN: Notify cancellation
        notify_transaction_cancelled(&updated_transaksi);
        
        Ok(updated_transaksi)
    }

    pub async fn add_detail_transaksi(db: Pool<Any>, detail: &DetailTransaksi) -> Result<DetailTransaksi, sqlx::Error> {
        let transaksi = Self::get_transaksi_by_id(db.clone(), detail.id_transaksi).await?;
        
        // STATE PATTERN: Check if can add items
        if !transaksi.can_add_items() {
            return Err(sqlx::Error::RowNotFound);
        }

        let db_connection = db.acquire().await?;
        let created_detail = TransaksiRepository::create_detail_transaksi(db_connection, detail).await?;

        Self::recalculate_transaction_total(db, detail.id_transaksi).await?;
        
        // OBSERVER PATTERN: Notify item added
        notify_item_added(detail.id_transaksi, &created_detail);

        Ok(created_detail)
    }

    pub async fn get_detail_by_transaksi_id(db: Pool<Any>, id_transaksi: i32) -> Result<Vec<DetailTransaksi>, sqlx::Error> {
        let db_connection = db.acquire().await?;
        TransaksiRepository::get_detail_by_transaksi_id(db_connection, id_transaksi).await
    }

    pub async fn update_detail_transaksi(db: Pool<Any>, detail: &DetailTransaksi) -> Result<DetailTransaksi, sqlx::Error> {
        let transaksi = Self::get_transaksi_by_id(db.clone(), detail.id_transaksi).await?;
        
        // STATE PATTERN: Check if can update items
        if !transaksi.can_update_items() {
            return Err(sqlx::Error::RowNotFound);
        }

        let db_connection = db.acquire().await?;
        let updated_detail = TransaksiRepository::update_detail_transaksi(db_connection, detail).await?;

        Self::recalculate_transaction_total(db, detail.id_transaksi).await?;

        Ok(updated_detail)
    }

    pub async fn delete_detail_transaksi(db: Pool<Any>, id: i32, id_transaksi: i32) -> Result<(), sqlx::Error> {
        let transaksi = Self::get_transaksi_by_id(db.clone(), id_transaksi).await?;
        
        // STATE PATTERN: Check if can delete items
        if !transaksi.can_delete_items() {
            return Err(sqlx::Error::RowNotFound);
        }

        // Ambil detail sebelum dihapus untuk mengembalikan stok
        let details = Self::get_detail_by_transaksi_id(db.clone(), id_transaksi).await?;
        if let Some(detail_to_delete) = details.iter().find(|d| d.id == id) {
            Self::restore_product_stock(detail_to_delete.id_produk, detail_to_delete.jumlah).await?;
        }

        let db_connection = db.acquire().await?;
        TransaksiRepository::delete_detail_transaksi(db_connection, id).await?;

        Self::recalculate_transaction_total(db, id_transaksi).await?;

        Ok(())
    }

    async fn recalculate_transaction_total(db: Pool<Any>, id_transaksi: i32) -> Result<(), sqlx::Error> {
        let details = Self::get_detail_by_transaksi_id(db.clone(), id_transaksi).await?;
        let total: f64 = details.iter().map(|d| d.subtotal).sum();

        let mut transaksi = Self::get_transaksi_by_id(db.clone(), id_transaksi).await?;
        let old_transaksi = transaksi.clone();
        transaksi.update_total_harga(total);

        let db_connection = db.acquire().await?;
        let updated_transaksi = TransaksiRepository::update_transaksi(db_connection, &transaksi).await?;
        
        // OBSERVER PATTERN: Notify update due to recalculation
        notify_transaction_updated(&old_transaksi, &updated_transaksi);

        Ok(())
    }

    // STRATEGY PATTERN: Method untuk search transaksi dengan pagination
    pub async fn search_transaksi_with_pagination(
        db: Pool<Any>,
        search_params: &TransaksiSearchParams
    ) -> Result<TransaksiSearchResult, sqlx::Error> {
        let mut transaksi_list = if let Some(customer_id) = search_params.id_pelanggan {
            Self::get_transaksi_by_pelanggan(db.clone(), customer_id).await?
        } else if let Some(ref status_str) = search_params.status {
            if let Some(status_enum) = StatusTransaksi::from_string(status_str) {
                Self::get_transaksi_by_status(db.clone(), &status_enum).await?
            } else {
                return Ok(TransaksiSearchResult::empty());
            }
        } else {
            Self::get_all_transaksi(db).await?
        };

        let mut result = TransaksiSearchResult::empty();

        // STRATEGY PATTERN: Apply sorting using strategy
        if let Some(ref sort_strategy) = search_params.sort {
            let sorting_strategy = SortingStrategyFactory::create(sort_strategy);
            transaksi_list = sorting_strategy.sort(transaksi_list);
            result.applied_sort = Some(sorting_strategy.get_name().to_string());
        }

        // STRATEGY PATTERN: Apply filtering using strategy
        if let Some(ref filter_strategy) = search_params.filter {
            if let Some(ref keyword_value) = search_params.keyword {
                let filtering_strategy = FilteringStrategyFactory::create(filter_strategy);
                transaksi_list = filtering_strategy.filter(transaksi_list, keyword_value);
                result.applied_filter = Some(filtering_strategy.get_name().to_string());
            }
        }

        // Apply pagination
        let total_count = transaksi_list.len();
        let page = search_params.page.unwrap_or(1);
        let limit = search_params.limit.unwrap_or(10);
        let offset = (page - 1) * limit;

        let paginated_list = transaksi_list
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();

        result.data = paginated_list;
        result.total_count = total_count;
        result.page = page;
        result.limit = limit;
        result.total_pages = (total_count + limit - 1) / limit;

        Ok(result)
    }

    // Legacy methods untuk backward compatibility (menggunakan Strategy Pattern di belakang layar)
    pub fn sort_transaksi(transaksi_list: Vec<Transaksi>, sort_by: &str) -> Vec<Transaksi> {
        let strategy = SortingStrategyFactory::create(sort_by);
        strategy.sort(transaksi_list)
    }

    pub fn filter_transaksi(transaksi_list: Vec<Transaksi>, filter_by: &str, keyword: &str) -> Vec<Transaksi> {
        let strategy = FilteringStrategyFactory::create(filter_by);
        strategy.filter(transaksi_list, keyword)
    }

    // Method untuk mengelola observers
    pub fn add_observer(observer: Arc<dyn crate::transaksi_penjualan::patterns::observer::transaksi_observer::TransaksiObserver>) {
        if let Ok(mut subject) = GLOBAL_OBSERVER_MANAGER.lock() {
            subject.attach(observer);
        }
    }

    pub fn remove_observer(observer_name: &str) {
        if let Ok(mut subject) = GLOBAL_OBSERVER_MANAGER.lock() {
            subject.detach(observer_name);
        }
    }

    pub fn get_active_observers() -> Vec<String> {
        if let Ok(subject) = GLOBAL_OBSERVER_MANAGER.lock() {
            subject.get_observer_names().iter().map(|s| s.to_string()).collect()
        } else {
            vec![]
        }
    }

    // Method untuk mendapatkan allowed actions berdasarkan state
    pub async fn get_allowed_actions(db: Pool<Any>, id: i32) -> Result<Vec<String>, sqlx::Error> {
        let transaksi = Self::get_transaksi_by_id(db, id).await?;
        let state = TransaksiStateFactory::create_state(&transaksi.status);
        Ok(state.get_allowed_actions())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::any::install_default_drivers;
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
    async fn test_state_pattern_implementation() {
        let db = setup().await;

        let transaksi = Transaksi::new(
            1,
            "Test Customer".to_string(),
            150000.0,
            Some("Test transaction".to_string()),
        );

        let created = TransaksiService::create_transaksi(db.clone(), &transaksi).await.unwrap();
        
        // Test state transitions
        let completed = TransaksiService::complete_transaksi(db.clone(), created.id).await.unwrap();
        assert_eq!(completed.status, StatusTransaksi::Selesai);

        // Test that completed transaction cannot be modified
        let actions = TransaksiService::get_allowed_actions(db, completed.id).await.unwrap();
        assert!(!actions.contains(&"add_item".to_string()));
    }

    #[async_test]
    async fn test_strategy_pattern_sorting() {
        let db = setup().await;

        // Create multiple transactions
        for i in 1..=5 {
            let transaksi = Transaksi::new(
                i,
                format!("Customer {}", 6 - i), // Reverse order names
                (i as f64) * 100000.0,
                None,
            );
            TransaksiService::create_transaksi(db.clone(), &transaksi).await.unwrap();
        }

        let search_params = TransaksiSearchParams {
            sort: Some("customer".to_string()),
            filter: None,
            keyword: None,
            status: None,
            id_pelanggan: None,
            page: Some(1),
            limit: Some(10),
        };

        let result = TransaksiService::search_transaksi_with_pagination(db, &search_params).await.unwrap();
        
        // Should be sorted by customer name
        assert_eq!(result.data[0].nama_pelanggan, "Customer 1");
        assert_eq!(result.applied_sort, Some("customer_asc".to_string()));
    }

    #[async_test]
    async fn test_strategy_pattern_filtering() {
        let db = setup().await;

        let transaksi1 = Transaksi::new(1, "Alice".to_string(), 100000.0, None);
        let transaksi2 = Transaksi::new(2, "Bob".to_string(), 200000.0, None);
        let transaksi3 = Transaksi::new(3, "Alice Smith".to_string(), 150000.0, None);

        TransaksiService::create_transaksi(db.clone(), &transaksi1).await.unwrap();
        TransaksiService::create_transaksi(db.clone(), &transaksi2).await.unwrap();
        TransaksiService::create_transaksi(db.clone(), &transaksi3).await.unwrap();

        let search_params = TransaksiSearchParams {
            sort: None,
            filter: Some("customer".to_string()),
            keyword: Some("Alice".to_string()),
            status: None,
            id_pelanggan: None,
            page: Some(1),
            limit: Some(10),
        };

        let result = TransaksiService::search_transaksi_with_pagination(db, &search_params).await.unwrap();
        
        assert_eq!(result.data.len(), 2);
        assert!(result.data.iter().all(|t| t.nama_pelanggan.contains("Alice")));
        assert_eq!(result.applied_filter, Some("customer".to_string()));
    }

    #[async_test]
    async fn test_observer_pattern() {
        let db = setup().await;

        // Test that observers are notified
        let observer_count_before = if let Ok(subject) = GLOBAL_OBSERVER_MANAGER.lock() {
            subject.get_observer_count()
        } else {
            0
        };

        let transaksi = Transaksi::new(
            1,
            "Observer Test".to_string(),
            100000.0,
            None,
        );

        // This should trigger observer notifications
        TransaksiService::create_transaksi(db.clone(), &transaksi).await.unwrap();

        // Verify observers are still there
        let observer_count_after = if let Ok(subject) = GLOBAL_OBSERVER_MANAGER.lock() {
            subject.get_observer_count()
        } else {
            0
        };

        assert_eq!(observer_count_before, observer_count_after);
        assert!(observer_count_after > 0); // Should have default observers
    }
}