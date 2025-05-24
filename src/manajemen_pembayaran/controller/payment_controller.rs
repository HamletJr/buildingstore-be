use std::collections::HashMap;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use rocket::{get, post, put, delete, routes, Route, State, catch};
use rocket::serde::json::Json;
use rocket::http::Status;

use crate::manajemen_pembayaran::model::payment::Payment;
use crate::manajemen_pembayaran::service::payment_service::{PaymentService, PaymentError};
use sqlx::{Any, Pool};

#[derive(Deserialize)]
pub struct CreatePaymentRequest {
    pub transaction_id: String,
    pub amount: f64,
    pub method: String,
    pub status: String,
    pub due_date: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePaymentStatusRequest {
    pub new_status: String,
    pub additional_amount: Option<f64>,
}

#[derive(Deserialize)]
pub struct AddInstallmentRequest {
    pub amount: f64,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[post("/payments", format = "json", data = "<payment_request>")]
pub async fn create_payment(payment_request: Json<CreatePaymentRequest>, db: &State<Pool<Any>>) -> (Status, Json<ApiResponse<Payment>>) {
    let payment_service = PaymentService::new();
    
    let method: crate::manajemen_pembayaran::model::payment::PaymentMethod = match payment_service.parse_payment_method(&payment_request.method) {
        Ok(m) => m,
        Err(e) => {
            return (
                Status::BadRequest,
                Json(ApiResponse {
                    success: false,
                    message: format!("Invalid payment method: {:?}", e),
                    data: None,
                }),
            );
        }
    };
    
    let status = match payment_service.parse_payment_status(&payment_request.status) {
        Ok(s) => s,
        Err(e) => {
            return (
                Status::BadRequest,
                Json(ApiResponse {
                    success: false,
                    message: format!("Invalid payment status: {:?}", e),
                    data: None,
                }),
            );
        }
    };
    
    let due_date: Option<chrono::DateTime<Utc>> = match &payment_request.due_date {
        Some(date_str) => match chrono::DateTime::parse_from_rfc3339(date_str) {
            Ok(dt) => Some(dt.with_timezone(&Utc)),
            Err(_) => {
                return (
                    Status::BadRequest,
                    Json(ApiResponse {
                        success: false,
                        message: "Invalid due date format. Use RFC3339 format".to_string(),
                        data: None,
                    }),
                );
            }
        },
        None => None,
    };
    
    let payment = Payment {
        id: payment_service.generate_payment_id(),
        transaction_id: payment_request.transaction_id.clone(),
        amount: payment_request.amount,
        method,
        status,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date,
    };
    
    match payment_service.create_payment(db, payment).await {
        Ok(created_payment) => (
            Status::Created,
            Json(ApiResponse {
                success: true,
                message: "Payment created successfully".to_string(),
                data: Some(created_payment),
            }),
        ),
        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to create payment: {:?}", e),
                data: None,
            }),
        ),
    }
}

#[get("/payments/<id>")]
pub async fn get_payment_by_id(id: String, db: &State<Pool<Any>>) -> (Status, Json<ApiResponse<Payment>>) {
    let payment_service = PaymentService::new();
    
    match payment_service.get_payment_by_id(db, &id).await {
        Ok(payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: "Payment retrieved successfully".to_string(),
                data: Some(payment),
            }),
        ),
        Err(PaymentError::NotFound(msg)) => (
            Status::NotFound,
            Json(ApiResponse {
                success: false,
                message: msg,
                data: None,
            }),
        ),
        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to retrieve payment: {:?}", e),
                data: None,
            }),
        ),
    }
}

#[get("/payments?<status>&<method>&<transaction_id>")]
pub async fn get_all_payments(
    status: Option<String>,
    method: Option<String>,
    transaction_id: Option<String>,
    db: &State<Pool<Any>>
) -> (Status, Json<ApiResponse<Vec<Payment>>>) {
    let payment_service = PaymentService::new();
    
    let mut filters = HashMap::new();
    if let Some(status_str) = status {
        filters.insert("status".to_string(), status_str);
    }
    if let Some(method_str) = method {
        filters.insert("method".to_string(), method_str);
    }
    if let Some(tx_id) = transaction_id {
        filters.insert("transaction_id".to_string(), tx_id);
    }
    
    let filters_option = if filters.is_empty() { None } else { Some(filters) };
    
    match payment_service.get_all_payments(db, filters_option).await {
        Ok(payments) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: format!("Successfully retrieved {} payments", payments.len()),
                data: Some(payments),
            }),
        ),
        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to retrieve payments: {:?}", e),
                data: None,
            }),
        ),
    }
}

#[put("/payments/<id>/status", format = "json", data = "<status_request>")]
pub async fn update_payment_status(
    id: String,
    status_request: Json<UpdatePaymentStatusRequest>,
    db: &State<Pool<Any>>
) -> (Status, Json<ApiResponse<Payment>>) {
    let payment_service = PaymentService::new();
    
    let new_status = match payment_service.parse_payment_status(&status_request.new_status) {
        Ok(s) => s,
        Err(e) => {
            return (
                Status::BadRequest,
                Json(ApiResponse {
                    success: false,
                    message: format!("Invalid payment status: {:?}", e),
                    data: None,
                }),
            );
        }
    };
    
    match payment_service.update_payment_status(db, id, new_status, status_request.additional_amount).await {
        Ok(updated_payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: "Payment status updated successfully".to_string(),
                data: Some(updated_payment),
            }),
        ),
        Err(PaymentError::NotFound(msg)) => (
            Status::NotFound,
            Json(ApiResponse {
                success: false,
                message: msg,
                data: None,
            }),
        ),
        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to update payment status: {:?}", e),
                data: None,
            }),
        ),
    }
}

#[post("/payments/<id>/installments", format = "json", data = "<installment_request>")]
pub async fn add_installment(
    id: String,
    installment_request: Json<AddInstallmentRequest>,
    db: &State<Pool<Any>>
) -> (Status, Json<ApiResponse<Payment>>) {
    let payment_service = PaymentService::new();
    
    match payment_service.add_installment(db, &id, installment_request.amount).await {
        Ok(updated_payment) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: "Installment added successfully".to_string(),
                data: Some(updated_payment),
            }),
        ),
        Err(PaymentError::NotFound(msg)) => (
            Status::NotFound,
            Json(ApiResponse {
                success: false,
                message: msg,
                data: None,
            }),
        ),
        Err(PaymentError::InvalidInput(msg)) => (
            Status::BadRequest,
            Json(ApiResponse {
                success: false,
                message: msg,
                data: None,
            }),
        ),
        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to add installment: {:?}", e),
                data: None,
            }),
        ),
    }
}

#[delete("/payments/<id>")]
pub async fn delete_payment(id: String, db: &State<Pool<Any>>) -> (Status, Json<ApiResponse<()>>) {
    let payment_service = PaymentService::new();
    
    match payment_service.delete_payment(db, &id).await {
        Ok(_) => (
            Status::Ok,
            Json(ApiResponse {
                success: true,
                message: "Payment deleted successfully".to_string(),
                data: None,
            }),
        ),
        Err(e) => (
            Status::InternalServerError,
            Json(ApiResponse {
                success: false,
                message: format!("Failed to delete payment: {:?}", e),
                data: None,
            }),
        ),
    }
}

#[derive(Deserialize)]
pub struct PaymentFilterRequest {
    pub status: Option<String>,
    pub method: Option<String>,
    pub transaction_id: Option<String>,
}

pub fn routes() -> Vec<Route> {
    routes![
        create_payment,
        get_payment_by_id,
        get_all_payments,
        update_payment_status,
        add_installment,
        delete_payment
    ]
}

#[catch(404)]
pub fn not_found_catcher() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: false,
        message: "Resource not found".to_string(),
        data: None,
    })
}

#[catch(400)]
pub fn bad_request_catcher() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: false,
        message: "Bad request".to_string(),
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_response_serialization() {
        let response: ApiResponse<String> = ApiResponse {
            success: true,
            message: "Test message".to_string(),
            data: Some("Test data".to_string()),
        };
        
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("Test message"));
        assert!(serialized.contains("Test data"));
        
        let deserialized: ApiResponse<String> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.success, true);
        assert_eq!(deserialized.message, "Test message");
        assert_eq!(deserialized.data, Some("Test data".to_string()));
    }
}