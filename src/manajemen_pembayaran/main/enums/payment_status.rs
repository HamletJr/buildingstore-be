// Enum untuk status pembayaran
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentStatus {
    Paid,    // LUNAS
    Installment,  // CICILAN
}

impl PaymentStatus {
    // Memeriksa apakah string input merupakan status yang valid
    pub fn from_string(status: &str) -> Option<Self> {
        match status.to_uppercase().as_str() {
            "LUNAS" => Some(PaymentStatus::Paid),
            "CICILAN" => Some(PaymentStatus::Installment),
            _ => None,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            PaymentStatus::Paid => "LUNAS".to_string(),
            PaymentStatus::Installment => "CICILAN".to_string(),
        }
    }
}