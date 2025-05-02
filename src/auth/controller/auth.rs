use rocket::{get, post, State};
use rocket::form::{Form, FromForm};
use rocket::http::{Status, Cookie, CookieJar};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::auth::model::user::User;
use crate::auth::service::auth::AuthService;

#[derive(FromForm)]
pub struct AuthForm {
    pub username: String,
    pub password: String,
}

#[post("/login", data = "<form>")]
pub async fn login(form: Form<AuthForm>, cookies: &CookieJar<'_>, db: &State<DatabaseConnection>) -> Status {
    let username = form.username.clone();
    let password = form.password.clone();

    let result = AuthService::login_user(&db, username, password).await;
    match result {
        Ok(session) => {
            cookies.add_private(Cookie::new("session_key", session.session_key));
            Status::Ok
        },
        Err(_) => Status::Unauthorized,
    }
}

#[post("/register", data = "<form>")]
pub async fn register(form: Form<AuthForm>, db: &State<DatabaseConnection>) -> Status {
    let username = form.username.clone();
    let password = form.password.clone();
    let is_admin = false; // Default to false for regular users

    let user = User::new(username, password, is_admin);
    let result = AuthService::register_user(&db, user).await;
    match result {
        Ok(_) => Status::Ok,
        Err(_) => Status::BadRequest
    }
}

#[get("/logout")]
pub async fn logout(db: &State<DatabaseConnection>, cookies: &CookieJar<'_>) -> Status {
    let session_key = cookies.get_private("session_key").map(|c| c.value().to_string()).unwrap_or_default();
    if session_key.is_empty() {
        return Status::BadRequest;
    }
    AuthService::logout_user(&db, Uuid::try_parse(&session_key).unwrap()).await.unwrap();
    cookies.remove_private(Cookie::build("session_key"));
    Status::Ok
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::local::asynchronous::Client;
    use rocket::http::Status;
    use rocket::{routes, uri, Rocket, async_test};
    use sea_orm::Database;
    use migration::{Migrator, MigratorTrait};
    

    async fn setup() -> Rocket<rocket::Build> {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();

        let rocket = rocket::build()
            .manage(reqwest::Client::builder().build().unwrap())
            .manage(db)
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
    async fn test_register_existing_user() {
        let rocket = setup().await;
        let client = Client::tracked(rocket).await.expect("Must provice a valid Rocket instance");
        client.post(uri!(super::register))
            .header(rocket::http::ContentType::Form)
            .body("username=testuser&password=testpass")
            .dispatch()
            .await;
        let response = client.post(uri!(super::register))
            .header(rocket::http::ContentType::Form)
            .body("username=testuser&password=testpass")
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::BadRequest);
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
        let session_key = cookies.get("session_key").map(|c| c.value()).unwrap_or_default();
        assert!(session_key.is_empty(), "Session cookie should be cleared");
    }
}