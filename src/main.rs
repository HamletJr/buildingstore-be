#[macro_use] extern crate rocket;
use rocket_db_pools::Database;
use rocket_db_pools::sqlx::{self, Row};
use buildingstore_be::{BuildingStoreDB};
use dotenvy::dotenv;
use sqlx::any::install_default_drivers;
use rocket::State;
use sqlx::{Any, Pool};
use rocket_cors::{AllowedOrigins, CorsOptions};

pub mod auth;
pub mod manajemen_produk;
pub mod manajemen_pelanggan;
pub mod transaksi_penjualan;
pub mod manajemen_pembayaran;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/db")]
async fn test_db(db: &State<Pool<Any>>) -> Option<String> {
    let mut db_conn = db.acquire().await.unwrap();
    let row = sqlx::query("SELECT * FROM users LIMIT 1")
        .fetch_one(&mut *db_conn)
        .await
        .unwrap();

    let id: i64 = row.get("id");
    let email: String = row.get("username");

    Some(format!("Hello, {}! Your ID is {}.", email, id))
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    // CORS Configuration
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::some_exact(&[
            "http://127.0.0.1:3000",
            "https://your-production-domain.com",
        ]))
        .allow_credentials(true)
        .to_cors()
        .expect("Failed to create CORS");

    install_default_drivers();
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = sqlx::AnyPool::connect(&database_url).await.unwrap();
    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");    rocket::build()
        .manage(reqwest::Client::builder().build().unwrap())
        .manage(db_pool)
        .attach(cors)
        .attach(BuildingStoreDB::init())
        .attach(auth::controller::route_stage())
        .attach(manajemen_pelanggan::controller::route_stage())
        .attach(manajemen_pembayaran::controller::route_stage())
        .attach(manajemen_pelanggan::controller::route_stage())
        .attach(transaksi_penjualan::controller::route_stage())
        .mount("/", routes![index, test_db])
}