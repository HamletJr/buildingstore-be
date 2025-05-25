use rocket::serde::json::Json;
use rocket::{get, routes, Route};
use crate::manajemen_produk::produk::repository;
use super::dto::{ProdukResponse, ApiResponse};

#[get("/produk")]
pub async fn list_produk() -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match repository::read::ambil_semua_produk().await {
        Ok(produk_list) => {
            let response_list = produk_list.into_iter()
                .map(ProdukResponse::from)
                .collect();
                
            Json(ApiResponse {
                success: true,
                message: Some("Berhasil mengambil daftar produk".to_string()),
                data: Some(response_list),
            })
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal mengambil daftar produk: {}", e)),
            data: None,
        }),
    }
}

#[get("/produk/<id>")]
pub async fn detail_produk(id: i64) -> Json<ApiResponse<ProdukResponse>> {
    match repository::read::ambil_produk_by_id(id).await {
        Ok(Some(produk)) => Json(ApiResponse {
            success: true,
            message: Some("Berhasil mengambil detail produk".to_string()),
            data: Some(ProdukResponse::from(produk)),
        }),
        Ok(None) => Json(ApiResponse {
            success: false,
            message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
            data: None,
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal mengambil detail produk: {}", e)),
            data: None,
        }),
    }
}

pub fn routes() -> Vec<Route> {
    routes![list_produk, detail_produk]
}

#[cfg(test)]
mod tests {
    use rocket::http::Status;
    use rocket::local::asynchronous::Client;
    use crate::manajemen_produk::produk::controller::dto::{ApiResponse, ProdukResponse};
    use crate::manajemen_produk::produk::model::Produk;
    use crate::manajemen_produk::produk::repository;
    use std::sync::Mutex;
    use std::sync::Arc;

    // Test synchronization to prevent race conditions
    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    async fn setup_test_client() -> Client {
        // Initialize database first
        let _ = repository::helper::init_database().await;
        
        // Use all routes instead of just read routes
        let rocket = rocket::build().mount("/api", super::super::routes());
        Client::tracked(rocket).await.expect("valid rocket instance")
    }

    async fn seed_test_data() {
        let produk1 = Produk::new(
            "Laptop Gaming".to_string(),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some("Laptop dengan RTX 4060".to_string()),
        );
        
        let produk2 = Produk::new(
            "Cat Tembok".to_string(),
            "Material".to_string(),
            150_000.0,
            50,
            Some("Cat tembok anti air".to_string()),
        );
        
        let produk3 = Produk::new(
            "Smartphone".to_string(),
            "Elektronik".to_string(),
            8_000_000.0,
            20,
            Some("Smartphone dengan kamera 108MP".to_string()),
        );
        
        let _ = repository::create::tambah_produk(&produk1).await;
        let _ = repository::create::tambah_produk(&produk2).await;
        let _ = repository::create::tambah_produk(&produk3).await;
    }

    async fn clean_test_data() {
        let _ = repository::delete::clear_all().await;
        // Add delay to ensure cleanup is complete
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    #[tokio::test]
    async fn test_list_produk() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        let client = setup_test_client().await;
        
        // Clean and seed test data
        clean_test_data().await;
        seed_test_data().await;
        
        // Test the endpoint
        let response = client.get("/api/produk")
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        assert_eq!(json.data.unwrap().len(), 3);
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_detail_produk() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        let client = setup_test_client().await;
        
        // Clean and seed test data
        clean_test_data().await;
        seed_test_data().await;
        
        // Get all products first to find an ID
        let response = client.get("/api/produk").dispatch().await;
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        let data = json.data.as_ref().unwrap();
        let produk_id = data[0].id.unwrap();
        
        // Test the detail endpoint
        let response = client.get(format!("/api/produk/{}", produk_id))
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        assert_eq!(json.data.unwrap().id.unwrap(), produk_id);
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_detail_produk_not_found() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        let client = setup_test_client().await;
        
        // Ensure clean state first
        clean_test_data().await;
        
        // Test with non-existent ID
        let response = client.get("/api/produk/9999")
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
        
        assert!(!json.success);
        assert!(json.data.is_none());
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_list_produk_empty() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        let client = setup_test_client().await;
        
        // Clean test data to ensure empty state and wait for cleanup
        clean_test_data().await;
        
        // Test the endpoint
        let response = client.get("/api/produk")
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        let products = json.data.unwrap();
        assert_eq!(products.len(), 0, "Expected empty list but got {} products: {:?}", 
                  products.len(), products.iter().map(|p| &p.nama).collect::<Vec<_>>());
        
        // Clean up (though should already be clean)
        clean_test_data().await;
    }
}