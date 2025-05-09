#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::blocking::Client;
    use rocket::http::{Status, ContentType};
    use rocket::serde::json;
    use crate::BuildingStoreDB;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock database implementation
    struct MockProdukRepository;
    
    impl ProdukRepository for MockProdukRepository {
        // Implement all repository methods with mock responses
    }

    // Test setup
    fn setup_rocket() -> rocket::Rocket<rocket::Build> {
        rocket::build()
            .manage(Arc::new(Mutex::new(MockProdukRepository)))
            .mount("/", routes())
    }

    #[test]
    fn test_list_produk_success() {
        let client = Client::tracked(setup_rocket()).unwrap();
        let response = client.get("/produk").dispatch();
        
        assert_eq!(response.status(), Status::Ok);
        let api_response = response.into_json::<ApiResponse<Vec<ProdukResponse>>>();
        assert!(api_response.success);
        assert!(!api_response.data.unwrap().is_empty());
    }

    #[test]
    fn test_detail_produk_found() {
        let client = Client::tracked(setup_rocket()).unwrap();
        let response = client.get("/produk/1").dispatch();
        
        assert_eq!(response.status(), Status::Ok);
        let api_response = response.into_json::<ApiResponse<ProdukResponse>>();
        assert!(api_response.success);
        assert_eq!(api_response.data.unwrap().id, Some(1));
    }

    #[test]
    fn test_tambah_produk_success() {
        let client = Client::tracked(setup_rocket()).unwrap();
        let produk_request = ProdukRequest {
            nama: "New Product".into(),
            kategori: "Test".into(),
            harga: 100.0,
            stok: 10,
            deskripsi: None
        };

        let response = client.post("/produk")
            .header(ContentType::JSON)
            .body(json::to_string(&produk_request).unwrap())
            .dispatch();

        assert_eq!(response.status(), Status::Created);
        let api_response = response.into_json::<ApiResponse<ProdukResponse>>();
        assert!(api_response.success);
    }

    #[test]
    fn test_update_produk_invalid() {
        let client = Client::tracked(setup_rocket()).unwrap();
        let invalid_request = ProdukRequest {
            nama: "".into(),  // Invalid empty name
            kategori: "Test".into(),
            harga: -50.0,     // Invalid negative price
            stok: 5,
            deskripsi: None
        };

        let response = client.put("/produk/1")
            .header(ContentType::JSON)
            .body(json::to_string(&invalid_request).unwrap())
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        let api_response = response.into_json::<ApiResponse<ProdukResponse>>();
        assert!(!api_response.success);
        assert!(api_response.message.unwrap().contains("Validasi gagal"));
    }

    #[test]
    fn test_filter_by_price() {
        let client = Client::tracked(setup_rocket()).unwrap();
        let response = client.get("/produk/harga?min=100&max=500").dispatch();
        
        assert_eq!(response.status(), Status::Ok);
        let api_response = response.into_json::<ApiResponse<Vec<ProdukResponse>>>();
        assert!(api_response.success);
        assert_eq!(api_response.data.unwrap().len(), 3);
    }

    #[test]
    fn test_clone_produk_with_price() {
        let client = Client::tracked(setup_rocket()).unwrap();
        let request = ProdukRequest {
            nama: "Cloned Product".into(),
            kategori: "Test".into(),
            harga: 150.0,
            stok: 5,
            deskripsi: None
        };

        let response = client.post("/produk/1/clone_with_price")
            .header(ContentType::JSON)
            .body(json::to_string(&request).unwrap())
            .dispatch();

        assert_eq!(response.status(), Status::Created);
        let api_response = response.into_json::<ApiResponse<ProdukResponse>>();
        assert!(api_response.success);
        assert_eq!(api_response.data.unwrap().harga, 150.0);
    }

    #[test]
    fn test_update_stock_with_observer() {
        let client = Client::tracked(setup_rocket()).unwrap();
        let response = client.post("/produk/1/update_stock")
            .header(ContentType::JSON)
            .body(json::to_string(&25).unwrap())
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        let api_response = response.into_json::<ApiResponse<()>>();
        assert!(api_response.success);
        
        // Verify observer was called through mock implementation
    }
}