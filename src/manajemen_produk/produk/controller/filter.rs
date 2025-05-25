use rocket::serde::json::Json;
use rocket::{get, routes, Route};
use crate::manajemen_produk::produk::repository;
use super::dto::{ProdukResponse, ApiResponse};

#[get("/produk/kategori/<kategori>")]
pub async fn filter_produk_by_kategori(
    kategori: String
) -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match repository::read::filter_produk_by_kategori(&kategori).await {
        Ok(produk_list) => {
            let response_list = produk_list.into_iter()
                .map(ProdukResponse::from)
                .collect();
                
            Json(ApiResponse {
                success: true,
                message: Some(format!("Berhasil filter produk berdasarkan kategori '{}'", kategori)),
                data: Some(response_list),
            })
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal filter produk: {}", e)),
            data: None,
        }),
    }
}

#[get("/produk/harga?<min>&<max>")]
pub async fn filter_produk_by_price(
    min: f64,
    max: f64
) -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match repository::read::filter_produk_by_price_range(min, max).await {
        Ok(produk_list) => {
            let response_list = produk_list.into_iter()
                .map(ProdukResponse::from)
                .collect();
                
            Json(ApiResponse {
                success: true,
                message: Some(format!("Berhasil filter produk dengan harga {} - {}", min, max)),
                data: Some(response_list),
            })
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal filter produk: {}", e)),
            data: None,
        }),
    }
}

#[get("/produk/stok?<min_stok>")]
pub async fn filter_produk_by_stock(
    min_stok: i32
) -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match repository::read::filter_produk_by_stock_availability(min_stok.try_into().unwrap_or(0)).await {
        Ok(produk_list) => {
            let response_list = produk_list.into_iter()
                .map(ProdukResponse::from)
                .collect();
                
            Json(ApiResponse {
                success: true,
                message: Some(format!("Berhasil filter produk dengan stok minimal {}", min_stok)),
                data: Some(response_list),
            })
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Gagal filter produk: {}", e)),
            data: None,
        }),
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        filter_produk_by_kategori,
        filter_produk_by_price,
        filter_produk_by_stock,
    ]
}

#[cfg(test)]
mod tests {
    use rocket::http::Status;
    use rocket::local::asynchronous::Client;
    use crate::manajemen_produk::produk::controller::dto::{ApiResponse, ProdukResponse};
    use crate::manajemen_produk::produk::model::Produk;
    use crate::manajemen_produk::produk::repository;

    async fn setup_test_client() -> Client {
        // Use all routes instead of just filter routes
        let rocket = rocket::build().mount("/api", super::super::routes());
        Client::tracked(rocket).await.expect("valid rocket instance")
    }

    async fn seed_test_data() {
        // Ensure clean state first
        clean_test_data().await;
        
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
        // Add small delay to ensure cleanup is complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_filter_produk_by_kategori() {
        let client = setup_test_client().await;
        
        // Clean and seed test data
        clean_test_data().await;
        seed_test_data().await;
        
        // Test filter by category
        let response = client.get("/api/produk/kategori/Elektronik")
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        let products = json.data.unwrap();
        assert_eq!(products.len(), 2); // Should have 2 electronic products
        assert!(products.iter().all(|p| p.kategori == "Elektronik"));
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_filter_produk_by_kategori_not_found() {
        let client = setup_test_client().await;
        
        // Clean and seed test data
        clean_test_data().await;
        seed_test_data().await;
        
        // Test with non-existent category
        let response = client.get("/api/produk/kategori/NonExistent")
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        assert_eq!(json.data.unwrap().len(), 0); // Empty list, not an error
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_filter_produk_by_price() {
        let client = setup_test_client().await;
        
        // Clean and seed test data
        clean_test_data().await;
        seed_test_data().await;
        
        // Test filter by price range - Fixed to exclude Cat Tembok (150,000)
        // Use range 5,000,000 - 20,000,000 to match only Laptop Gaming (15M) and Smartphone (8M)
        let response = client.get("/api/produk/harga?min=5000000&max=20000000")
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success, "Expected success but got: {:?}", json.message);
        let products = json.data.unwrap();
        
        // Should have 2 products in this price range (Laptop Gaming: 15M, Smartphone: 8M)
        assert_eq!(products.len(), 2, "Expected 2 products but got {}: {:?}", 
                  products.len(), products.iter().map(|p| format!("{}: {}", p.nama, p.harga)).collect::<Vec<_>>());
        assert!(products.iter().all(|p| p.harga >= 5_000_000.0 && p.harga <= 20_000_000.0),
                "Not all products are in the expected price range");
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_filter_produk_by_stock() {
        let client = setup_test_client().await;
        
        // Clean and seed test data
        clean_test_data().await;
        seed_test_data().await;
        
        // Test filter by minimum stock
        // Seed data: Laptop Gaming (10), Cat Tembok (50), Smartphone (20)
        // min_stok=20 should match Cat Tembok (50) and Smartphone (20)
        let response = client.get("/api/produk/stok?min_stok=20")
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        let products = json.data.unwrap();
        assert_eq!(products.len(), 2); // Should have 2 products with stock >= 20
        assert!(products.iter().all(|p| p.stok >= 20));
        
        // Clean up
        clean_test_data().await;
    }
}