#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_payment_processor_factory() {
        let transaction_id = format!("TRX-{}", Uuid::new_v4());
        
        for method in [
            PaymentMethod::Cash,
            PaymentMethod::CreditCard,
            PaymentMethod::BankTransfer,
            PaymentMethod::EWallet,
        ] {
            let processor = PaymentProcessorFactory::create(method.clone());
            
            assert_eq!(processor.get_method(), method);
            
            let result = processor.process(1000.0, &transaction_id);
            assert!(result.is_ok());
            
            let invalid_result = processor.process(-100.0, &transaction_id);
            assert!(invalid_result.is_err());
        }
    }
}