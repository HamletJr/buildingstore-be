use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

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

#[derive(Serialize)]
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

    #[test]
    fn test_delete_payment_not_found() {
        let controller = setup_controller();

        let response = controller.delete_payment("non-existent-id");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.message.is_some());
        assert_eq!(response.message.unwrap(), "Payment not found");
    }
}