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
}