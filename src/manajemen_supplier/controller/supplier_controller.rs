use rocket::{State, get, put, delete, post, routes};
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::response::status::{Created, NotFound, NoContent};
use std::sync::Arc;

use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::service::supplier_service::SupplierService;

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SupplierRequest {
    pub name: String,
    pub jenis_barang: String,
    pub jumlah_barang: i32,
    pub resi: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SupplierResponse {
    pub id: String,
    pub name: String,
    pub jenis_barang: String,
    pub jumlah_barang: i32,
    pub resi: String,
    pub updated_at: String,
}

impl From<Supplier> for SupplierResponse {
    fn from(supplier: Supplier) -> Self {
        Self {
            id: supplier.id,
            name: supplier.name,
            jenis_barang: supplier.jenis_barang,
            jumlah_barang: supplier.jumlah_barang,
            resi: supplier.resi,
            updated_at: supplier.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}


#[post("/suppliers", format = "json", data = "<request>")]
pub async fn save_supplier(
    request: Json<SupplierRequest>,
    service: &State<Arc<dyn SupplierService>>,
) -> Result<Created<Json<ApiResponse<SupplierResponse>>>, Json<ApiResponse<String>>> {
    let supplier = Supplier {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name.clone(),
        jenis_barang: request.jenis_barang.clone(),
        jumlah_barang: request.jumlah_barang,
        resi: request.resi.clone(),
        updated_at: chrono::Utc::now(),
    };

    match service.save_supplier(supplier).await {
        Ok(saved) => {
            let response = ApiResponse {
                success: true,
                message: "Supplier created successfully".to_string(),
                data: Some(SupplierResponse::from(saved)),
            };
            Ok(Created::new("/suppliers").body(Json(response)))
        }
        Err(e) => Err(Json(ApiResponse {
            success: false,
            message: format!("Failed to create supplier: {}", e),
            data: None,
        })),
    }
}

#[get("/suppliers/<id>")]
pub async fn get_supplier(
    id: String,
    service: &State<Arc<dyn SupplierService>>,
) -> Result<Json<ApiResponse<SupplierResponse>>, NotFound<Json<ApiResponse<String>>>> {
    match service.get_supplier(&id).await {
        Some(supplier) => Ok(Json(ApiResponse {
            success: true,
            message: "Supplier found".to_string(),
            data: Some(SupplierResponse::from(supplier)),
        })),
        None => Err(NotFound(Json(ApiResponse {
            success: false,
            message: format!("Supplier with id {} not found", id),
            data: None,
        }))),
    }
}

#[put("/suppliers/<id>", format = "json", data = "<request>")]
pub async fn update_supplier(
    service: &State<Arc<dyn SupplierService>>,
    id: String,
    request: Json<SupplierRequest>,
) -> Result<Json<ApiResponse<SupplierResponse>>, Json<ApiResponse<String>>> {
    let supplier = Supplier {
        id: id.clone(),
        name: request.name.clone(),
        jenis_barang: request.jenis_barang.clone(),
        jumlah_barang: request.jumlah_barang,
        resi: request.resi.clone(),
        updated_at: chrono::Utc::now(),
    };

    match service.update_supplier(supplier).await {
        Ok(()) => {
            if let Some(updated) = service.get_supplier(&id).await {
                Ok(Json(ApiResponse {
                    success: true,
                    message: "Supplier updated successfully".to_string(),
                    data: Some(SupplierResponse::from(updated)),
                }))
            } else {
                Err(Json(ApiResponse {
                    success: false,
                    message: "Failed to fetch updated supplier after update".to_string(),
                    data: None,
                }))
            }
        }
        Err(e) => Err(Json(ApiResponse {
            success: false,
            message: format!("Failed to update supplier: {}", e),
            data: None,
        })),
    }
}

#[delete("/suppliers/<id>")]
pub async fn delete_supplier(
    id: String,
    service: &State<Arc<dyn SupplierService>>,
) -> Result<NoContent, NotFound<Json<ApiResponse<String>>>> {
    match service.delete_supplier(&id).await {
        Ok(()) => Ok(NoContent),
        Err(e) => Err(NotFound(Json(ApiResponse {
            success: false,
            message: format!("Failed to delete supplier: {}", e),
            data: None,
        }))),
    }
}

pub fn supplier_routes() -> Vec<rocket::Route> {
    routes![
        save_supplier,
        get_supplier,
        update_supplier,
        delete_supplier,
    ]
}


#[cfg(test)]
mod tests {
    use super::*; // Imports items from the parent module (supplier_controller)
    use rocket::{local::asynchronous::Client, http::{Status, ContentType}, async_test};
    use async_trait::async_trait;
    use std::sync::Arc;
    use chrono::Utc;
    use mockall::{automock, predicate::*};

    use crate::manajemen_supplier::model::supplier::Supplier;
    use crate::manajemen_supplier::service::supplier_service::SupplierService;

    #[automock]
    pub trait SupplierServiceMock: Send + Sync {
        async fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String>;
        async fn get_supplier(&self, id: &str) -> Option<Supplier>;
        async fn update_supplier(&self, supplier: Supplier) -> Result<(), String>;
        async fn delete_supplier(&self, id: &str) -> Result<(), String>;
    }
    #[async_trait]
    impl SupplierService for MockSupplierServiceMock {
        async fn save_supplier(&self, supplier: Supplier) -> Result<Supplier, String> {
            SupplierServiceMock::save_supplier(self, supplier).await
        }

        async fn get_supplier(&self, id: &str) -> Option<Supplier> {
            SupplierServiceMock::get_supplier(self, id).await
        }

        async fn update_supplier(&self, supplier: Supplier) -> Result<(), String> {
            SupplierServiceMock::update_supplier(self, supplier).await
        }

        async fn delete_supplier(&self, id: &str) -> Result<(), String> {
            SupplierServiceMock::delete_supplier(self, id).await
        }
    }

    impl super::SupplierRequest {
        pub fn new_test_helper(name: &str, jenis_barang: &str, jumlah_barang: i32, resi: &str) -> Self {
            Self {
                name: name.to_string(),
                jenis_barang: jenis_barang.to_string(),
                jumlah_barang,
                resi: resi.to_string(),
            }
        }
    }

    fn create_supplier_sample() -> Supplier {
        Supplier {
            id: "test-id".to_string(),
            name: "Test Name".to_string(),
            jenis_barang: "Type".to_string(),
            jumlah_barang: 20,
            resi: "ResiTest".to_string(),
            updated_at: Utc::now(),
        }
    }

    async fn build_test_client_with_mock(
        setup_mock: impl FnOnce(&mut MockSupplierServiceMock) // Uses the mock defined in this module
    ) -> Client {
        let mut mock = MockSupplierServiceMock::new();
        setup_mock(&mut mock);
        let service: Arc<dyn SupplierService> = Arc::new(mock);
        let rocket = rocket::build()
            .manage(service)
            .mount("/", routes![
                super::save_supplier,
                super::get_supplier,
                super::update_supplier,
                super::delete_supplier
            ]);
        Client::tracked(rocket).await.unwrap()
    }

    async fn post_json<'a>(client: &'a Client, url: &'a str, req: &'a super::SupplierRequest) -> rocket::local::asynchronous::LocalResponse<'a> {
        client.post(url)
            .header(ContentType::JSON)
            .body(serde_json::to_string(req).unwrap())
            .dispatch().await
    }

    async fn put_json<'a>(client: &'a Client, url: &'a str, req: &'a super::SupplierRequest) -> rocket::local::asynchronous::LocalResponse<'a> {
        client.put(url)
            .header(ContentType::JSON)
            .body(serde_json::to_string(req).unwrap())
            .dispatch().await
    }

    fn assert_failure_response(res: &str, message_substr: &str) {
        assert!(res.contains(message_substr), "Response did not contain expected error substring '{}'. Response was: {}", message_substr, res);
        assert!(res.contains("\"success\":false"), "Response did not indicate failure. Response was: {}", res);
    }

    #[async_test]
    async fn test_get_supplier_found() {
        let supplier = create_supplier_sample();

        let client = build_test_client_with_mock(|mock| {
            mock.expect_get_supplier()
                .with(eq("test-id"))
                .times(1)
                .return_once({ 
                    let supplier_clone = supplier.clone();
                    move |_| Some(supplier_clone)
                });
        }).await;

        let resp = client.get("/suppliers/test-id").dispatch().await;
        assert_eq!(resp.status(), Status::Ok);

        let body = resp.into_string().await.unwrap();
        assert!(body.contains("Supplier found"));
        assert!(body.contains(&supplier.name));
    }

    #[async_test]
    async fn test_get_supplier_not_found() {
        let client = build_test_client_with_mock(|mock| {
            mock.expect_get_supplier()
                .with(eq("unknown-id"))
                .times(1)
                .return_once(|_| None);
        }).await;

        let resp = client.get("/suppliers/unknown-id").dispatch().await;
        assert_eq!(resp.status(), Status::NotFound);
        let body = resp.into_string().await.unwrap();
        assert!(body.contains("not found"));
    }

    #[async_test]
    async fn test_save_supplier_success() {
        let expected_id = "new-uuid-from-mock"; 
        let mut supplier_to_save = create_supplier_sample();
        supplier_to_save.id = "".to_string(); 

        let saved_supplier_mock_response = Supplier {
            id: expected_id.to_string(), 
            name: "New".to_string(),
            jenis_barang: "Type".to_string(),
            jumlah_barang: 10,
            resi: "123".to_string(),
            updated_at: Utc::now(),
        };

        let client = build_test_client_with_mock(|mock| {
            mock.expect_save_supplier()
                .withf(|sup: &Supplier| sup.name == "New" && sup.id != "") 
                .times(1)
                .returning({
                    let saved = saved_supplier_mock_response.clone();
                    move |_sup_arg| Ok(saved.clone())
                });
        }).await;

        let req = super::SupplierRequest::new_test_helper("New", "Type", 10, "123");
        let resp = post_json(&client, "/suppliers", &req).await;

        assert_eq!(resp.status(), Status::Created);
        let body = resp.into_string().await.unwrap();
        assert!(body.contains("Supplier created successfully"));
        assert!(body.contains(expected_id));
    }

    #[async_test]
    async fn test_update_supplier_success() {
        let supplier_id = "test-id";
        let existing_supplier = Supplier { id: supplier_id.to_string(), ..create_supplier_sample()};
        let updated_supplier_from_get = Supplier { name: "Updated Name".to_string(), ..existing_supplier.clone() };


        let client = build_test_client_with_mock(|mock| {
            mock.expect_update_supplier()
                .withf(move |s: &Supplier| s.id == supplier_id && s.name == "Updated Name")
                .times(1)
                .returning(|_| Ok(()));

            mock.expect_get_supplier()
                .with(eq(supplier_id))
                .times(1)
                .return_once({
                    let s = updated_supplier_from_get.clone();
                    move |_| Some(s)
                });
        }).await;

        let req = super::SupplierRequest::new_test_helper("Updated Name", &existing_supplier.jenis_barang, existing_supplier.jumlah_barang, &existing_supplier.resi);
        let url = format!("/suppliers/{}", supplier_id);
        let resp = put_json(&client, &url, &req).await;

        assert_eq!(resp.status(), Status::Ok);
        let body = resp.into_string().await.unwrap();
        assert!(body.contains("Supplier updated successfully"));
        assert!(body.contains("Updated Name"));
    }

    #[async_test]
    async fn test_delete_supplier_success() {
        let client = build_test_client_with_mock(|mock| {
            mock.expect_delete_supplier()
                .with(eq("test-id"))
                .times(1)
                .returning(|_| Ok(()));
        }).await;

        let resp = client.delete("/suppliers/test-id").dispatch().await;
        assert_eq!(resp.status(), Status::NoContent);
    }

    #[async_test]
    async fn test_save_supplier_failure() {
        let client = build_test_client_with_mock(|mock| {
            mock.expect_save_supplier()
                .times(1)
                .returning(|_| Err("Failed to save from mock".to_string()));
        }).await;

        let req = super::SupplierRequest::new_test_helper("New", "Type", 10, "123");
        let resp = post_json(&client, "/suppliers", &req).await;

        assert_eq!(resp.status(), Status::Ok);
        let body = resp.into_string().await.unwrap();
        assert_failure_response(&body, "Failed to create supplier: Failed to save from mock");
    }
    
    #[async_test]
    async fn test_update_supplier_service_error() {
        let client = build_test_client_with_mock(|mock| {
            mock.expect_update_supplier()
                .withf(|s: &Supplier| s.id == "missing-id")
                .times(1)
                .returning(|_| Err("No such supplier from mock".to_string()));
        }).await;
    
        let req = super::SupplierRequest::new_test_helper("None", "None", 0, "");
        let resp = put_json(&client, "/suppliers/missing-id", &req).await;
    
        assert_eq!(resp.status(), Status::Ok); 
        let body = resp.into_string().await.unwrap();
        assert_failure_response(&body, "Failed to update supplier: No such supplier from mock");
    }

    #[async_test]
    async fn test_update_supplier_get_updated_failed() {
        let supplier_id = "test-id";
        let client = build_test_client_with_mock(|mock| {
            mock.expect_update_supplier()
                .withf(move |s: &Supplier| s.id == supplier_id)
                .times(1)
                .returning(|_| Ok(()));
            mock.expect_get_supplier()
                .with(eq(supplier_id))
                .times(1)
                .return_once(|_| None);
        }).await;

        let req = super::SupplierRequest::new_test_helper("Test", "Type", 10, "A");
        let url = format!("/suppliers/{}", supplier_id);
        let resp = put_json(&client, &url, &req).await;

        assert_eq!(resp.status(), Status::Ok); 
        let body = resp.into_string().await.unwrap();
        assert_failure_response(&body, "Failed to fetch updated supplier after update");
    }

    #[async_test]
    async fn test_delete_supplier_not_found_or_error() {
        let client = build_test_client_with_mock(|mock| {
            mock.expect_delete_supplier()
                .with(eq("missing-id"))
                .times(1)
                .returning(|_| Err("Supplier not found or deletion error from mock".to_string()));
        }).await;

        let resp = client.delete("/suppliers/missing-id").dispatch().await;
        assert_eq!(resp.status(), Status::NotFound);
        let body = resp.into_string().await.unwrap();
        assert_failure_response(&body, "Failed to delete supplier: Supplier not found or deletion error from mock");
    }

    #[async_test]
    async fn test_malformed_post_request() {

        let client = build_test_client_with_mock(|_| {}).await;

        let resp = client.post("/suppliers")
            .header(ContentType::JSON)
            .body("{bad json") 
            .dispatch().await;

        assert_eq!(resp.status(), Status::BadRequest);
    }

    #[async_test]
    async fn test_malformed_put_request() {
        let client = build_test_client_with_mock(|_| {}).await;

        let resp = client.put("/suppliers/test-id")
            .header(ContentType::JSON)
            .body("{bad json") 
            .dispatch().await;

        assert_eq!(resp.status(), Status::BadRequest);
    }
}