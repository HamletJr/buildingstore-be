use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::main::model::payment::{Payment, PaymentMethod};
use crate::main::enums::payment_status::PaymentStatus;
use crate::main::service::payment_service::PaymentService;

#[derive(Deserialize)]
pub struct CreatePaymentRequest {
    transaction_id: String,
    amount: f64,
    method: String,
}

#[derive(Deserialize)]
pub struct UpdatePaymentStatusRequest {
    payment_id: String,
    new_status: String,
    additional_amount: Option<f64>,
}

#[derive(Deserialize)]
pub struct AddInstallmentRequest {
    payment_id: String,
    amount: f64,
}

#[derive(Deserialize)]
pub struct PaymentFilterRequest {
    status: Option<String>,
    method: Option<String>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

pub struct PaymentController {
    service: Arc<dyn PaymentService>,
}

impl PaymentController {
    pub fn new(service: Arc<dyn PaymentService>) -> Self {
        Self { service }
    }

    pub fn create_payment(&self, request: CreatePaymentRequest) -> ApiResponse<Payment> {
        let method = match request.method.to_uppercase().as_str() {
            "CASH" => PaymentMethod::Cash,
            "CREDIT_CARD" => PaymentMethod::CreditCard,
            "BANK_TRANSFER" => PaymentMethod::BankTransfer,
            "E_WALLET" => PaymentMethod::EWallet,
            _ => {
                return ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Metode pembayaran tidak valid".to_string()),
                }
            }
        };

        match self.service.create_payment(request.transaction_id, request.amount, method) {
            Ok(payment) => ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            },
            Err(error) => ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            },
        }
    }

    pub fn update_payment_status(&self, request: UpdatePaymentStatusRequest) -> ApiResponse<Payment> {
        let status = match PaymentStatus::from_string(&request.new_status) {
            Some(status) => status,
            None => {
                return ApiResponse {
                    success: false,
                    data: None,
                    message: Some("Status pembayaran tidak valid".to_string()),
                }
            }
        };

        match self.service.update_payment_status(
            request.payment_id,
            status,
            request.additional_amount,
        ) {
            Ok(payment) => ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            },
            Err(error) => ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            },
        }
    }

    pub fn add_installment(&self, request: AddInstallmentRequest) -> ApiResponse<Payment> {
        match self.service.add_installment(&request.payment_id, request.amount) {
            Ok(payment) => ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            },
            Err(error) => ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            },
        }
    }

    pub fn get_payment(&self, payment_id: &str) -> ApiResponse<Payment> {
        match self.service.get_payment(payment_id) {
            Some(payment) => ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            },
            None => ApiResponse {
                success: false,
                data: None,
                message: Some(format!("Pembayaran dengan ID {} tidak ditemukan", payment_id)),
            },
        }
    }

    pub fn get_payment_by_transaction(&self, transaction_id: &str) -> ApiResponse<Payment> {
        match self.service.get_payment_by_transaction(transaction_id) {
            Some(payment) => ApiResponse {
                success: true,
                data: Some(payment),
                message: None,
            },
            None => ApiResponse {
                success: false,
                data: None,
                message: Some(format!(
                    "Pembayaran untuk transaksi {} tidak ditemukan",
                    transaction_id
                )),
            },
        }
    }

    pub fn get_all_payments(&self, filter: Option<PaymentFilterRequest>) -> ApiResponse<Vec<Payment>> {
        let mut filters: Option<HashMap<String, String>> = None;
        
        if let Some(filter_req) = filter {
            let mut map = HashMap::new();
            
            if let Some(status) = filter_req.status {
                map.insert("status".to_string(), status);
            }
            
            if let Some(method) = filter_req.method {
                map.insert("method".to_string(), method);
            }
            
            if !map.is_empty() {
                filters = Some(map);
            }
        }

        let payments = self.service.get_all_payments(filters);
        
        ApiResponse {
            success: true,
            data: Some(payments),
            message: None,
        }
    }

    pub fn delete_payment(&self, payment_id: &str) -> ApiResponse<()> {
        match self.service.delete_payment(payment_id.to_string()) {
            Ok(_) => ApiResponse {
                success: true,
                data: Some(()),
                message: Some("Pembayaran berhasil dihapus".to_string()),
            },
            Err(error) => ApiResponse {
                success: false,
                data: None,
                message: Some(error),
            },
        }
    }
}