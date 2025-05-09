#![cfg(test)]

use rocket::{local::asynchronous::Client, http::{Status, ContentType}};
use std::sync::Arc;
use chrono::Utc;

use mockall::{automock, predicate::*};

use crate::manajemen_supplier::main::{
    model::supplier::Supplier,
    service::supplier_service::SupplierService,
    controller::supplier_controller::{supplier_routes, SupplierRequest},
};

#[automock]
pub trait SupplierServiceMock: Send + Sync {
    fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String>;
    fn get_supplier(&self, id: &str) -> Option<Supplier>;
    fn update_supplier(&self, supplier: Supplier) -> Result<(), String>;
    fn delete_supplier(&self, id: &str) -> Result<(), String>;
}

impl SupplierService for MockSupplierServiceMock {
    fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String> {
        SupplierServiceMock::save_supplier(self, supplier)
    }

    fn get_supplier(&self, id: &str) -> Option<Supplier> {
        SupplierServiceMock::get_supplier(self, id)
    }

    fn update_supplier(&self, supplier: Supplier) -> Result<(), String> {
        SupplierServiceMock::update_supplier(self, supplier)
    }

    fn delete_supplier(&self, id: &str) -> Result<(), String> {
        SupplierServiceMock::delete_supplier(self, id)
    }
}

fn create_supplier() -> Supplier {
    Supplier {
        id: "test-id".to_string(),
        name: "Test Name".to_string(),
        jenis_barang: "Type".to_string(),
        jumlah_barang: 20,
        resi: "ResiTest".to_string(),
        updated_at: Utc::now(),
    }
}

#[rocket::async_test]
async fn test_get_supplier_found() {
    let mut mock = MockSupplierServiceMock::new();
    let supplier = create_supplier();

    mock.expect_get_supplier()
        .with(eq("test-id"))
        .return_const(Some(supplier.clone()));

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.get("/suppliers/test-id").dispatch().await;
    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.unwrap();
    assert!(body.contains("Supplier found"));
    assert!(body.contains("Test Name"));
}

#[rocket::async_test]
async fn test_get_supplier_not_found() {
    let mut mock = MockSupplierServiceMock::new();

    mock.expect_get_supplier()
        .with(eq("unknown-id"))
        .return_const(None);

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.get("/suppliers/unknown-id").dispatch().await;
    assert_eq!(response.status(), Status::NotFound);

    let body = response.into_string().await.unwrap();
    assert!(body.contains("not found"));
}

#[rocket::async_test]
async fn test_save_supplier_success() {
    let mut mock = MockSupplierServiceMock::new();

    mock.expect_save_supplier()
        .returning(|sup| Ok(sup));

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let req = SupplierRequest {
        name: "New".to_string(),
        jenis_barang: "Type".to_string(),
        jumlah_barang: 10,
        resi: "123".to_string(),
    };

    let response = client.post("/suppliers")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&req).unwrap())
        .dispatch().await;

    assert_eq!(response.status(), Status::Created);
    let body = response.into_string().await.unwrap();
    assert!(body.contains("Supplier created successfully"));
}

#[rocket::async_test]
async fn test_update_supplier_success() {
    let mut mock = MockSupplierServiceMock::new();
    let supplier = create_supplier();

    mock.expect_get_supplier()
        .with(eq("test-id"))
        .return_const(Some(supplier.clone()));
    mock.expect_update_supplier()
        .returning(|_| Ok(()));

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let req = SupplierRequest {
        name: supplier.name.clone(),
        jenis_barang: supplier.jenis_barang.clone(),
        jumlah_barang: supplier.jumlah_barang,
        resi: supplier.resi.clone(),
    };

    let response = client.put("/suppliers/test-id")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&req).unwrap())
        .dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap();
    assert!(body.contains("Supplier updated successfully"));
}

#[rocket::async_test]
async fn test_delete_supplier_success() {

    let mut mock = MockSupplierServiceMock::new();

    mock.expect_delete_supplier()
        .with(eq("test-id"))
        .returning(|_| Ok(()));

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.delete("/suppliers/test-id").dispatch().await;
    assert_eq!(response.status(), Status::NoContent);
}

#[rocket::async_test]
async fn test_save_supplier_failure() {
    let mut mock = MockSupplierServiceMock::new();

    mock.expect_save_supplier()
        .returning(|_| Err("Failed to save".to_string()));

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let req = SupplierRequest {
        name: "New".to_string(),
        jenis_barang: "Type".to_string(),
        jumlah_barang: 10,
        resi: "123".to_string(),
    };

    let response = client.post("/suppliers")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&req).unwrap())
        .dispatch().await;

    // Your controller returns 200 OK with an error JSON body
    assert_eq!(response.status(), Status::Ok);

    let body = response.into_string().await.unwrap();
    assert!(body.contains("Failed to create supplier"));
    assert!(body.contains("\"success\":false"));
}

#[rocket::async_test]
async fn test_update_supplier_not_found() {
    let mut mock = MockSupplierServiceMock::new();

    mock.expect_update_supplier()
        .returning(|_| Err("No such supplier".to_string()));

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let req = SupplierRequest {
        name: "None".to_string(),
        jenis_barang: "None".to_string(),
        jumlah_barang: 0,
        resi: "".to_string(),
    };

    let response = client.put("/suppliers/missing-id")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&req).unwrap())
        .dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap();
    assert!(body.contains("Failed to update supplier: No such supplier"));
    assert!(body.contains("\"success\":false"));
}

#[rocket::async_test]
async fn test_update_supplier_get_updated_failed() {
    let mut mock = MockSupplierServiceMock::new();

    mock.expect_update_supplier()
        .returning(|_| Ok(()));
    mock.expect_get_supplier()
        .with(eq("test-id"))
        .return_const(None);

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let req = SupplierRequest {
        name: "Test".to_string(),
        jenis_barang: "Type".to_string(),
        jumlah_barang: 10,
        resi: "A".to_string(),
    };

    let response = client.put("/suppliers/test-id")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&req).unwrap())
        .dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap();
    assert!(body.contains("Failed to fetch updated supplier"));
    assert!(body.contains("\"success\":false"));
}

#[rocket::async_test]
async fn test_update_supplier_failure() {
    let mut mock = MockSupplierServiceMock::new();
    let supplier = create_supplier();

    mock.expect_update_supplier()
        .returning(|_| Err("Failed update".to_string()));

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let req = SupplierRequest {
        name: supplier.name.clone(),
        jenis_barang: supplier.jenis_barang.clone(),
        jumlah_barang: supplier.jumlah_barang,
        resi: supplier.resi.clone(),
    };

    let response = client.put("/suppliers/test-id")
        .header(ContentType::JSON)
        .body(serde_json::to_string(&req).unwrap())
        .dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    let body = response.into_string().await.unwrap();
    assert!(body.contains("Failed to update supplier: Failed update"));
    assert!(body.contains("\"success\":false"));
}

#[rocket::async_test]
async fn test_delete_supplier_not_found() {
    let mut mock = MockSupplierServiceMock::new();

    mock.expect_delete_supplier()
        .with(eq("missing-id"))
        .returning(|_| Err("Supplier not found".to_string()));

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.delete("/suppliers/missing-id").dispatch().await;

    assert_eq!(response.status(), Status::NotFound);
    let body = response.into_string().await.unwrap();
    assert!(body.contains("Failed to delete supplier"));
    assert!(body.contains("\"success\":false"));
}

#[rocket::async_test]
async fn test_malformed_post_request() {
    let mock = MockSupplierServiceMock::new();

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.post("/suppliers")
        .header(ContentType::JSON)
        .body("{bad json")
        .dispatch().await;

    assert_eq!(response.status(), Status::BadRequest);
}

#[rocket::async_test]
async fn test_malformed_put_request() {
    let mock = MockSupplierServiceMock::new();

    let service: Arc<dyn SupplierService> = Arc::new(mock);
    let rocket = rocket::build().manage(service).mount("/", supplier_routes());
    let client = Client::tracked(rocket).await.unwrap();

    let response = client.put("/suppliers/test-id")
        .header(ContentType::JSON)
        .body("{bad json")
        .dispatch().await;

    assert_eq!(response.status(), Status::BadRequest);
}