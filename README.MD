# BuildingStore - Modul Manajemen Pembayaran

Implementasi modul Manajemen Pembayaran untuk aplikasi BuildingStore dengan pendekatan TDD (Test-Driven Development) dan menggunakan berbagai Design Pattern.

## Design Pattern yang Digunakan

### 1. Strategy Pattern
**Tujuan:** Memungkinkan penggunaan algoritma berbeda untuk memproses pembayaran berdasarkan metode pembayaran yang dipilih.

**Implementasi:**
- Interface: `PaymentProcessor` trait
- Concrete Implementations: `CashPaymentProcessor`, `CreditCardPaymentProcessor`, `BankTransferPaymentProcessor`, `EWalletPaymentProcessor`

**Manfaat:**
- Memisahkan logika proses pembayaran berdasarkan metode
- Memudahkan penambahan metode pembayaran baru
- Memungkinkan pilihan metode pembayaran secara dinamis

### 2. Factory Pattern
**Tujuan:** Menyediakan cara untuk membuat objek processor pembayaran dan state pembayaran.

**Implementasi:**
- `PaymentProcessorFactory` - Factory untuk membuat `PaymentProcessor`
- `PaymentStateFactory` - Factory untuk membuat `PaymentState`

**Manfaat:**
- Menyembunyikan logika pembuatan objek
- Memudahkan perubahan implementasi objek
- Menciptakan objek berdasarkan parameter

### 3. Observer Pattern
**Tujuan:** Memberitahu modul terkait ketika terjadi perubahan status pembayaran.

**Implementasi:**
- Interface: `PaymentObserver` trait
- Concrete Implementations: `TransactionObserver`, `InventoryObserver`, `CustomerObserver`
- Subject: `PaymentSubject`

**Manfaat:**
- Memungkinkan komunikasi antar modul secara loose coupling
- Memastikan konsistensi data saat status pembayaran berubah
- Mendukung prinsip Single Responsibility

### 4. State Pattern
**Tujuan:** Mengelola perilaku berbeda berdasarkan status pembayaran.

**Implementasi:**
- Interface: `PaymentState` trait
- Concrete Implementations: `PaidState`, `InstallmentState`

**Manfaat:**
- Memisahkan perilaku berdasarkan status
- Menghindari kondisional yang kompleks
- Memudahkan penambahan status baru

### 5. Command Pattern
**Tujuan:** Mengenkapsulasi operasi pembayaran sebagai objek.

**Implementasi:**
- Interface: `PaymentCommand` trait
- Concrete Implementations: `CreatePaymentCommand`, `UpdatePaymentStatusCommand`, `AddInstallmentCommand`, `DeletePaymentCommand`

**Manfaat:**
- Memisahkan objek yang mengeluarkan perintah dari objek yang melaksanakannya
- Memungkinkan penyimpanan riwayat perintah
- Memfasilitasi operasi kompleks

### 6. Repository Pattern
**Tujuan:** Menyediakan abstraksi untuk akses data pembayaran.

**Implementasi:**
- Interface: `PaymentRepository` trait
- Concrete Implementation: `InMemoryPaymentRepository`

**Manfaat:**
- Memisahkan logika bisnis dari akses data
- Memudahkan pengujian dengan mock repository
- Menyediakan operasi CRUD yang konsisten
