use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use rocket::serde::json::{json, Value};
use crate::manajemen_produk::produk::controller::{
    ApiResponse, ProdukResponse, routes
};
use crate::manajemen_produk::produk::model::Produk;
use crate::manajemen_produk::produk::repository::ProdukRepository;

// Helper function to setup the test environment
async fn setup_test_client() -> Client {
    // Setup Rocket client with our routes
    let rocket = rocket::build().mount("/api", routes());
    Client::tracked(rocket).await.expect("valid rocket instance")
}

// Helper function to seed test data
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
    
    // Add products to the repository
    let _ = ProdukRepository::tambah_produk(&produk1).await;
    let _ = ProdukRepository::tambah_produk(&produk2).await;
    let _ = ProdukRepository::tambah_produk(&produk3).await;
}

// Clean up test data after each test
async fn clean_test_data() {
    let _ = ProdukRepository::clear_all().await;
}

#[tokio::test]
async fn test_list_produk() {
    let client = setup_test_client().await;
    
    // Seed test data
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
    let client = setup_test_client().await;
    
    // Seed test data
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
    
    // Test with non-existent ID
    let response = client.get("/api/produk/9999")
        .dispatch()
        .await;
    
    assert_eq!(response.status(), Status::Ok); // API returns 200 with error message
    
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
    
    assert!(!json.success);
    assert!(json.data.is_none());
    
    // Clean up
    clean_test_data().await;
}

#[tokio::test]
async fn test_tambah_produk() {
    let client = setup_test_client().await;
    
    // Test adding a new product
    let response = client.post("/api/produk")
        .header(ContentType::JSON)
        .body(json!({
            "nama": "Monitor Gaming",
            "kategori": "Elektronik",
            "harga": 5_000_000.0,
            "stok": 15,
            "deskripsi": "Monitor 144Hz"
        }).to_string())
        .dispatch()
        .await;
    
    assert_eq!(response.status(), Status::Ok);
    
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
    
    assert!(json.success);
    let produk = json.data.unwrap();
    assert_eq!(produk.nama, "Monitor Gaming");
    assert_eq!(produk.kategori, "Elektronik");
    assert_eq!(produk.harga, 5_000_000.0);
    assert_eq!(produk.stok, 15);
    assert_eq!(produk.deskripsi, Some("Monitor 144Hz".to_string()));
    
    // Test validation failure
    let response = client.post("/api/produk")
        .header(ContentType::JSON)
        .body(json!({
            "nama": "",
            "kategori": "Elektronik",
            "harga": 5_000_000.0,
            "stok": 15,
            "deskripsi": "Monitor 144Hz"
        }).to_string())
        .dispatch()
        .await;
    
    assert_eq!(response.status(), Status::Ok); // API returns 200 with error message
    
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
    
    assert!(!json.success);
    assert!(json.data.is_none());
    
    // Clean up
    clean_test_data().await;
}

#[tokio::test]
async fn test_update_produk() {
    let client = setup_test_client().await;
    
    // Seed test data
    seed_test_data().await;
    
    // Get all products first to find an ID
    let response = client.get("/api/produk").dispatch().await;
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
    let data = json.data.as_ref().unwrap();
    let produk_id = data[0].id.unwrap();
    
    // Test update
    let response = client.put(format!("/api/produk/{}", produk_id))
        .header(ContentType::JSON)
        .body(json!({
            "nama": "Updated Laptop Gaming",
            "kategori": "Elektronik",
            "harga": 16_000_000.0,
            "stok": 8,
            "deskripsi": "Updated description"
        }).to_string())
        .dispatch()
        .await;
    
    assert_eq!(response.status(), Status::Ok);
    
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
    
    assert!(json.success);
    let produk = json.data.unwrap();
    assert_eq!(produk.nama, "Updated Laptop Gaming");
    assert_eq!(produk.harga, 16_000_000.0);
    assert_eq!(produk.stok, 8);
    assert_eq!(produk.deskripsi, Some("Updated description".to_string()));
    
    // Test update with non-existent ID
    let response = client.put("/api/produk/9999")
        .header(ContentType::JSON)
        .body(json!({
            "nama": "Non-existent Product",
            "kategori": "Elektronik",
            "harga": 1000.0,
            "stok": 10,
            "deskripsi": "This should fail"
        }).to_string())
        .dispatch()
        .await;
    
    assert_eq!(response.status(), Status::Ok); // API returns 200 with error message
    
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
    
    assert!(!json.success);
    assert!(json.data.is_none());
    
    // Clean up
    clean_test_data().await;
}

#[tokio::test]
async fn test_hapus_produk() {
    let client = setup_test_client().await;
    
    // Seed test data
    seed_test_data().await;
    
    // Get all products first to find an ID
    let response = client.get("/api/produk").dispatch().await;
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
    let data = json.data.as_ref().unwrap();
    let produk_id = data[0].id.unwrap();
    
    // Test deletion
    let response = client.delete(format!("/api/produk/{}", produk_id))
        .dispatch()
        .await;
    
    assert_eq!(response.status(), Status::Ok);
    
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<()> = serde_json::from_str(&body).unwrap();
    
    assert!(json.success);
    
    // Verify the product is removed
    let response = client.get(format!("/api/produk/{}", produk_id))
        .dispatch()
        .await;
    
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<ProdukResponse> = serde_json::from_str(&body).unwrap();
    
    assert!(!json.success);
    assert!(json.data.is_none());
    
    // Test delete with non-existent ID
    let response = client.delete("/api/produk/9999")
        .dispatch()
        .await;
    
    assert_eq!(response.status(), Status::Ok); // API returns 200 with error message
    
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<()> = serde_json::from_str(&body).unwrap();
    
    assert!(!json.success);
    
    // Clean up
    clean_test_data().await;
}

#[tokio::test]
async fn test_filter_produk_by_kategori() {
    let client = setup_test_client().await;
    
    // Seed test data
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
    
    // Seed test data
    seed_test_data().await;
    
    // Test filter by price range
    let response = client.get("/api/produk/harga?min=7000000&max=16000000")
        .dispatch()
        .await;
    
    assert_eq!(response.status(), Status::Ok);
    
    let body = response.into_string().await.unwrap();
    let json: ApiResponse<Vec<ProdukResponse>> = serde_json::from_str(&body).unwrap();
    
    assert!(json.success);
    let products = json.data.unwrap();
    assert_eq!(products.len(), 2); // Should have 2 products in this price range
    assert!(products.iter().all(|p| p.harga >= 7000000.0 && p.harga <= 16000000.0));
    
    // Clean up
    clean_test_data().await;
}

#[tokio::test]
async fn test_filter_produk_by_stock() {
    let client = setup_test_client().await;
    
    // Seed test data
    seed_test_data().await;
    
    // Test filter by minimum stock
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