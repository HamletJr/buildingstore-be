use rocket::serde::json::Json;
use rocket::{post, routes, Route};
use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::repository;
use super::dto::{ProdukRequest, ProdukResponse, ApiResponse};

#[post("/produk", format = "json", data = "<request>")]
pub async fn tambah_produk(
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    let produk = Produk::new(
        request.nama.clone(),
        request.kategori.clone(),
        request.harga,
        request.stok.try_into().unwrap_or(0),
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
                _ => {
                    Json(ApiResponse {
                        success: false,
                        message: Some("Produk berhasil dibuat tetapi gagal mengambil data".to_string()),
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

#[post("/produk/batch", format = "json", data = "<requests>")]
pub async fn tambah_batch_produk(
    requests: Json<Vec<ProdukRequest>>
) -> Json<ApiResponse<Vec<ProdukResponse>>> {
    let produk_list: Vec<Produk> = requests.iter()
        .map(|req| Produk::new(
            req.nama.clone(),
            req.kategori.clone(),
            req.harga,
            req.stok.try_into().unwrap_or(0),
            req.deskripsi.clone(),
        ))
        .collect();

    match repository::create::tambah_batch_produk(&produk_list).await {
        Ok(ids) => {
            // Ambil semua produk yang baru dibuat
            let mut response_list = Vec::new();
            for id in ids {
                if let Ok(Some(produk)) = repository::read::ambil_produk_by_id(id).await {
                    response_list.push(ProdukResponse::from(produk));
                }
            }

            Json(ApiResponse {
                success: true,
                message: Some(format!("Berhasil menambahkan {} produk", response_list.len())),
                data: Some(response_list),
            })
        },
        Err(e) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Gagal menambahkan batch produk: {}", e)),
                data: None,
            })
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![tambah_produk, tambah_batch_produk]
}

#[cfg(test)]
mod tests {
    use rocket::http::{ContentType, Status};
    use rocket::local::asynchronous::Client;
    use rocket::serde::json::json;
    use crate::manajemen_produk::produk::controller::{ApiResponse, ProdukResponse, routes};

    async fn setup_test_client() -> Client {
        let rocket = rocket::build().mount("/api", routes());
        Client::tracked(rocket).await.expect("valid rocket instance")
    }

    async fn clean_test_data() {
        let _ = crate::manajemen_produk::produk::repository::delete::clear_all().await;
    }

    #[tokio::test]
    async fn test_tambah_produk() {
        let client = setup_test_client().await;
        
        // Clean up first
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
        assert_eq!(produk.kategori, "Elektronik");
        assert_eq!(produk.harga, 10_000_000.0);
        assert_eq!(produk.stok, 5);
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_tambah_batch_produk() {
        let client = setup_test_client().await;
        
        // Clean up first
        clean_test_data().await;
        
        let response = client.post("/api/produk/batch")
            .header(ContentType::JSON)
            .body(json!([
                {
                    "nama": "Laptop 1",
                    "kategori": "Elektronik",
                    "harga": 10_000_000.0,
                    "stok": 5,
                    "deskripsi": "Laptop description 1"
                },
                {
                    "nama": "Laptop 2",
                    "kategori": "Elektronik",
                    "harga": 12_000_000.0,
                    "stok": 3,
                    "deskripsi": "Laptop description 2"
                }
            ]).to_string())
            .dispatch()
            .await;
        
        assert_eq!(response.status(), Status::Ok);
        
        let body = response.into_string().await.unwrap();
        let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
        
        assert!(json.success);
        let products = json.data.unwrap();
        assert_eq!(products.len(), 2);
        
        // Clean up
        clean_test_data().await;
    }

    #[tokio::test]
    async fn test_tambah_produk_invalid_data() {
        let client = setup_test_client().await;
        
        let response = client.post("/api/produk")
            .header(ContentType::JSON)
            .body(json!({
                "nama": "",
                "kategori": "Elektronik",
                "harga": -1000.0,
                "stok": 5,
                "deskripsi": "Invalid product"
            }).to_string())
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
}