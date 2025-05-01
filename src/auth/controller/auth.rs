use rocket::{get, post, State};
use rocket::form::{Form, FromForm};
use rocket::http::CookieJar;
use sqlx::{Any, Pool};

#[derive(FromForm)]
pub struct AuthForm {
    pub username: String,
    pub password: String,
}

#[post("/login", data = "<form>")]
pub async fn login(form: Form<AuthForm>, cookies: &CookieJar<'_>, mut db: &State<Pool<Any>>) -> String {
    todo!()
}

#[post("/register", data = "<form>")]
pub async fn register(form: Form<AuthForm>, mut db: &State<Pool<Any>>) -> String {
    todo!()
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> String {
    todo!()
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::http::Status;
    use rocket::{routes, uri, Rocket, async_test};
    use sqlx::any::install_default_drivers;

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
            .manage(reqwest::Client::builder().build().unwrap())
            .manage(db.clone())
            .mount("/", routes![login, register, logout]);

        rocket
    }

    #[async_test]
    async fn test_register() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        let response = client.post(uri!(super::register))
            .header(rocket::http::ContentType::Form)
            .body("username=testuser&password=testpass")
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
    }

    #[async_test]
    async fn test_login_valid_credentials() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        client.post(uri!(super::register))
            .header(rocket::http::ContentType::Form)
            .body("username=testuser&password=testpass")
            .dispatch()
            .await;
        let response = client.post(uri!(super::login))
            .header(rocket::http::ContentType::Form)
            .body("username=testuser&password=testpass")
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let cookies = response.cookies();
        assert!(cookies.get("session_key").is_some(), "Session cookie should be set");
    }

    #[async_test]
    async fn test_login_invalid_credentials() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        client.post(uri!(super::register))
            .header(rocket::http::ContentType::Form)
            .body("username=testuser&password=testpass")
            .dispatch()
            .await;
        let response = client.post(uri!(super::login))
            .header(rocket::http::ContentType::Form)
            .body("username=invaliduser&password=invalidpass")
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Unauthorized);
        let cookies = response.cookies();
        assert!(cookies.get("session_key").is_none(), "Session cookie should not be set");
    }

    #[async_test]
    async fn test_logout() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        client.post(uri!(super::register))
            .header(rocket::http::ContentType::Form)
            .body("username=testuser&password=testpass")
            .dispatch()
            .await;
        client.post(uri!(super::login))
            .header(rocket::http::ContentType::Form)
            .body("username=testuser&password=testpass")
            .dispatch()
            .await;
        let response = client.get(uri!(super::logout))
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);
        let cookies = response.cookies();
        assert!(cookies.get("session_key").is_none(), "Session cookie should be cleared");
    }
}