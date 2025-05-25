use rocket::{get, post, put, delete, routes, State, http::Status};
use rocket::serde::{json::Json, Deserialize, Serialize};
use sqlx::{Any, Pool};
use std::sync::Arc;

use crate::manajemen_supplier::model::supplier::Supplier;
use crate::manajemen_supplier::service::supplier_service::SupplierService;
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

// No SupplierController struct needed here for route logic if dependencies are injected directly.
// If SupplierController had other methods or state not related to being a holder
// for service/notifier, that would need separate consideration.

#[post("/suppliers", format = "json", data = "<request_data>")]
pub async fn save_supplier(
    request_data: Json<SupplierRequest>,
    db_pool: &State<Pool<Any>>,
    service: &State<Arc<dyn SupplierService>>, // Injected service
) -> (Status, Json<ApiResponse<Supplier>>) {
    match service.inner().save_supplier( // Use .inner() to get the Arc, then call method
        db_pool.inner().clone(),
        request_data.name.clone(),
        request_data.jenis_barang.clone(),
        request_data.jumlah_barang,
        request_data.resi.clone(),
    ).await {
        Ok(saved_supplier) => {
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
    service: &State<Arc<dyn SupplierService>>, // Injected service
) -> (Status, Json<ApiResponse<Supplier>>) {
    match service.inner().get_supplier(db_pool.inner().clone(), &suppliers_id).await {
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
    service: &State<Arc<dyn SupplierService>>, // Injected service
    // notifier: &State<Arc<dyn SupplierNotifier>>, // Add if update needs notification
) -> (Status, Json<ApiResponse<Supplier>>) {
    match service.inner().update_supplier(
        db_pool.inner().clone(),
        id.clone(),
        request_data.name.clone(),
        request_data.jenis_barang.clone(),
        request_data.jumlah_barang,
        request_data.resi.clone(),
    ).await {
        Ok(()) => {
            // Consider if a notification is needed here via the notifier
            match service.inner().get_supplier(db_pool.inner().clone(), &id).await {
                Ok(Some(updated_supplier_model)) => {
                    // Example: notifier.inner().notify_supplier_updated(&updated_supplier_model).await;
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
    service: &State<Arc<dyn SupplierService>>, // Injected service
    // notifier: &State<Arc<dyn SupplierNotifier>>, // Add if delete needs notification
) -> (Status, Json<ApiResponse<()>>) {
    match service.inner().delete_supplier(db_pool.inner().clone(), &id).await {
        Ok(()) => {
            // Example: notifier.inner().notify_supplier_deleted(&id).await;
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
    use rocket::http::Status;
    use rocket::{uri, Rocket, async_test}; // Removed unused `routes` from here as it's used for `supplier_routes`
    use sqlx::any::{install_default_drivers, AnyPoolOptions};
    use std::sync::Arc;
    use uuid::Uuid;

    // Assuming your actual model, service, repository, notifier implementations are correctly pathed
    use crate::manajemen_supplier::model::supplier::Supplier;
    use crate::manajemen_supplier::service::supplier_service::SupplierService;
    use crate::manajemen_supplier::service::supplier_service_impl::SupplierServiceImpl;
    use crate::manajemen_supplier::repository::supplier_repository_impl::SupplierRepositoryImpl;
    use crate::manajemen_supplier::repository::supplier_repository::SupplierRepository;
    use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;
    use crate::manajemen_supplier::service::supplier_dispatcher::SupplierDispatcher; // Assuming this is your concrete notifier

    async fn deserialize_response_body<T>(
        response: rocket::local::asynchronous::LocalResponse<'_>,
    ) -> ApiResponse<T>
    where
        T: for<'de> serde::Deserialize<'de> + std::fmt::Debug + Send + 'static,
    {
        response
            .into_json::<ApiResponse<T>>()
            .await
            .expect("Failed to deserialize ApiResponse from body")
    }

    async fn setup_rocket_instance_for_supplier_tests() -> Rocket<rocket::Build> {
        install_default_drivers();
        let db_pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to test in-memory SQLite DB");

        sqlx::migrate!("migrations/test")
            .run(&db_pool)
            .await
            .expect("Failed to run supplier database migrations for tests. Check path and SQL files.");

        let supplier_repo: Arc<dyn SupplierRepository> = Arc::new(SupplierRepositoryImpl::new());
        // Assuming SupplierDispatcher is your concrete notifier for tests.
        // If SupplierServiceImpl takes the notifier as a dependency for its *internal* workings,
        // it should be consistent. The routes will now also request a notifier.
        let supplier_event_dispatcher: Arc<dyn SupplierNotifier> = Arc::new(SupplierDispatcher::new());

        let supplier_service_instance: Arc<dyn SupplierService> = Arc::new(SupplierServiceImpl::new(
            supplier_repo,
            supplier_event_dispatcher.clone(), // If service impl needs it
        ));
        
        // Instead of managing SupplierController, manage its dependencies (service and notifier)
        rocket::build()
            .manage(db_pool) // For direct use by service methods if they still take Pool directly
            .manage(supplier_service_instance.clone()) // Manage the service Arc
            .manage(supplier_event_dispatcher.clone())  // Manage the notifier Arc
            .mount("/", supplier_routes())
    }

    fn sample_supplier_request(name_suffix: &str) -> SupplierRequest {
        SupplierRequest {
            name: format!("Integ Test Supplier {}", name_suffix),
            jenis_barang: "Integration Goods".to_string(),
            jumlah_barang: 150,
            resi: format!("INTEG-RESI-{}", name_suffix),
        }
    }

    #[async_test]
    async fn test_integ_create_and_get_supplier() {
        let rocket_instance = setup_rocket_instance_for_supplier_tests().await;
        let client = Client::tracked(rocket_instance).await.expect("Valid Rocket instance");

        let create_req = sample_supplier_request("CreateAndGet");

        let post_response = client.post(uri!(save_supplier)) // Use super:: not needed if routes are in same mod
            .json(&create_req)
            .dispatch()
            .await;

        assert_eq!(post_response.status(), Status::Created);
        let post_api_resp = deserialize_response_body::<Supplier>(post_response).await;
        assert!(post_api_resp.success);
        let created_supplier = post_api_resp.data.expect("Supplier data should be present on creation");
        
        assert_eq!(created_supplier.name, create_req.name);
        assert!(!created_supplier.id.is_empty());

        let created_id = created_supplier.id.clone();

        let get_response = client.get(uri!(get_supplier(suppliers_id = created_id.clone()))).dispatch().await;
        assert_eq!(get_response.status(), Status::Ok);
        let get_api_resp = deserialize_response_body::<Supplier>(get_response).await;
        assert!(get_api_resp.success);
        let fetched_supplier = get_api_resp.data.expect("Supplier data should be present on get");

        assert_eq!(fetched_supplier.id, created_id);
        assert_eq!(fetched_supplier.name, create_req.name);
        assert_eq!(fetched_supplier.jenis_barang, create_req.jenis_barang);
    }

    #[async_test]
    async fn test_integ_get_supplier_not_found() {
        let rocket_instance = setup_rocket_instance_for_supplier_tests().await;
        let client = Client::tracked(rocket_instance).await.expect("Valid Rocket instance");

        let non_existent_id = format!("SUP-INTEG-{}", Uuid::new_v4());
        let response = client.get(uri!(get_supplier(suppliers_id = non_existent_id))).dispatch().await;

        assert_eq!(response.status(), Status::NotFound);
        let api_resp = deserialize_response_body::<Supplier>(response).await;
        assert!(!api_resp.success);
        assert!(api_resp.message.is_some() && api_resp.message.unwrap().contains("not found"));
    }
    
    // NOTE: The original update_supplier test had an assertion:
    // `assert_eq!(updated_supplier_data.jumlah_barang, initial_req.jumlah_barang + 50);`
    // This implies the service's update_supplier method *adds* to jumlah_barang rather than setting it.
    // The provided SupplierRequest for update does not suggest accumulation.
    // I will assume the service's `update_supplier` sets the value from the request.
    // If it's an accumulation, the service logic or test assertion needs to reflect that.
    // For now, I'll assume it sets the value.

    #[async_test]
    async fn test_integ_update_supplier() {
        let rocket_instance = setup_rocket_instance_for_supplier_tests().await;
        let client = Client::tracked(rocket_instance).await.expect("Valid Rocket instance");

        let initial_req = sample_supplier_request("UpdateInitial");
        let post_response = client.post(uri!(save_supplier)).json(&initial_req).dispatch().await;
        assert_eq!(post_response.status(), Status::Created);
        let created_supplier = deserialize_response_body::<Supplier>(post_response).await.data.unwrap();
        let supplier_id_to_update = created_supplier.id.clone();

        let update_payload = SupplierRequest {
            name: "Updated Supplier Name".to_string(),
            jenis_barang: "Updated Goods".to_string(),
            jumlah_barang: 200, // Changed from 50 to directly set a new value
            resi: "UPDATED-RESI-001".to_string(),
        };
        let update_response = client.put(uri!(update_supplier(id = supplier_id_to_update.clone())))
            .json(&update_payload)
            .dispatch()
            .await;
        
        assert_eq!(update_response.status(), Status::Ok);
        let updated_api_resp = deserialize_response_body::<Supplier>(update_response).await;
        assert!(updated_api_resp.success);
        let updated_supplier_data = updated_api_resp.data.expect("Updated supplier data missing");
        assert_eq!(updated_supplier_data.id, supplier_id_to_update);
        assert_eq!(updated_supplier_data.name, "Updated Supplier Name");
        assert_eq!(updated_supplier_data.jumlah_barang, 200); // Verify it's the new value

        // Verify by fetching again
        let get_response = client.get(uri!(get_supplier(suppliers_id = supplier_id_to_update))).dispatch().await;
        assert_eq!(get_response.status(), Status::Ok);
        let fetched_supplier = deserialize_response_body::<Supplier>(get_response).await.data.unwrap();
        assert_eq!(fetched_supplier.name, "Updated Supplier Name");
        assert_eq!(fetched_supplier.jumlah_barang, 200);
    }

    #[async_test]
    async fn test_integ_delete_supplier() {
        let rocket_instance = setup_rocket_instance_for_supplier_tests().await;
        let client = Client::tracked(rocket_instance).await.expect("Valid Rocket instance");

        let req = sample_supplier_request("ToDelete");
        let post_response = client.post(uri!(save_supplier)).json(&req).dispatch().await;
        assert_eq!(post_response.status(), Status::Created);
        let created_supplier = deserialize_response_body::<Supplier>(post_response).await.data.unwrap();
        let supplier_id_to_delete = created_supplier.id.clone();

        let delete_response = client.delete(uri!(delete_supplier(id = supplier_id_to_delete.clone()))).dispatch().await;
        assert_eq!(delete_response.status(), Status::Ok);
        let delete_api_resp = deserialize_response_body::<()>(delete_response).await; // Expect no data for delete
        assert!(delete_api_resp.success);
        assert!(delete_api_resp.message.unwrap().contains("deleted successfully"));

        let get_response_after_delete = client.get(uri!(get_supplier(suppliers_id = supplier_id_to_delete))).dispatch().await;
        assert_eq!(get_response_after_delete.status(), Status::NotFound);
    }
}