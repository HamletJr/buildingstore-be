#[macro_use] extern crate rocket;
use dotenvy::dotenv;
use rocket::State;
use sea_orm::{Database, DatabaseConnection};
use migration::{Migrator, MigratorTrait};

pub mod auth;
pub mod manajemen_produk;
pub mod manajemen_pelanggan;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/db")]
async fn test_db(db: &State<DatabaseConnection>) -> Option<String> {
    todo!()
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db: DatabaseConnection = Database::connect(db_url).await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    rocket::build()
        .manage(reqwest::Client::builder().build().unwrap())
        .manage(db)
        // .attach(auth::controller::route_stage())
        .attach(manajemen_pelanggan::controller::route_stage())
        .mount("/", routes![index, test_db])
}