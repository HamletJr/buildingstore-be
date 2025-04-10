use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Pelanggan {
    pub id: usize,
    pub nama: String,
    pub alamat: String,
    pub no_telp: String,
    pub tanggal_gabung: chrono::NaiveDate,
}

impl Pelanggan {
    pub fn new(nama: String, alamat: String, no_telp: String) -> Self {
        Pelanggan {
            id: 0,
            nama,
            alamat,
            no_telp,
            tanggal_gabung: NaiveDate::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_pelanggan() {
        let pelanggan = Pelanggan::new( "Castorice".to_string(), "Amphoreus".to_string(), "1234567890".to_string());
        assert_eq!(pelanggan.nama, "Castorice");
        assert_eq!(pelanggan.alamat, "Amphoreus");
        assert_eq!(pelanggan.no_telp, "1234567890");
        assert_eq!(pelanggan.tanggal_gabung, chrono::Utc::now().date_naive());
    }
}