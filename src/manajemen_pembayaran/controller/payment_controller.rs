use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use rocket::{get, post, put, delete, routes, Route, State, catch};
use rocket::serde::{json::Json};
use rocket::http::Status;
use sqlx::{Any, Pool};

use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::service::payment_service::PaymentService;

#[derive(Deserialize)]
pub struct CreatePaymentRequest {
    pub transaction_id: String,
    pub amount: f64,
    pub method: String,
}

#[derive(Deserialize)]
pub struct UpdatePaymentStatusRequest {
    pub payment_id: String,
    pub new_status: String,
    pub additional_amount: Option<f64>,
}

#[derive(Deserialize)]
pub struct AddInstallmentRequest {
    pub payment_id: String,
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct PaymentFilterRequest {
    pub status: Option<String>,
    pub method: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

pub struct PaymentController {
    service: Arc<dyn PaymentService>,
}

impl PaymentController {
    pub fn new(service: Arc<dyn PaymentService>) -> Self {
        Self { service }
    }
}

// Updated endpoints yang menggunakan koneksi database
#[post("/create", format = "json", data = "<request>")]
pub async fn create_payment_endpoint(
    db: &State<Pool<Any>>,
    request: Json<CreatePaymentRequest>,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<Payment>>) {
    // Convert method string ke enum
    let method = match request.method.to_uppercase().as_str() {
        "CASH" => PaymentMethod::Cash,
        "CREDIT_CARD" => PaymentMethod::CreditCard,
        "TRANSFER" => PaymentMethod::Transfer,
        _ => {
            return (
                Status::BadRequest,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Invalid payment method".to_string()),
                }),
            )
        }
    };

    // Panggil service dengan koneksi database
    let result = controller.service.create_payment(
        db.inner().clone(),
        request.transaction_id.clone(),
        request.amount,
        method,
    ).await;

    // Handle response
    match result {
        Ok(payment) => (
            Status::Created,
            Json(ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            }),
        ),
        Err(error) => (
            Status::BadRequest,
            Json(ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            }),
        ),
    }
}

#[put("/update_status", format = "json", data = "<request>")]
pub async fn update_payment_status_endpoint(
    db: &State<Pool<Any>>,
    request: Json<UpdatePaymentStatusRequest>,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<Payment>>) {
    // Convert status string ke enum
    let status = match request.new_status.to_uppercase().as_str() {
        "PENDING" => PaymentStatus::Pending,
        "PAID" | "LUNAS" => PaymentStatus::Paid,
        "INSTALLMENT" | "CICILAN" => PaymentStatus::Installment,
        _ => {
            return (
                Status::BadRequest,
                Json(ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Invalid payment status".to_string()),
                }),
            )
        }
    };

    // Panggil service dengan koneksi database
    let result = controller.service.update_payment_status(
        db.inner().clone(),
        request.payment_id.clone(),
        status,
        request.additional_amount,
    ).await;

    // Handle response
    match result {
        Ok(payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            }),
        ),
        Err(error) => (
            Status::BadRequest,
            Json(ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            }),
        ),
    }
}

#[put("/add_installment", format = "json", data = "<request>")]
pub async fn add_installment_endpoint(
    db: &State<Pool<Any>>,
    request: Json<AddInstallmentRequest>,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<Payment>>) {
    // Panggil service dengan koneksi database
    let result = controller.service.add_installment(
        db.inner().clone(),
        &request.payment_id,
        request.amount,
    ).await;

    // Handle response
    match result {
        Ok(payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            }),
        ),
        Err(error) => (
            Status::BadRequest,
            Json(ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            }),
        ),
    }
}

#[get("/<payment_id>")]
pub async fn get_payment_endpoint(
    db: &State<Pool<Any>>,
    payment_id: String,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<Payment>>) {
    // Panggil service dengan koneksi database
    let result = controller.service.get_payment(db.inner().clone(), &payment_id).await;

    // Handle response
    match result {
        Ok(payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            }),
        ),
        Err(_) => (
            Status::NotFound,
            Json(ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Payment with ID {} not found", payment_id)),
            }),
        ),
    }
}

#[get("/by_transaction/<transaction_id>")]
pub async fn get_payment_by_transaction_endpoint(
    db: &State<Pool<Any>>,
    transaction_id: String,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<Payment>>) {
    // Panggil service dengan koneksi database
    let result = controller.service.get_payment_by_transaction(
        db.inner().clone(),
        &transaction_id,
    ).await;

    // Handle response
    match result {
        Ok(payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            }),
        ),
        Err(_) => (
            Status::NotFound,
            Json(ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Payment with transaction ID {} not found", transaction_id)),
            }),
        ),
    }
}

#[get("/all?<status>&<method>")]
pub async fn get_all_payments_endpoint(
    db: &State<Pool<Any>>,
    status: Option<String>,
    method: Option<String>,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<Vec<Payment>>>) {
    // Buat filters jika parameter tersedia
    let mut filters = None;
    if status.is_some() || method.is_some() {
        let mut filter_map = HashMap::new();
        
        if let Some(status_val) = status {
            filter_map.insert("status".to_string(), status_val);
        }
        
        if let Some(method_val) = method {
            filter_map.insert("method".to_string(), method_val);
        }
        
        filters = Some(filter_map);
    }

    // Panggil service dengan koneksi database
    let result = controller.service.get_all_payments(db.inner().clone(), filters).await;

    // Handle response
    match result {
        Ok(payments) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                data: Some(payments),
                message: None,
            }),
        ),
        Err(error) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            }),
        ),
    }
}

#[delete("/<payment_id>")]
pub async fn delete_payment_endpoint(
    db: &State<Pool<Any>>,
    payment_id: String,
    controller: &State<PaymentController>,
) -> (Status, Json<ApiResponse<()>>) {
    // Panggil service dengan koneksi database
    let result = controller.service.delete_payment(db.inner().clone(), payment_id.clone()).await;

    // Handle response
    match result {
        Ok(_) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                data: Some(()),
                message: None,
            }),
        ),
        Err(error) => (
            Status::BadRequest,
            Json(ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            }),
        ),
    }
}

// Error catchers
#[catch(404)]
pub fn not_found_catcher() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: false,
        data: None,
        message: Some("The requested resource was not found.".to_string()),
    })
}

#[catch(400)]
pub fn bad_request_catcher() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: false,
        data: None,
        message: Some("Bad request. Please check your input.".to_string()),
    })
}

// Function to get all routes
pub fn get_routes() -> Vec<Route> {
    routes![
        create_payment_endpoint,
        update_payment_status_endpoint,
        add_installment_endpoint,
        get_payment_endpoint,
        get_payment_by_transaction_endpoint,
        get_all_payments_endpoint,
        delete_payment_endpoint
    ]
}

// Function to get all catchers
pub fn get_catchers() -> Vec<rocket::Catcher> {
    catchers![not_found_catcher, bad_request_catcher]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use rocket::catchers;
    use uuid::Uuid;

    use crate::manajemen_pembayaran::repository::payment_repository_impl::PaymentRepositoryImpl;
    use crate::manajemen_pembayaran::repository::payment_repository::PaymentRepository;
    use crate::manajemen_pembayaran::patterns::observer::{PaymentSubject, TransactionObserver, PaymentObserver};
    use crate::manajemen_pembayaran::service::payment_service_impl::PaymentServiceImpl;
    use crate::manajemen_pembayaran::service::payment_service::PaymentService;

    fn setup_controller() -> PaymentController {
        let repository: Arc<dyn PaymentRepository> = Arc::new(PaymentRepositoryImpl::new());
        
        let mut subject = PaymentSubject::new();
        let observer = Arc::new(TransactionObserver) as Arc<dyn PaymentObserver>;
        subject.attach(observer);
        let subject = Arc::new(subject);
        
        let service: Arc<dyn PaymentService> = Arc::new(PaymentServiceImpl::new(repository, subject));
        
        PaymentController::new(service)
    }

    #[test]
    fn test_create_payment() {
        let controller = setup_controller();
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let request = CreatePaymentRequest {
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        
        let response = controller.create_payment(request);
        
        assert!(response.success);
        assert!(response.data.is_some());
        let payment = response.data.unwrap();
        assert_eq!(payment.transaction_id, transaction_id);
        assert_eq!(payment.amount, 1000.0);
        assert_eq!(payment.method, PaymentMethod::Cash);
    }

    #[test]
    fn test_invalid_payment_method() {
        let controller = setup_controller();
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let request = CreatePaymentRequest {
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: "INVALID".to_string(),
        };
        
        let response = controller.create_payment(request);
        
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.message.is_some());
        assert_eq!(response.message.unwrap(), "Metode pembayaran tidak valid");
    }

    #[test]
    fn test_update_payment_status() {
        let controller = setup_controller();
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let create_request = CreatePaymentRequest {
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        
        let create_response = controller.create_payment(create_request);
        let payment = create_response.data.unwrap();
        
        let update_request = UpdatePaymentStatusRequest {
            payment_id: payment.id.clone(),
            new_status: "CICILAN".to_string(),
            additional_amount: Some(500.0),
        };
        
        let update_response = controller.update_payment_status(update_request);
        
        assert!(update_response.success);
        assert!(update_response.data.is_some());
        let updated_payment = update_response.data.unwrap();
        assert_eq!(updated_payment.status, PaymentStatus::Installment);
        assert_eq!(updated_payment.installments.len(), 1);
        assert_eq!(updated_payment.installments[0].amount, 500.0);
    }

    #[test]
    fn test_get_payment() {
        let controller = setup_controller();
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        let create_request = CreatePaymentRequest {
            transaction_id: transaction_id.clone(),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        
        let create_response = controller.create_payment(create_request);
        let payment = create_response.data.unwrap();
        
        let get_response = controller.get_payment(&payment.id);
        
        assert!(get_response.success);
        assert!(get_response.data.is_some());
        assert_eq!(get_response.data.unwrap().id, payment.id);
        
        let get_by_tx_response = controller.get_payment_by_transaction(&transaction_id);
        
        assert!(get_by_tx_response.success);
        assert!(get_by_tx_response.data.is_some());
        assert_eq!(get_by_tx_response.data.unwrap().transaction_id, transaction_id);
    }

    #[test]
    fn test_add_installment() {
        let controller = setup_controller();
        
        let create_request = CreatePaymentRequest {
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        
        let create_response = controller.create_payment(create_request);
        let payment = create_response.data.unwrap();
        
        let update_request = UpdatePaymentStatusRequest {
            payment_id: payment.id.clone(),
            new_status: "CICILAN".to_string(),
            additional_amount: None,
        };
        
        controller.update_payment_status(update_request);
        
        let add_request = AddInstallmentRequest {
            payment_id: payment.id.clone(),
            amount: 500.0,
        };
        
        let add_response = controller.add_installment(add_request);
        
        assert!(add_response.success);
        assert!(add_response.data.is_some());
        assert_eq!(add_response.data.unwrap().installments.len(), 1);
    }

    #[test]
    fn test_delete_payment() {
        let controller = setup_controller();
        
        let create_request = CreatePaymentRequest {
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        
        let create_response = controller.create_payment(create_request);
        let payment = create_response.data.unwrap();
        
        let update_request = UpdatePaymentStatusRequest {
            payment_id: payment.id.clone(),
            new_status: "CICILAN".to_string(),
            additional_amount: None,
        };
        
        controller.update_payment_status(update_request);
        
        let delete_response = controller.delete_payment(&payment.id);
        
        assert!(delete_response.success);
        assert!(delete_response.data.is_some());
        
        let get_response = controller.get_payment(&payment.id);
        assert!(!get_response.success);
    }

    #[test]
    fn test_get_all_payments() {
        let controller = setup_controller();

        let request1 = CreatePaymentRequest {
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 1000.0,
            method: "CASH".to_string(),
        };
        controller.create_payment(request1);

        let request2 = CreatePaymentRequest {
            transaction_id: format!("TRX-{}", Uuid::new_v4()),
            amount: 2000.0,
            method: "CREDIT_CARD".to_string(),
        };
        controller.create_payment(request2);

        let response = controller.get_all_payments(None);
        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().len(), 2);

        let filter_request = PaymentFilterRequest {
            status: None,
            method: Some("CASH".to_string()),
        };
        let response = controller.get_all_payments(Some(filter_request));
        assert!(response.success);
        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap().len(), 1);
    }

    use rocket::local::blocking::Client;
    use rocket::http::{Status, ContentType};

    fn setup_rocket() -> rocket::Rocket<rocket::Build> {
        let controller = setup_controller();
        rocket::build()
            .manage(controller)
            .mount("/payments", get_routes())
            .register("/", get_catchers())
    }

    #[test]
    fn test_create_payment_with_invalid_method() {
        let client = Client::tracked(setup_rocket()).expect("valid rocket instance");
        let request_body = serde_json::json!({
            "transaction_id": "TRX-INVALID",
            "amount": 1000.0,
            "method": "INVALID_METHOD"
        });

        let response = client
            .post("/payments/create")
            .header(ContentType::JSON)
            .body(request_body.to_string())
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        if let Some(response_body) = response.into_json::<ApiResponse<()>>() {
            assert!(!response_body.success);
            assert_eq!(response_body.message.unwrap(), "Metode pembayaran tidak valid");
        } else {
            panic!("Response body is None");
        }
    }

    #[test]
    fn test_get_payment_not_found() {
        let client = Client::tracked(setup_rocket()).expect("valid rocket instance");
        let response = client.get("/payments/unknown_id").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        if let Some(response_body) = response.into_json::<ApiResponse<()>>() {
            assert!(!response_body.success);
            assert_eq!(response_body.message.unwrap(), "Pembayaran dengan ID unknown_id tidak ditemukan");
        } else {
            panic!("Response body is None");
        }
    }

    #[test]
    fn test_delete_payment_not_found() {
        let client = Client::tracked(setup_rocket()).expect("valid rocket instance");
        let response = client.delete("/payments/unknown_id").dispatch();

        assert_eq!(response.status(), Status::NotFound);
        if let Some(response_body) = response.into_json::<ApiResponse<()>>() {
            assert!(!response_body.success);
            assert_eq!(response_body.message.unwrap(), "Pembayaran dengan ID unknown_id tidak ditemukan");
        } else {
            panic!("Response body is None");
        }
    }

    #[test]
    fn test_update_payment_status_invalid_status() {
        let client = Client::tracked(setup_rocket()).expect("valid rocket instance");
        let request_body = serde_json::json!({
            "payment_id": "valid_id",
            "new_status": "INVALID_STATUS",
            "additional_amount": 500.0
        });

        let response = client
            .put("/payments/update_status")
            .header(ContentType::JSON)
            .body(request_body.to_string())
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        if let Some(response_body) = response.into_json::<ApiResponse<()>>() {
            assert!(!response_body.success);
            assert_eq!(response_body.message.unwrap(), "Status pembayaran tidak valid");
        } else {
            panic!("Response body is None");
        }
    }
}