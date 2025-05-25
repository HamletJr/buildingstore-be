use rocket::serde::json::Json;
use rocket::{get, routes, Route};
use crate::manajemen_produk::repository;
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
    use crate::manajemen_produk::controller::dto::{ApiResponse, ProdukResponse};
    use crate::manajemen_produk::model::Produk;
    use crate::manajemen_produk::repository;

    async fn setup_test_client() -> Client {
        let _ = repository::dto::init_database().await;
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
        
        let _ = repository::create::tambah_produk(&produk1).await;
    }

    async fn clean_test_data() {
        let _ = repository::delete::clear_all().await;
    }

    #[tokio::test]
    async fn test_list_produk() {
        let client = setup_test_client().await;
        clean_test_data().await;
        seed_test_data().await;
        
        let response = client.get("/api/produk").dispatch().await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        assert!(json.data.unwrap().len() > 0);
        
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_detail_produk() {
        let client = setup_test_client().await;
        clean_test_data().await;
        seed_test_data().await;
        
        // Get first product ID
        let response = client.get("/api/produk").dispatch().await;
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        let produk_id = json.data.unwrap()[0].id.unwrap();
        
        // Test detail endpoint
        let response = client.get(format!("/api/produk/{}", produk_id)).dispatch().await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        assert_eq!(json.data.unwrap().id.unwrap(), produk_id);
        
        clean_test_data().await;
    }
}