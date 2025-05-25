use rocket::serde::json::Json;
use rocket::{post, routes, Route};
use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::repository;
use super::dto::{ProdukRequest, ProdukResponse, ApiResponse};

#[post("/produk", format = "json", data = "<request>")]
pub async fn tambah_produk(
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    // Validasi stok tidak boleh negatif
    let stok = if request.stok < 0 { 0 } else { request.stok as u32 };
    
    let produk = Produk::new(
        request.nama.clone(),
        request.kategori.clone(),
        request.harga,
        stok,
        request.deskripsi.clone(),
    );

    match repository::create::tambah_produk(&produk).await {
        Ok(id) => {
            // Ambil produk yang baru dibuat untuk response
            match repository::read::ambil_produk_by_id(id).await {
                Ok(Some(created_produk)) => {
                    Json(ApiResponse {
                        success: true,
                        message: Some("Berhasil menambahkan produk".to_string()),
                        data: Some(ProdukResponse::from(created_produk)),
                    })
                },
                Ok(None) => {
                    Json(ApiResponse {
                        success: false,
                        message: Some("Produk berhasil dibuat tetapi tidak ditemukan".to_string()),
                        data: None,
                    })
                },
                Err(e) => {
                    Json(ApiResponse {
                        success: false,
                        message: Some(format!("Produk berhasil dibuat tetapi gagal mengambil data: {}", e)),
                        data: None,
                    })
                }
            }
        },
        Err(e) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Gagal menambahkan produk: {}", e)),
                data: None,
            })
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![tambah_produk]
}

#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status};
    use rocket::local::asynchronous::Client;
    use rocket::serde::json::json;
    use serde_json;
    use crate::manajemen_produk::produk::controller::{ApiResponse, ProdukResponse};
    use crate::manajemen_produk::produk::repository::dto::init_database;

    async fn setup_test_client() -> Client {
        let _ = init_database().await;
        let rocket = rocket::build().mount("/api", super::routes());
        Client::tracked(rocket).await.expect("valid rocket instance")
    }

    async fn clean_test_data() {
        let _ = init_database().await;
        let _ = crate::manajemen_produk::produk::repository::delete::clear_all().await;
    }

    #[tokio::test]
    async fn test_tambah_produk() {
        let client = setup_test_client().await;
        clean_test_data().await;
        
        let response = client.post("/api/produk")
            .header(ContentType::JSON)
            .body(json!({
                "nama": "Test Laptop",
                "kategori": "Elektronik",
                "harga": 10_000_000.0,
                "stok": 5,
                "deskripsi": "Test laptop description"
            }).to_string())
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        let produk = json.data.unwrap();
        assert_eq!(produk.nama, "Test Laptop");
        assert_eq!(produk.stok, 5);
        
        clean_test_data().await;
    }
}