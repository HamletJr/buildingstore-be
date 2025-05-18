use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{get, post, put, delete, routes};
use crate::manajemen_produk::produk::model::{Produk, ProdukBuilder};
use crate::manajemen_produk::produk::repository::{ProdukRepository, RepositoryError};

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProdukRequest {
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: i32,
    pub deskripsi: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProdukResponse {
    pub id: Option<i64>,
    pub nama: String,
    pub kategori: String,
    pub harga: f64,
    pub stok: u32,
    pub deskripsi: Option<String>,
}

impl From<Produk> for ProdukResponse {
    fn from(produk: Produk) -> Self {
        Self {
            id: produk.id,
            nama: produk.nama,
            kategori: produk.kategori,
            harga: produk.harga,
            stok: produk.stok,
            deskripsi: produk.deskripsi,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
}

// Controller functions
#[get("/produk")]
pub async fn list_produk() -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match ProdukRepository::ambil_semua_produk().await {
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
    match ProdukRepository::ambil_produk_by_id(id).await {
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

#[post("/produk", format = "json", data = "<request>")]
pub async fn tambah_produk(
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    // Using Builder Pattern to create a new product
    let produk_result = ProdukBuilder::new(request.nama.clone(), request.kategori.clone())
        .harga(request.harga)
        .stok(request.stok.try_into().unwrap_or(0))
        .deskripsi(request.deskripsi.clone().unwrap_or_default())
        .build();
    
    match produk_result {
        Ok(produk) => {
            match ProdukRepository::tambah_produk(&produk).await {
                Ok(id) => {
                    let produk_with_id = Produk::with_id(
                        id, 
                        produk.nama, 
                        produk.kategori, 
                        produk.harga, 
                        produk.stok, 
                        produk.deskripsi
                    );
                    
                    Json(ApiResponse {
                        success: true,
                        message: Some("Berhasil menambahkan produk baru".to_string()),
                        data: Some(ProdukResponse::from(produk_with_id)),
                    })
                },
                Err(e) => Json(ApiResponse {
                    success: false,
                    message: Some(format!("Gagal menyimpan produk: {}", e)),
                    data: None,
                }),
            }
        },
        Err(e) => Json(ApiResponse {
            success: false,
            message: Some(format!("Validasi gagal: {:?}", e)),
            data: None,
        }),
    }
}

#[put("/produk/<id>", format = "json", data = "<request>")]
pub async fn update_produk(
    id: i64,
    request: Json<ProdukRequest>
) -> Json<ApiResponse<ProdukResponse>> {
    // Check if product exists
    match ProdukRepository::ambil_produk_by_id(id).await {
        Ok(Some(_)) => {
            // Using builder to update product
            let updated_produk = ProdukBuilder::new(request.nama.clone(), request.kategori.clone())
                .id(id)
                .harga(request.harga)
                .stok(request.stok.try_into().unwrap_or(0))
                .deskripsi(request.deskripsi.clone().unwrap_or_default())
                .build();
                
            match updated_produk {
                Ok(updated_produk) => {
                    match ProdukRepository::update_produk(id, &updated_produk).await {
                        Ok(true) => {
                            Json(ApiResponse {
                                success: true,
                                message: Some("Berhasil memperbarui produk".to_string()),
                                data: Some(ProdukResponse::from(updated_produk)),
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
                                message: Some(format!("Gagal memperbarui produk: {}", e)),
                                data: None,
                            })
                        }
                    }
                },
                Err(e) => {
                    Json(ApiResponse {
                        success: false,
                        message: Some(format!("Validasi gagal: {:?}", e)),
                        data: None,
                    })
                }
            }
        },
        Ok(None) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Produk dengan ID {} tidak ditemukan", id)),
                data: None,
            })
        },
        Err(e) => {
            Json(ApiResponse {
                success: false,
                message: Some(format!("Gagal mengambil produk untuk update: {}", e)),
                data: None,
            })
        }
    }
}

#[delete("/produk/<id>")]
pub async fn hapus_produk(
    id: i64
) -> Json<ApiResponse<()>> {
    match ProdukRepository::hapus_produk(id).await {
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

#[get("/produk/kategori/<kategori>")]
pub async fn filter_produk_by_kategori(
    kategori: String
) -> Json<ApiResponse<Vec<ProdukResponse>>> {
    match ProdukRepository::filter_produk_by_kategori(&kategori).await {
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
    match ProdukRepository::filter_produk_by_price_range(min, max).await {
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
    match ProdukRepository::filter_produk_by_stock_availability(min_stok.try_into().unwrap_or(0)).await {
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

// Fungsi untuk mendaftarkan semua routes
pub fn routes() -> Vec<rocket::Route> {
    routes![
        list_produk,
        detail_produk,
        tambah_produk,
        update_produk,
        hapus_produk,
        filter_produk_by_kategori,
        filter_produk_by_price,
        filter_produk_by_stock,
    ]
}