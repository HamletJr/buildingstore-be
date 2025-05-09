use rocket::{get, post, put, delete};
use rocket::serde::json::Json;
use rocket::response::status::{Created, NotFound, BadRequest};
use rocket::State;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::manajemen_pembayaran::controller::payment_controller::{
    PaymentController, CreatePaymentRequest, UpdatePaymentStatusRequest, 
    AddInstallmentRequest, PaymentFilterRequest, ApiResponse
};
use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod};
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;

#[derive(Deserialize)]
pub struct PaymentRequest {
    pub transactionId: i32,
    pub amount: f64,
    pub paymentMethod: String,
    pub paymentProof: Option<String>,
}

#[derive(Serialize)]
pub struct ApiSuccessResponse<T> {
    pub status: String,
    pub data: T,
}

#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub status: String,
    pub code: i32,
    pub message: String,
}

#[derive(Serialize)]
pub struct PaymentResponse {
    pub id: String,
    pub transactionId: i32,
    pub amount: f64,
    pub paymentMethod: String,
    pub paymentDate: DateTime<Utc>,
    pub status: String,
    pub paymentProof: Option<String>,
    pub receiptUrl: String,
}

impl From<Payment> for PaymentResponse {
    fn from(payment: Payment) -> Self {
        let status_str = match payment.status {
            PaymentStatus::Paid => "LUNAS",
            PaymentStatus::Installment => "CICILAN",
            _ => "PENDING",
        };

        let method_str = match payment.method {
            PaymentMethod::Cash => "CASH",
            PaymentMethod::CreditCard => "CARD",
            PaymentMethod::BankTransfer => "TRANSFER",
            PaymentMethod::EWallet => "E_WALLET",
        };

        let transaction_id = payment.transaction_id
            .replace("TRX-", "")
            .parse::<i32>()
            .unwrap_or(0);

        PaymentResponse {
            id: payment.id,
            transactionId: transaction_id,
            amount: payment.amount,
            paymentMethod: method_str.to_string(),
            paymentDate: payment.created_at,
            status: status_str.to_string(),
            paymentProof: payment.payment_proof,
            receiptUrl: format!("https://buildingstore.com/receipts/{}.pdf", payment.id),
        }
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        create_payment,
        get_payment_by_id,
        get_payment_by_transaction,
        get_all_payments,
        update_payment_status,
        add_installment,
        delete_payment
    ]
}

#[post("/api/payments", format = "application/json", data = "<request>")]
fn create_payment(
    controller: &State<PaymentController>,
    request: Json<PaymentRequest>
) -> Result<Created<Json<ApiSuccessResponse<PaymentResponse>>>, Json<ApiErrorResponse>> {
    if !["CASH", "CARD", "TRANSFER", "E_WALLET"].contains(&request.paymentMethod.as_str()) {
        return Err(Json(ApiErrorResponse {
            status: "error".to_string(),
            code: 4003,
            message: "Invalid payment method".to_string(),
        }));
    }

    if request.paymentMethod != "CASH" && request.paymentProof.is_none() {
        return Err(Json(ApiErrorResponse {
            status: "error".to_string(),
            code: 4004,
            message: "Payment proof required for selected method".to_string(),
        }));
    }

    let transaction_id = format!("TRX-{}", request.transactionId);

    let method = match request.paymentMethod.as_str() {
        "CASH" => "CASH",
        "CARD" => "CREDIT_CARD",
        "TRANSFER" => "BANK_TRANSFER",
        "E_WALLET" => "E_WALLET",
        _ => "CASH",
    };

    let create_request = CreatePaymentRequest {
        transaction_id,
        amount: request.amount,
        method: method.to_string(),
    };

    let response = controller.create_payment(create_request);

    match response.success {
        true => {
            let payment = response.data.unwrap();
            if let Some(proof) = &request.paymentProof {
            }
            
            let payment_response = PaymentResponse::from(payment);
            
            Ok(Created::new("/").body(Json(ApiSuccessResponse {
                status: "success".to_string(),
                data: payment_response,
            })))
        },
        false => {
            let error_code = if response.message.as_ref().unwrap().contains("tidak valid") {
                4003 // Invalid payment method
            } else if response.message.as_ref().unwrap().contains("transaksi") {
                4001 // Transaction ID not found
            } else {
                4002 // Invalid payment amount (default)
            };

            Err(Json(ApiErrorResponse {
                status: "error".to_string(),
                code: error_code,
                message: response.message.unwrap(),
            }))
        }
    }
}

#[get("/api/payments/<payment_id>")]
fn get_payment_by_id(
    controller: &State<PaymentController>,
    payment_id: String
) -> Result<Json<ApiSuccessResponse<PaymentResponse>>, NotFound<Json<ApiErrorResponse>>> {
    let response = controller.get_payment(&payment_id);

    match response.success {
        true => {
            let payment = response.data.unwrap();
            let payment_response = PaymentResponse::from(payment);
            
            Ok(Json(ApiSuccessResponse {
                status: "success".to_string(),
                data: payment_response,
            }))
        },
        false => {
            Err(NotFound(Json(ApiErrorResponse {
                status: "error".to_string(),
                code: 4001,
                message: response.message.unwrap_or_else(|| "Payment not found".to_string()),
            })))
        }
    }
}

#[get("/api/payments/transaction/<transaction_id>")]
fn get_payment_by_transaction(
    controller: &State<PaymentController>,
    transaction_id: String
) -> Result<Json<ApiSuccessResponse<PaymentResponse>>, NotFound<Json<ApiErrorResponse>>> {
    let formatted_transaction_id = format!("TRX-{}", transaction_id);
    let response = controller.get_payment_by_transaction(&formatted_transaction_id);

    match response.success {
        true => {
            let payment = response.data.unwrap();
            let payment_response = PaymentResponse::from(payment);
            
            Ok(Json(ApiSuccessResponse {
                status: "success".to_string(),
                data: payment_response,
            }))
        },
        false => {
            Err(NotFound(Json(ApiErrorResponse {
                status: "error".to_string(),
                code: 4001,
                message: response.message.unwrap_or_else(|| "Payment not found".to_string()),
            })))
        }
    }
}

#[get("/api/payments?<status>&<method>")]
fn get_all_payments(
    controller: &State<PaymentController>,
    status: Option<String>,
    method: Option<String>
) -> Json<ApiSuccessResponse<Vec<PaymentResponse>>> {
    let filter = if status.is_some() || method.is_some() {
        Some(PaymentFilterRequest {
            status,
            method,
        })
    } else {
        None
    };

    let response = controller.get_all_payments(filter);
    let payments = response.data.unwrap_or_default();
    
    let payment_responses: Vec<PaymentResponse> = payments
        .into_iter()
        .map(PaymentResponse::from)
        .collect();

    Json(ApiSuccessResponse {
        status: "success".to_string(),
        data: payment_responses,
    })
}

#[derive(Deserialize)]
pub struct StatusUpdateRequest {
    pub new_status: String,
    pub additional_amount: Option<f64>,
}

#[put("/api/payments/<payment_id>/status", data = "<request>")]
fn update_payment_status(
    controller: &State<PaymentController>,
    payment_id: String,
    request: Json<StatusUpdateRequest>
) -> Result<Json<ApiSuccessResponse<PaymentResponse>>, BadRequest<Json<ApiErrorResponse>>> {
    let update_request = UpdatePaymentStatusRequest {
        payment_id: payment_id.clone(),
        new_status: request.new_status.clone(),
        additional_amount: request.additional_amount,
    };

    let response = controller.update_payment_status(update_request);

    match response.success {
        true => {
            let payment = response.data.unwrap();
            let payment_response = PaymentResponse::from(payment);
            
            Ok(Json(ApiSuccessResponse {
                status: "success".to_string(),
                data: payment_response,
            }))
        },
        false => {
            Err(BadRequest(Json(ApiErrorResponse {
                status: "error".to_string(),
                code: 4001, // Generic error code
                message: response.message.unwrap_or_else(|| "Failed to update payment status".to_string()),
            })))
        }
    }
}

#[derive(Deserialize)]
pub struct InstallmentRequest {
    pub amount: f64,
}

#[post("/api/payments/<payment_id>/installment", data = "<request>")]
fn add_installment(
    controller: &State<PaymentController>,
    payment_id: String,
    request: Json<InstallmentRequest>
) -> Result<Json<ApiSuccessResponse<PaymentResponse>>, BadRequest<Json<ApiErrorResponse>>> {
    let add_request = AddInstallmentRequest {
        payment_id: payment_id.clone(),
        amount: request.amount,
    };

    let response = controller.add_installment(add_request);

    match response.success {
        true => {
            let payment = response.data.unwrap();
            let payment_response = PaymentResponse::from(payment);
            
            Ok(Json(ApiSuccessResponse {
                status: "success".to_string(),
                data: payment_response,
            }))
        },
        false => {
            Err(BadRequest(Json(ApiErrorResponse {
                status: "error".to_string(),
                code: 4001, // Generic error code
                message: response.message.unwrap_or_else(|| "Failed to add installment".to_string()),
            })))
        }
    }
}

#[delete("/api/payments/<payment_id>")]
fn delete_payment(
    controller: &State<PaymentController>,
    payment_id: String
) -> Result<Json<ApiSuccessResponse<String>>, NotFound<Json<ApiErrorResponse>>> {
    let response = controller.delete_payment(&payment_id);

    match response.success {
        true => {
            Ok(Json(ApiSuccessResponse {
                status: "success".to_string(),
                data: "Payment successfully deleted".to_string(),
            }))
        },
        false => {
            Err(NotFound(Json(ApiErrorResponse {
                status: "error".to_string(),
                code: 4001,
                message: response.message.unwrap_or_else(|| "Payment not found".to_string()),
            })))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
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
}