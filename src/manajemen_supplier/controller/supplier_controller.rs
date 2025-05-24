use rocket::{get, post, put, delete, routes, State, http::Status,uri};
use rocket::serde::{json::Json, Deserialize, Serialize};
use sqlx::{Any, Pool};
use std::sync::Arc;

use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::service::supplier_service::SupplierService;
use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SupplierRequest {
    pub name: String,
    pub jenis_barang: String,
    pub jumlah_barang: i32,
    pub resi: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

pub struct SupplierController {
    service: Arc<dyn SupplierService>,
    notifier: Arc<dyn SupplierNotifier>,
}

impl SupplierController {
    pub fn new(service: Arc<dyn SupplierService>, notifier: Arc<dyn SupplierNotifier>) -> Self {
        Self { service, notifier }
    }
}

#[post("/suppliers", format = "json", data = "<request_data>")]
pub async fn save_supplier(
    request_data: Json<SupplierRequest>,
    db_pool: &State<Pool<Any>>,
    controller_state: &State<SupplierController>,
) -> (Status, Json<ApiResponse<Supplier>>) {
    match controller_state.service.save_supplier(
        db_pool.inner().clone(),
        request_data.name.clone(),
        request_data.jenis_barang.clone(),
        request_data.jumlah_barang,
        request_data.resi.clone(),
    ).await {
        Ok(saved_supplier) => {
            controller_state.notifier.notify_supplier_saved(&saved_supplier).await;
            (
                Status::Created,
                Json(ApiResponse {
                    success: true,
                    message: Some("Supplier created successfully".to_string()),
                    data: Some(saved_supplier),
                }),
            )
        }
        Err(service_error_msg) => {
            eprintln!("[Controller Error] Save supplier failed: {}", service_error_msg);
            (
                Status::InternalServerError,
                Json(ApiResponse {
                    success: false,
                    message: Some(service_error_msg),
                    data: None::<Supplier>,
                }),
            )
        }
    }
}

#[get("/suppliers/<suppliers_id>")]
pub async fn get_supplier(
    suppliers_id: String,
    db_pool: &State<Pool<Any>>,
    controller_state: &State<SupplierController>,
) -> (Status, Json<ApiResponse<Supplier>>) {
    match controller_state.service.get_supplier(db_pool.inner().clone(), &suppliers_id).await {
        Ok(Some(supplier_model)) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: Some("Supplier found successfully.".to_string()),
                data: Some(supplier_model),
            }),
        ),
        Ok(None) => (
            Status::NotFound,
            Json(ApiResponse {
                success: false,
                message: Some(format!("Supplier with ID '{}' not found.", suppliers_id)),
                data: None::<Supplier>,
            }),
        ),
        Err(service_error_msg) => {
            eprintln!("[Controller Error] Get supplier failed for ID '{}': {}", suppliers_id, service_error_msg);
            (
                Status::InternalServerError,
                Json(ApiResponse {
                    success: false,
                    message: Some(service_error_msg),
                    data: None::<Supplier>,
                }),
            )
        }
    }
}

#[put("/suppliers/<id>", format = "json", data = "<request_data>")]
pub async fn update_supplier(
    id: String,
    request_data: Json<SupplierRequest>,
    db_pool: &State<Pool<Any>>,
    controller_state: &State<SupplierController>,
) -> (Status, Json<ApiResponse<Supplier>>) {
    match controller_state.service.update_supplier(
        db_pool.inner().clone(),
        id.clone(),
        request_data.name.clone(),
        request_data.jenis_barang.clone(),
        request_data.jumlah_barang,
        request_data.resi.clone(),
    ).await {
        Ok(()) => {
            match controller_state.service.get_supplier(db_pool.inner().clone(), &id).await {
                Ok(Some(updated_supplier_model)) => {
                    (
                        Status::Ok,
                        Json(ApiResponse {
                            success: true,
                            message: Some("Supplier updated successfully.".to_string()),
                            data: Some(updated_supplier_model),
                        }),
                    )
                }
                Ok(None) => {
                    eprintln!("[Controller Error] Supplier {} not found after update operation (was it deleted concurrently?).", id);
                    (
                        Status::NotFound,
                        Json(ApiResponse {
                            success: false,
                            message: Some(format!("Supplier with ID '{}' not found after update.", id)),
                            data: None::<Supplier>,
                        }),
                    )
                }
                Err(e) => {
                    eprintln!("[Controller Error] Failed to fetch supplier {} after update: {}", id, e);
                    (
                        Status::InternalServerError,
                        Json(ApiResponse {
                            success: false,
                            message: Some(format!("Error fetching supplier after update: {}", e)),
                            data: None::<Supplier>,
                        }),
                    )
                }
            }
        }
        Err(service_error_msg) => {
            eprintln!("[Controller Error] Update supplier failed for ID '{}': {}", id, service_error_msg);
            let status_code = if service_error_msg.to_lowercase().contains("not found") {
                Status::NotFound
            } else {
                Status::InternalServerError
            };
            (
                status_code,
                Json(ApiResponse {
                    success: false,
                    message: Some(service_error_msg),
                    data: None::<Supplier>,
                }),
            )
        }
    }
}

#[delete("/suppliers/<id>")]
pub async fn delete_supplier(
    id: String,
    db_pool: &State<Pool<Any>>,
    controller_state: &State<SupplierController>,
) -> (Status, Json<ApiResponse<()>>) {
    match controller_state.service.delete_supplier(db_pool.inner().clone(), &id).await {
        Ok(()) => {
            (
                Status::Ok,
                Json(ApiResponse {
                    success: true,
                    message: Some(format!("Supplier with ID '{}' deleted successfully.", id)),
                    data: None::<()>,
                }),
            )
        }
        Err(service_error_msg) => {
            eprintln!("[Controller Error] Delete supplier failed for ID '{}': {}", id, service_error_msg);
            let status_code = if service_error_msg.to_lowercase().contains("not found") {
                Status::NotFound
            } else {
                Status::InternalServerError
            };
            (
                status_code,
                Json(ApiResponse {
                    success: false,
                    message: Some(service_error_msg),
                    data: None::<()>,
                }),
            )
        }
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
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::http::{uri, Status};
    use rocket::async_test;
    use sqlx::any::{install_default_drivers, AnyPoolOptions};
    use std::sync::Arc;
    use chrono::Utc;
    use mockall::predicate::*;

    use crate::manajemen_supplier::service::supplier_service::MockSupplierService;
    use crate::manajemen_supplier::service::supplier_notifier::MockSupplierNotifier;
    use crate::manajemen_supplier::model::supplier::Supplier;


    async fn setup_rocket_with_custom_mocks(
        service_setup: impl FnOnce(&mut MockSupplierService),
        notifier_setup: impl FnOnce(&mut MockSupplierNotifier),
    ) -> Client {
        install_default_drivers();
        let test_db_pool = AnyPoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        let mut mock_service = MockSupplierService::new();
        service_setup(&mut mock_service);
        let mut mock_notifier = MockSupplierNotifier::new();
        notifier_setup(&mut mock_notifier);

        let controller = SupplierController::new(Arc::new(mock_service), Arc::new(mock_notifier));
        let rocket_instance = rocket::build().manage(test_db_pool).manage(controller).mount("/", supplier_routes());
        Client::tracked(rocket_instance).await.unwrap()
    }

    async fn deserialize_api_response_body<T>(
            response: rocket::local::asynchronous::LocalResponse<'_>,
        ) -> ApiResponse<T>
    where
        T: for<'de> serde::Deserialize<'de> + std::fmt::Debug + Send + 'static,
    {
        response.into_json::<ApiResponse<T>>().await.expect("Failed to deserialize ApiResponse")
    }

    fn sample_req_data(name_suffix: &str) -> SupplierRequest {
        SupplierRequest {
            name: format!("Test Supplier {}", name_suffix),
            jenis_barang: "Electronics".to_string(),
            jumlah_barang: 100,
            resi: format!("RESI-{}", name_suffix),
        }
    }

    fn sample_model_returned_by_service(id: &str, name: &str, jenis: &str, jumlah: i32, resi: &str) -> Supplier {
        Supplier {
            id: id.to_string(), name: name.to_string(), jenis_barang: jenis.to_string(),
            jumlah_barang: jumlah, resi: resi.to_string(), updated_at: Utc::now()
        }
    }

    #[async_test]
    async fn test_save_supplier_success() {
        let req = sample_req_data("NewSave");
        let expected_id = "service-gen-id-1";
        let returned_supplier_from_service = sample_model_returned_by_service(
            expected_id, &req.name, &req.jenis_barang, req.jumlah_barang, &req.resi
        );

        let client = setup_rocket_with_custom_mocks(
            |mock_service| {
                mock_service.expect_save_supplier()
                    .with(
                        always(),
                        eq(req.name.clone()),
                        eq(req.jenis_barang.clone()),
                        eq(req.jumlah_barang),
                        eq(req.resi.clone()),
                    )
                    .times(1)
                    .returning({
                        let supplier_clone = returned_supplier_from_service.clone();
                        move |_, _, _, _, _| Ok(supplier_clone.clone())
                    });
            },
            |mock_notifier| {
                mock_notifier.expect_notify_supplier_saved()
                    .with(eq(returned_supplier_from_service.clone()))
                    .times(1)
                    .returning(|_| ());
            }
        ).await;

        let response = client.post(uri!(super::save_supplier)).json(&req).dispatch().await;

        assert_eq!(response.status(), Status::Created);
        let api_resp = deserialize_api_response_body::<Supplier>(response).await;
        assert!(api_resp.success);
        assert_eq!(api_resp.message.unwrap(), "Supplier created successfully");
        let data = api_resp.data.unwrap();
        assert_eq!(data.id, expected_id);
        assert_eq!(data.name, req.name);
    }

    #[async_test]
    async fn test_get_supplier_found() {
        let supplier_id = "get-me-007";
        let expected_model = sample_model_returned_by_service(supplier_id, "James Bond", "Spy Gear", 7, "RESI-JB007");

        let client = setup_rocket_with_custom_mocks(
            |mock_service| {
                mock_service.expect_get_supplier()
                    .with(always(), eq(supplier_id))
                    .times(1)
                    .returning({
                        let model_clone = expected_model.clone();
                        move |_, _| Ok(Some(model_clone.clone()))
                    });
            },
            |_| {}
        ).await;

        let response = client.get(uri!(super::get_supplier(suppliers_id = supplier_id))).dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let api_resp = deserialize_api_response_body::<Supplier>(response).await;
        assert!(api_resp.success);
        assert_eq!(api_resp.message.unwrap(), "Supplier found successfully.");
        let data = api_resp.data.unwrap();
        assert_eq!(data.id, supplier_id);
    }

    #[async_test]
    async fn test_get_supplier_not_found() {
        let supplier_id_404 = "get-me-404";
        let client = setup_rocket_with_custom_mocks(
            |mock_service| {
                mock_service.expect_get_supplier()
                    .with(always(), eq(supplier_id_404))
                    .times(1)
                    .returning(|_, _| Ok(None));
            },
            |_| {}
        ).await;

        let response = client.get(uri!(super::get_supplier(suppliers_id = supplier_id_404))).dispatch().await;
        assert_eq!(response.status(), Status::NotFound);
        let api_resp = deserialize_api_response_body::<Supplier>(response).await;
        assert!(!api_resp.success);
        assert!(api_resp.message.unwrap().contains("not found"));
    }

    #[async_test]
    async fn test_update_supplier_success() {
        let supplier_id_to_update = "update-me-001";
        let req = sample_req_data("Updated");
        let model_after_update = sample_model_returned_by_service(
            supplier_id_to_update, &req.name, &req.jenis_barang, req.jumlah_barang, &req.resi
        );

        let client = setup_rocket_with_custom_mocks(
            |mock_service| {
                mock_service.expect_update_supplier()
                    .with(
                        always(),
                        eq(supplier_id_to_update.to_string()),
                        eq(req.name.clone()),
                        eq(req.jenis_barang.clone()),
                        eq(req.jumlah_barang),
                        eq(req.resi.clone())
                    )
                    .times(1)
                    .returning(|_, _, _, _, _, _| Ok(()));

                mock_service.expect_get_supplier()
                    .with(always(), eq(supplier_id_to_update))
                    .times(1)
                    .returning({
                        let model_clone = model_after_update.clone();
                        move |_, _| Ok(Some(model_clone.clone()))
                    });
            },
            |_notifier_mock| { }
        ).await;

        let response = client.put(uri!(super::update_supplier(id = supplier_id_to_update)))
            .json(&req)
            .dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let api_resp = deserialize_api_response_body::<Supplier>(response).await;
        assert!(api_resp.success);
        assert_eq!(api_resp.message.unwrap(), "Supplier updated successfully.");
        let data = api_resp.data.unwrap();
        assert_eq!(data.id, supplier_id_to_update);
        assert_eq!(data.name, req.name);
    }

    #[async_test]
    async fn test_update_supplier_service_returns_not_found() {
        let supplier_id_404 = "update-me-404";
        let req = sample_req_data("NotFoundUpdate");

        let client = setup_rocket_with_custom_mocks(
            |mock_service| {
                mock_service.expect_update_supplier()
                    .with(always(), eq(supplier_id_404.to_string()), always(), always(), always(), always())
                    .times(1)
                    .returning(|_, _, _, _, _, _| Err("Service: Supplier not found for update.".to_string()));
            },
            |_| {}
        ).await;

        let response = client.put(uri!(super::update_supplier(id = supplier_id_404)))
            .json(&req)
            .dispatch().await;

        assert_eq!(response.status(), Status::NotFound);
        let api_resp = deserialize_api_response_body::<Supplier>(response).await;
        assert!(!api_resp.success);
        assert!(api_resp.message.unwrap().contains("not found"));
    }

    #[async_test]
    async fn test_delete_supplier_success() {
        let supplier_id_to_delete = "delete-me-001";

        let client = setup_rocket_with_custom_mocks(
            |mock_service| {
                mock_service.expect_delete_supplier()
                    .with(always(), eq(supplier_id_to_delete))
                    .times(1)
                    .returning(|_, _| Ok(()));
            },
            |_notifier_mock| {}
        ).await;

        let response = client.delete(uri!(super::delete_supplier(id = supplier_id_to_delete))).dispatch().await;

        assert_eq!(response.status(), Status::Ok); 
        let api_resp = deserialize_api_response_body::<()>(response).await;
        assert!(api_resp.success);
        assert!(api_resp.message.unwrap().contains("deleted successfully"));
    }

    #[async_test]
    async fn test_delete_supplier_service_returns_not_found() {
        let supplier_id_404 = "delete-me-404";

        let client = setup_rocket_with_custom_mocks(
            |mock_service| {
                mock_service.expect_delete_supplier()
                    .with(always(), eq(supplier_id_404))
                    .times(1)
                    .returning(|_, _| Err("Service: Supplier not found for delete.".to_string()));
            },
            |_| {}
        ).await;

        let response = client.delete(uri!(super::delete_supplier(id = supplier_id_404))).dispatch().await;

        assert_eq!(response.status(), Status::NotFound);
        let api_resp = deserialize_api_response_body::<()>(response).await;
        assert!(!api_resp.success);
        assert!(api_resp.message.unwrap().contains("not found"));
    }
}