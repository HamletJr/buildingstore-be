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
        let rocket = rocket::build().mount("/api", super::super::routes());
        Client::tracked(rocket).await.expect("valid rocket instance")
    }

    async fn seed_test_data() {
        clean_test_data().await;
        
        let produk1 = Produk::new(
            "Laptop Gaming".to_string(),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some("Laptop dengan RTX 4060".to_string()),
        );
        
        let produk2 = Produk::new(
            "Smartphone".to_string(),
            "Elektronik".to_string(),
            8_000_000.0,
            20,
            Some("Smartphone dengan kamera 108MP".to_string()),
        );
        
        let _ = repository::create::tambah_produk(&produk1).await;
        let _ = repository::create::tambah_produk(&produk2).await;
    }

    async fn clean_test_data() {
        let _ = repository::delete::clear_all().await;
    }

    #[tokio::test]
    async fn test_filter_produk_by_kategori() {
        let client = setup_test_client().await;
        seed_test_data().await;
        
        let response = client.get("/api/produk/kategori/Elektronik")
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        let products = json.data.unwrap();
        assert_eq!(products.len(), 2);
        assert!(products.iter().all(|p| p.kategori == "Elektronik"));
        
        clean_test_data().await;
    }
}