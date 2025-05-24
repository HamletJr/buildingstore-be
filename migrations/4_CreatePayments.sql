CREATE TABLE payments (
    id TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL,
    amount REAL NOT NULL,
    method TEXT NOT NULL,
    status TEXT NOT NULL,
    payment_date TEXT NOT NULL,
    due_date TEXT
);

CREATE TABLE installments (
    id TEXT PRIMARY KEY,
    payment_id TEXT NOT NULL,
    amount REAL NOT NULL,
    payment_date TEXT NOT NULL,
    FOREIGN KEY (payment_id) REFERENCES payments(id) ON DELETE CASCADE
);
