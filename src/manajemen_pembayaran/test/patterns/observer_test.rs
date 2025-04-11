use std::sync::{Arc, Mutex};
use chrono::Utc;
use uuid::Uuid;

use crate::main::model::payment::{Payment, PaymentMethod};
use crate::main::enums::payment_status::PaymentStatus;
use crate::main::patterns::observer::{PaymentObserver, PaymentSubject};

struct MockObserver {
    notification_count: Mutex<usize>,
    last_payment_id: Mutex<Option<String>>,
}

impl MockObserver {
    fn new() -> Self {
        Self {
            notification_count: Mutex::new(0),
            last_payment_id: Mutex::new(None),
        }
    }
    
    fn get_notification_count(&self) -> usize {
        *self.notification_count.lock().unwrap()
    }
    
    fn get_last_payment_id(&self) -> Option<String> {
        self.last_payment_id.lock().unwrap().clone()
    }
}

impl PaymentObserver for MockObserver {
    fn payment_updated(&self, payment: &Payment) {
        let mut count = self.notification_count.lock().unwrap();
        *count += 1;
        
        let mut last_id = self.last_payment_id.lock().unwrap();
        *last_id = Some(payment.id.clone());
    }
}

#[test]
fn test_observer_notification() {
    let mut subject = PaymentSubject::new();
    let observer = Arc::new(MockObserver::new());
    
    subject.attach(observer.clone() as Arc<dyn PaymentObserver>);
    
    let payment = Payment {
        id: format!("PMT-{}", Uuid::new_v4()),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Paid,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    subject.notify(&payment);
    
    assert_eq!(observer.get_notification_count(), 1);
    assert_eq!(observer.get_last_payment_id(), Some(payment.id.clone()));
}

#[test]
fn test_multiple_observers() {
    let mut subject = PaymentSubject::new();
    let observer1 = Arc::new(MockObserver::new());
    let observer2 = Arc::new(MockObserver::new());
    
    subject.attach(observer1.clone() as Arc<dyn PaymentObserver>);
    subject.attach(observer2.clone() as Arc<dyn PaymentObserver>);
    
    let payment = Payment {
        id: format!("PMT-{}", Uuid::new_v4()),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Paid,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    subject.notify(&payment);
    
    assert_eq!(observer1.get_notification_count(), 1);
    assert_eq!(observer2.get_notification_count(), 1);
}

#[test]
fn test_concrete_observers() {
    let mut subject = PaymentSubject::new();
    
    let transaction_observer = Arc::new(crate::main::patterns::observer::TransactionObserver);
    let inventory_observer = Arc::new(crate::main::patterns::observer::InventoryObserver);
    let customer_observer = Arc::new(crate::main::patterns::observer::CustomerObserver);
    
    subject.attach(transaction_observer as Arc<dyn PaymentObserver>);
    subject.attach(inventory_observer as Arc<dyn PaymentObserver>);
    subject.attach(customer_observer as Arc<dyn PaymentObserver>);
    
    let payment = Payment {
        id: format!("PMT-{}", Uuid::new_v4()),
        transaction_id: format!("TRX-{}", Uuid::new_v4()),
        amount: 1000.0,
        method: PaymentMethod::Cash,
        status: PaymentStatus::Paid,
        payment_date: Utc::now(),
        installments: Vec::new(),
        due_date: None,
    };
    
    subject.notify(&payment);
    
    assert!(true);
}