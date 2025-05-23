#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use crate::manajemen_supplier::main::model::supplier::Supplier;
    use crate::manajemen_supplier::main::repository::supplier_repository::SupplierRepository;
    use crate::manajemen_supplier::main::repository::supplier_repository_impl::SupplierRepositoryImpl;
     
    #[tokio::test]
    async fn test_save_supplier(){
        let repository = SupplierRepositoryImpl::new();
        let supplier_id = format!("SUP-{}", Uuid::new_v4());

        let supplier = Supplier {
            id: supplier_id.clone(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now(),
        };

        let result = repository.save(supplier.clone()).await;
        let saved_supplier = result.unwrap();

        assert_eq!(saved_supplier.id, supplier_id);
        assert_eq!(saved_supplier.name, "PT. Ayam");
        assert_eq!(saved_supplier.jenis_barang, "ayam");
        assert_eq!(saved_supplier.jumlah_barang, 1000);
        assert_eq!(saved_supplier.resi, "2306206282");
    }
    
    #[tokio::test]
    async fn test_find_supplier_by_id(){
        let repository = SupplierRepositoryImpl::new();
        let supplier_id = format!("SUP-{}", Uuid::new_v4());
        
        let supplier = Supplier {
            id: supplier_id.clone(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now(),
        };
        
        repository.save(supplier.clone()).await.unwrap();
        
        let result = repository.find_by_id(&supplier_id).await;
        
        assert!(result.is_some());
        let found_supplier = result.unwrap();
        assert_eq!(found_supplier.id, supplier_id);
    }

    #[tokio::test]
    async fn test_update_supplier(){
        let repository = SupplierRepositoryImpl::new();
        let supplier_id = format!("SUP-{}", Uuid::new_v4());
        
        let supplier = Supplier {
            id: supplier_id.clone(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now(),
        };
        
        repository.save(supplier.clone()).await.unwrap();
        
        let mut updated_supplier = supplier.clone();
        updated_supplier.jumlah_barang = 1;
        let result = repository.update(updated_supplier).await;
        
        assert!(result.is_ok());
        let found_supplier = repository.find_by_id(&supplier_id).await.unwrap();
        assert_eq!(found_supplier.jumlah_barang, 1);
    }

    #[tokio::test]
    async fn test_delete_supplier(){
        let repository = SupplierRepositoryImpl::new();
        let supplier_id = format!("SUP-{}", Uuid::new_v4());
        
        let supplier = Supplier {
            id: supplier_id.clone(),
            name: "PT. Ayam".to_string(),
            jenis_barang: "ayam".to_string(),
            jumlah_barang: 1000,
            resi: "2306206282".to_string(),
            updated_at: Utc::now(),
        };
        
        repository.save(supplier.clone()).await.unwrap();

        let result = repository.delete(&supplier_id).await;
    
        assert!(result.is_ok());
        assert!(repository.find_by_id(&supplier_id).await.is_none());
    }
}