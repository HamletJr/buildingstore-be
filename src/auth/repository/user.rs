use rocket_db_pools::sqlx;
use sqlx::Any;
use sqlx::pool::PoolConnection;
use crate::auth::model::user::User;

pub struct UserRepository;

impl UserRepository {
    pub async fn create_user(mut db: PoolConnection<Any>, user: User) -> Result<User, sqlx::Error> {
        todo!()
    }

    pub async fn get_user_by_username(mut db: PoolConnection<Any>, username: &str) -> Result<User, sqlx::Error> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::async_test;
    use sqlx::any::install_default_drivers;
    use sqlx::Pool;

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
    async fn test_create_user() {
        let db = setup().await;
        let user = User {
            id: 0,
            username: "test_user".to_string(),
            password: "password".to_string(),
            is_admin: false,
        };

        let result = UserRepository::create_user(db.acquire().await.unwrap(), user.clone()).await;
        assert!(result.is_ok());
        let created_user = result.unwrap();
        assert_eq!(created_user.username, user.username);
        assert_eq!(created_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_get_user_by_username() {
        let db = setup().await;
        let user = User {
            id: 0,
            username: "test_user".to_string(),
            password: "password".to_string(),
            is_admin: false,
        };

        UserRepository::create_user(db.acquire().await.unwrap(), user.clone()).await.unwrap();
        let result = UserRepository::get_user_by_username(db.acquire().await.unwrap(), &user.username).await;
        assert!(result.is_ok());
        let fetched_user = result.unwrap();
        assert_eq!(fetched_user.username, user.username);
        assert_eq!(fetched_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_get_user_by_nonexistent_username() {
        let db = setup().await;
        let result = UserRepository::get_user_by_username(db.acquire().await.unwrap(), "nonexistent_user").await;
        assert!(result.is_err());
    }
}