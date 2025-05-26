use rocket::serde::json::Json;
use rocket::{delete, routes, Route};
use crate::manajemen_produk::repository;
use super::dto::ApiResponse;
use autometrics::autometrics;

#[autometrics]
#[delete("/produk/<id>")]
pub async fn hapus_produk(
    id: i64
) -> Json<ApiResponse<()>> {
    match repository::delete::hapus_produk(id).await {
        Ok(true) => {
            Json(ApiResponse {
                success: true,
                message: Some(format!("Produk dengan ID {} berhasil dihapus", id)),
                data: None,
            })
        },
        Ok(false) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
                data: None,
            })
        },
        Err(e) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Gagal menghapus produk: {}", e)),
                data: None,
            })
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![hapus_produk]
}

#[cfg(test)]
mod tests {
    use rocket::http::Status;
    use rocket::local::asynchronous::Client;
    use crate::manajemen_produk::controller::{ApiResponse, routes};
    use crate::manajemen_produk::model::Produk;
    use crate::manajemen_produk::repository;

    async fn setup_test_client() -> Client {
        let rocket = rocket::build().mount("/api", routes());
        Client::tracked(rocket).await.expect("valid rocket instance")
    }

    async fn seed_test_data() -> i64 {
        let produk1 = Produk::new(
            "Laptop Gaming".to_string(),
            "Elektronik".to_string(),
            15_000_000.0,
            10,
            Some("Laptop dengan RTX 4060".to_string()),
        );
        
        repository::create::tambah_produk(&produk1).await.unwrap()
    }

    async fn clean_test_data() {
        let _ = repository::delete::clear_all().await;
    }

    #[tokio::test]
    async fn test_hapus_produk() {
        let client = setup_test_client().await;
        let produk_id = seed_test_data().await;
        
        let response = client.delete(format!("/api/produk/{}", produk_id))
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<()> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        
        clean_test_data().await;
    }
}