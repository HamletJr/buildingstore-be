use crate::main::model::supplier::Supplier;
use crate::main::model::supplier_transaction::SupplierTransaction;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_create_supplier() {
    let supplier = Supplier::new(
        "PT. Beton".to_string(),
        "semen".to_string(),
        1000,
        "2306206282".to_string(),
    );

    assert_eq!(supplier.name, "PT. Beton");
    assert_eq!(supplier.jenis_barang, "semen");
    assert_eq!(supplier.jumlah_barang, 1000);
    assert_eq!(supplier.pengiriman_info, "2306206282");
}