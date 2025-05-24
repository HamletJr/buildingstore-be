CREATE TABLE supplier_transactions (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    supplier_id VARCHAR(255) NOT NULL,
    supplier_name VARCHAR(255) NOT NULL,
    jenis_barang VARCHAR(255),
    jumlah_barang INTEGER NOT NULL,
    pengiriman_info VARCHAR(255),
    tanggal_transaksi TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (supplier_id) REFERENCES suppliers(id) ON DELETE CASCADE
);