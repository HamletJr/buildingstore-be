use rocket::{get, post, patch, delete, put};
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use sqlx::{Any, Pool};
use crate::transaksi_penjualan::model::transaksi::Transaksi;
use crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi;
use crate::transaksi_penjualan::service::transaksi::TransaksiService;
use crate::transaksi_penjualan::enums::status_transaksi::StatusTransaksi;

#[get("/transaksi?<sort>&<filter>&<keyword>&<status>&<id_pelanggan>")]
pub async fn get_all_transaksi(
    db: &State<Pool<Any>>, 
    sort: Option<String>, 
    filter: Option<String>, 
    keyword: Option<String>,
    status: Option<String>,
    id_pelanggan: Option<i32>
) -> Result<Json<Vec<Transaksi>>, Status> {
    let mut transaksi_list = if let Some(customer_id) = id_pelanggan {
        TransaksiService::get_transaksi_by_pelanggan(db.inner().clone(), customer_id)
            .await
            .map_err(|_| Status::InternalServerError)?
    } else if let Some(status_str) = status {
        if let Some(status_enum) = StatusTransaksi::from_string(&status_str) {
            TransaksiService::get_transaksi_by_status(db.inner().clone(), &status_enum)
                .await
                .map_err(|_| Status::InternalServerError)?
        } else {
            return Err(Status::BadRequest);
        }
    } else {
        TransaksiService::get_all_transaksi(db.inner().clone())
            .await
            .map_err(|_| Status::InternalServerError)?
    };

    if let Some(sort_strategy) = &sort {
        transaksi_list = TransaksiService::sort_transaksi(transaksi_list, sort_strategy);
    }

    if let Some(filter_strategy) = &filter {
        if let Some(keyword_value) = &keyword {
            transaksi_list = TransaksiService::filter_transaksi(transaksi_list, filter_strategy, keyword_value);
        }
    }

    Ok(Json(transaksi_list))
}

#[post("/transaksi", data = "<request>")]
pub async fn create_transaksi(
    db: &State<Pool<Any>>, 
    request: Json<crate::transaksi_penjualan::dto::transaksi_request::CreateTransaksiRequest>
) -> Result<Json<Transaksi>, Status> {
    if let Err(_err_msg) = TransaksiService::validate_product_stock(&request.detail_transaksi).await {
        return Err(Status::BadRequest);
    }

    let new_transaksi = TransaksiService::create_transaksi_with_details(db.inner().clone(), &request)
        .await
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(Json(new_transaksi))
}

#[get("/transaksi/<id>")]
pub async fn get_transaksi_by_id(db: &State<Pool<Any>>, id: i32) -> Result<Json<Transaksi>, Status> {
    let transaksi = TransaksiService::get_transaksi_by_id(db.inner().clone(), id)
        .await
        .map_err(|_| Status::NotFound)?;
    Ok(Json(transaksi))
}

#[patch("/transaksi/<id>", data = "<transaksi>")]
pub async fn update_transaksi(db: &State<Pool<Any>>, id: i32, transaksi: Json<Transaksi>) -> Result<Json<Transaksi>, Status> {
    if transaksi.id != id {
        return Err(Status::BadRequest);
    }

    let updated_transaksi = TransaksiService::update_transaksi(db.inner().clone(), &transaksi)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Json(updated_transaksi))
}

#[delete("/transaksi/<id>")]
pub async fn delete_transaksi(db: &State<Pool<Any>>, id: i32) -> Result<Status, Status> {
    TransaksiService::delete_transaksi(db.inner().clone(), id)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Status::NoContent)
}

#[put("/transaksi/<id>/complete")]
pub async fn complete_transaksi(db: &State<Pool<Any>>, id: i32) -> Result<Json<Transaksi>, Status> {
    let completed_transaksi = TransaksiService::complete_transaksi(db.inner().clone(), id)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Json(completed_transaksi))
}

#[put("/transaksi/<id>/cancel")]
pub async fn cancel_transaksi(db: &State<Pool<Any>>, id: i32) -> Result<Json<Transaksi>, Status> {
    let cancelled_transaksi = TransaksiService::cancel_transaksi(db.inner().clone(), id)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Json(cancelled_transaksi))
}

#[get("/transaksi/<id_transaksi>/detail")]
pub async fn get_detail_transaksi(db: &State<Pool<Any>>, id_transaksi: i32) -> Result<Json<Vec<DetailTransaksi>>, Status> {
    let detail_list = TransaksiService::get_detail_by_transaksi_id(db.inner().clone(), id_transaksi)
        .await
        .map_err(|_| Status::NotFound)?;
    Ok(Json(detail_list))
}

#[post("/transaksi/<id_transaksi>/detail", data = "<detail>")]
pub async fn add_detail_transaksi(
    db: &State<Pool<Any>>, 
    id_transaksi: i32, 
    detail: Json<DetailTransaksi>
) -> Result<Json<DetailTransaksi>, Status> {
    if detail.id_transaksi != id_transaksi {
        return Err(Status::BadRequest);
    }

    let new_detail = TransaksiService::add_detail_transaksi(db.inner().clone(), &detail)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Json(new_detail))
}

#[patch("/transaksi/<id_transaksi>/detail/<id_detail>", data = "<detail>")]
pub async fn update_detail_transaksi(
    db: &State<Pool<Any>>, 
    id_transaksi: i32, 
    id_detail: i32, 
    detail: Json<DetailTransaksi>
) -> Result<Json<DetailTransaksi>, Status> {
    if detail.id != id_detail || detail.id_transaksi != id_transaksi {
        return Err(Status::BadRequest);
    }

    let updated_detail = TransaksiService::update_detail_transaksi(db.inner().clone(), &detail)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Json(updated_detail))
}

#[delete("/transaksi/<id_transaksi>/detail/<id_detail>")]
pub async fn delete_detail_transaksi(
    db: &State<Pool<Any>>, 
    id_transaksi: i32, 
    id_detail: i32
) -> Result<Status, Status> {
    TransaksiService::delete_detail_transaksi(db.inner().clone(), id_detail, id_transaksi)
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Status::NoContent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::{routes, uri, Rocket, async_test};
    use sqlx::any::install_default_drivers;
    use crate::transaksi_penjualan::model::transaksi::Transaksi;
    use crate::transaksi_penjualan::model::detail_transaksi::DetailTransaksi;

    async fn setup() -> Rocket<rocket::Build> {
        install_default_drivers();
        let db = sqlx::any::AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("migrations/test")
            .run(&db)
            .await
            .unwrap();

        let rocket = rocket::build()
            .manage(db.clone())
            .mount("/", routes![
                get_all_transaksi, create_transaksi, get_transaksi_by_id, 
                update_transaksi, delete_transaksi, complete_transaksi, cancel_transaksi,
                get_detail_transaksi, add_detail_transaksi, update_detail_transaksi, delete_detail_transaksi
            ]);
        rocket
    }

    #[async_test]
    async fn test_create_transaksi() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi_request = crate::transaksi_penjualan::dto::transaksi_request::CreateTransaksiRequest {
            id_pelanggan: 1,
            nama_pelanggan: "Castorice".to_string(),
            catatan: Some("Test transaction".to_string()),
            detail_transaksi: vec![
                crate::transaksi_penjualan::dto::transaksi_request::CreateDetailTransaksiRequest {
                    id_produk: 1,
                    nama_produk: "Contoh Produk".to_string(),
                    harga_satuan: 10000.0,
                    jumlah: 2,
                },
            ],
        };


        let response = client.post(uri!(super::create_transaksi))
            .json(&new_transaksi_request)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Transaksi>().await.unwrap();
        assert_eq!(body.nama_pelanggan, new_transaksi_request.nama_pelanggan);
        assert!(body.total_harga > 0.0);
    }

    #[async_test]
    async fn test_get_all_transaksi() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let transaksi1 = Transaksi::new(1, "Castorice".to_string(), 150000.0, None);
        let transaksi2 = Transaksi::new(2, "Tribbie".to_string(), 200000.0, None);

        client.post(uri!(super::create_transaksi)).json(&transaksi1).dispatch().await;
        client.post(uri!(super::create_transaksi)).json(&transaksi2).dispatch().await;

        let response = client.get("/transaksi").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Transaksi>>().await.unwrap();
        assert_eq!(body.len(), 2);
    }

    #[async_test]
    async fn test_get_transaksi_by_id() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi = Transaksi::new(1, "Hyacine".to_string(), 300000.0, None);

        let response = client.post(uri!(super::create_transaksi))
            .json(&new_transaksi)
            .dispatch()
            .await;
        
        let created_transaksi = response.into_json::<Transaksi>().await.unwrap();

        let response = client.get(uri!(super::get_transaksi_by_id(created_transaksi.id))).dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Transaksi>().await.unwrap();
        assert_eq!(body.nama_pelanggan, new_transaksi.nama_pelanggan);
    }

    #[async_test]
    async fn test_complete_transaksi() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi = Transaksi::new(1, "Aglaea".to_string(), 500000.0, None);

        let response = client.post(uri!(super::create_transaksi))
            .json(&new_transaksi)
            .dispatch()
            .await;
        
        let created_transaksi = response.into_json::<Transaksi>().await.unwrap();

        let response = client.put(uri!(super::complete_transaksi(created_transaksi.id))).dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Transaksi>().await.unwrap();
        assert_eq!(body.status, StatusTransaksi::Selesai);
    }

    #[async_test]
    async fn test_cancel_transaksi() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi = Transaksi::new(2, "Polyxia".to_string(), 75000.0, None);

        let response = client.post(uri!(super::create_transaksi))
            .json(&new_transaksi)
            .dispatch()
            .await;
        
        let created_transaksi = response.into_json::<Transaksi>().await.unwrap();

        let response = client.put(uri!(super::cancel_transaksi(created_transaksi.id))).dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Transaksi>().await.unwrap();
        assert_eq!(body.status, StatusTransaksi::Dibatalkan);
    }

    #[async_test]
    async fn test_add_detail_transaksi() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi = Transaksi::new(1, "Pollux".to_string(), 0.0, None);

        let response = client.post(uri!(super::create_transaksi))
            .json(&new_transaksi)
            .dispatch()
            .await;
        
        let created_transaksi = response.into_json::<Transaksi>().await.unwrap();

        let detail = DetailTransaksi::new(
            created_transaksi.id,
            101,
            15000000.0,
            1,
        );

        let response = client.post(uri!(super::add_detail_transaksi(created_transaksi.id)))
            .json(&detail)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<DetailTransaksi>().await.unwrap();
        assert_eq!(body.subtotal, 15000000.0);
    }

    #[async_test]
    async fn test_get_detail_transaksi() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi = Transaksi::new(1, "Anaxa".to_string(), 0.0, None);

        let response = client.post(uri!(super::create_transaksi))
            .json(&new_transaksi)
            .dispatch()
            .await;
        
        let created_transaksi = response.into_json::<Transaksi>().await.unwrap();

        let detail1 = DetailTransaksi::new(created_transaksi.id, 101, 250000.0, 2);
        let detail2 = DetailTransaksi::new(created_transaksi.id, 102, 500000.0, 1);

        client.post(uri!(super::add_detail_transaksi(created_transaksi.id))).json(&detail1).dispatch().await;
        client.post(uri!(super::add_detail_transaksi(created_transaksi.id))).json(&detail2).dispatch().await;

        let response = client.get(uri!(super::get_detail_transaksi(created_transaksi.id))).dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<DetailTransaksi>>().await.unwrap();
        assert_eq!(body.len(), 2);
    }

    #[async_test]
    async fn test_filter_transaksi_by_status() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let transaksi1 = Transaksi::new(1, "Alice".to_string(), 100000.0, None);
        let transaksi2 = Transaksi::new(2, "Bob".to_string(), 200000.0, None);

        let response1 = client.post(uri!(super::create_transaksi)).json(&transaksi1).dispatch().await;
        let created1 = response1.into_json::<Transaksi>().await.unwrap();
        
        let response2 = client.post(uri!(super::create_transaksi)).json(&transaksi2).dispatch().await;
        let _created2 = response2.into_json::<Transaksi>().await.unwrap();

        client.put(uri!(super::complete_transaksi(created1.id))).dispatch().await;

        let response = client.get("/transaksi?status=MASIH_DIPROSES").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Transaksi>>().await.unwrap();
        assert_eq!(body.len(), 1);
        assert_eq!(body[0].nama_pelanggan, "Bob");
    }

    #[async_test]
    async fn test_sort_transaksi_by_nama() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let transaksi1 = Transaksi::new(1, "Charlie".to_string(), 100000.0, None);
        let transaksi2 = Transaksi::new(2, "Alice".to_string(), 200000.0, None);
        let transaksi3 = Transaksi::new(3, "Bob".to_string(), 150000.0, None);

        client.post(uri!(super::create_transaksi)).json(&transaksi1).dispatch().await;
        client.post(uri!(super::create_transaksi)).json(&transaksi2).dispatch().await;
        client.post(uri!(super::create_transaksi)).json(&transaksi3).dispatch().await;

        let response = client.get("/transaksi?sort=pelanggan").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Transaksi>>().await.unwrap();
        assert_eq!(body[0].nama_pelanggan, "Alice");
        assert_eq!(body[1].nama_pelanggan, "Bob");
        assert_eq!(body[2].nama_pelanggan, "Charlie");
    }

    #[async_test]
    async fn test_filter_transaksi_by_nama() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let transaksi1 = Transaksi::new(1, "Alice Smith".to_string(), 100000.0, None);
        let transaksi2 = Transaksi::new(2, "Bob Johnson".to_string(), 200000.0, None);
        let transaksi3 = Transaksi::new(3, "Alice Brown".to_string(), 150000.0, None);

        client.post(uri!(super::create_transaksi)).json(&transaksi1).dispatch().await;
        client.post(uri!(super::create_transaksi)).json(&transaksi2).dispatch().await;
        client.post(uri!(super::create_transaksi)).json(&transaksi3).dispatch().await;

        let response = client.get("/transaksi?filter=pelanggan&keyword=Alice").dispatch().await;

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_json::<Vec<Transaksi>>().await.unwrap();
        assert_eq!(body.len(), 2);
        assert!(body.iter().all(|t| t.nama_pelanggan.contains("Alice")));
    }

    #[async_test]
    async fn test_get_transaksi_by_id_not_found() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let response = client.get(uri!(super::get_transaksi_by_id(999))).dispatch().await;
        assert_eq!(response.status(), Status::NotFound);
    }

    #[async_test]
    async fn test_update_transaksi_with_wrong_id() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi = Transaksi::new(1, "Test".to_string(), 100000.0, None);
        let response = client.post(uri!(super::create_transaksi)).json(&new_transaksi).dispatch().await;
        let mut created = response.into_json::<Transaksi>().await.unwrap();
        
        created.id = 999; // Wrong ID

        let response = client.patch(uri!(super::update_transaksi(created.id - 999)))
            .json(&created)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[async_test]
    async fn test_delete_detail_transaksi() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provide a valid Rocket instance");

        let new_transaksi = Transaksi::new(1, "Test User".to_string(), 0.0, None);
        let response = client.post(uri!(super::create_transaksi)).json(&new_transaksi).dispatch().await;
        let created_transaksi = response.into_json::<Transaksi>().await.unwrap();

        let detail = DetailTransaksi::new(created_transaksi.id, 101, 50000.0, 2);
        let response = client.post(uri!(super::add_detail_transaksi(created_transaksi.id)))
            .json(&detail)
            .dispatch()
            .await;
        let created_detail = response.into_json::<DetailTransaksi>().await.unwrap();

        let response = client.delete(uri!(super::delete_detail_transaksi(created_transaksi.id, created_detail.id)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::NoContent);

        let response = client.get(uri!(super::get_detail_transaksi(created_transaksi.id))).dispatch().await;
        let details = response.into_json::<Vec<DetailTransaksi>>().await.unwrap();
        assert_eq!(details.len(), 0);
    }
}
