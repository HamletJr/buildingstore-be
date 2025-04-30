use sqlx::{Any, pool::PoolConnection};
use crate::auth::model::user::User;
use crate::auth::model::session::Session;

pub struct AuthService;

impl AuthService {
    pub async fn register_user(mut db: PoolConnection<Any>, user: User) -> Result<User, sqlx::Error> {
        todo!()
    }

    pub async fn login_user(mut db: PoolConnection<Any>, user: User) -> Result<Session, sqlx::Error> {
        todo!()
    }

    pub async fn logout_user(mut db: PoolConnection<Any>, user_id: i64) -> Result<(), sqlx::Error> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::async_test;
    use sqlx::any::install_default_drivers;
    use sqlx::Pool;
    use crate::auth::model::user::User;

    async fn setup() -> Pool<Any> {
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

        db
    }

    #[async_test]
    async fn test_register_user() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let result = AuthService::register_user(db.acquire().await.unwrap(), user.clone()).await;
        assert!(result.is_ok());

        let registered_user = result.unwrap();
        assert_eq!(registered_user.username, user.username);
        assert_eq!(registered_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_login_valid_user_credentials() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        AuthService::register_user(db.acquire().await.unwrap(), user.clone()).await.unwrap();
        let result = AuthService::login_user(db.acquire().await.unwrap(), user.clone()).await;
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.user_id, user.id);
    }

    #[async_test]
    async fn test_login_invalid_username() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        AuthService::register_user(db.acquire().await.unwrap(), user.clone()).await.unwrap();
        let invalid_user = User::new("invalid_user".to_string(), "wrong_password".to_string(), false);

        let result = AuthService::login_user(db.acquire().await.unwrap(), invalid_user.clone()).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_login_invalid_password() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        AuthService::register_user(db.acquire().await.unwrap(), user.clone()).await.unwrap();
        let invalid_user = User::new("test_user".to_string(), "wrong_password".to_string(), false);

        let result = AuthService::login_user(db.acquire().await.unwrap(), invalid_user.clone()).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_login_invalid_user_and_password() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let result = AuthService::login_user(db.acquire().await.unwrap(), user.clone()).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_logout_user() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        AuthService::register_user(db.acquire().await.unwrap(), user.clone()).await.unwrap();
        AuthService::login_user(db.acquire().await.unwrap(), user.clone()).await.unwrap();
        let result = AuthService::logout_user(db.acquire().await.unwrap(), user.id).await;
        assert!(result.is_ok());
    }
}