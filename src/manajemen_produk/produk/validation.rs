use super::model::Produk;

pub trait ValidationRule {
    fn validate(&self, produk: &Produk) -> Result<(), String>;
}

pub struct NamaNotEmpty;
impl ValidationRule for NamaNotEmpty {
    fn validate(&self, produk: &Produk) -> Result<(), String> {
        if produk.nama.trim().is_empty() {
            Err("Nama produk tidak boleh kosong".to_string())
        } else {
            Ok(())
        }
    }
}

pub struct HargaNonNegatif;
impl ValidationRule for HargaNonNegatif {
    fn validate(&self, produk: &Produk) -> Result<(), String> {
        if produk.harga < 0.0 {
            Err("Harga tidak boleh negatif".to_string())
        } else {
            Ok(())
        }
    }
}

// Added missing validation rules
pub struct KategoriNotEmpty;
impl ValidationRule for KategoriNotEmpty {
    fn validate(&self, produk: &Produk) -> Result<(), String> {
        if produk.kategori.trim().is_empty() {
            Err("Kategori produk tidak boleh kosong".to_string())
        } else {
            Ok(())
        }
    }
}

pub struct StokNonNegatif;
impl ValidationRule for StokNonNegatif {
    fn validate(&self, _produk: &Produk) -> Result<(), String> {
        // Note: This can't actually fail since stok is u32 which is always non-negative
        // But we'll keep the validation rule for consistency
        Ok(())
    }
}

pub struct DeskripsiMaxLength;
impl ValidationRule for DeskripsiMaxLength {
    fn validate(&self, produk: &Produk) -> Result<(), String> {
        const MAX_LENGTH: usize = 500;
        
        if let Some(desc) = &produk.deskripsi {
            if desc.len() > MAX_LENGTH {
                return Err("Deskripsi terlalu panjang (maksimal 500 karakter)".to_string());
            }
        }
        
        Ok(())
    }
}

pub struct ProdukValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ProdukValidator {
    pub fn default() -> Self {
        Self {
            rules: vec![
                Box::new(NamaNotEmpty),
                Box::new(KategoriNotEmpty),
                Box::new(HargaNonNegatif),
                Box::new(StokNonNegatif),
                Box::new(DeskripsiMaxLength),
            ],
        }
    }

    pub fn validate(&self, produk: &Produk) -> Result<(), Vec<String>> {
        let errors = self.rules
            .iter()
            .filter_map(|rule| rule.validate(produk).err())
            .collect::<Vec<_>>();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}