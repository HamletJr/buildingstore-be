use uuid::Uuid;
use sea_orm::{DatabaseConnection, DbErr};

use crate::auth::model::user::User;
use crate::auth::model::session::Session;
use crate::auth::repository::user::UserRepository;
use crate::auth::repository::session::SessionRepository;

pub struct AuthService;

impl AuthService {
    pub async fn register_user(db: &DatabaseConnection, user: User) -> Result<User, DbErr> {
        let existing_user = UserRepository::get_user_by_username(&db, &user.username).await;
        if existing_user.is_ok() {
            return Err(DbErr::Custom("Username already exists".to_string()));
        }
        let new_user = UserRepository::create_user(&db, user).await?;
        Ok(new_user)
    }

    pub async fn login_user(db: &DatabaseConnection, username: String, password: String) -> Result<Session, DbErr> {
        let existing_user = UserRepository::get_user_by_username(&db, &username).await;
        if existing_user.is_err() {
            return Err(DbErr::Custom("User not found".to_string()));
        }
        let existing_user = existing_user.unwrap();
        let is_password_valid = existing_user.verify_password(&password);
        if !is_password_valid {
            return Err(DbErr::Custom("Invalid password".to_string()));
        }
        let session = Session::new(existing_user.clone());
        SessionRepository::create_session(&db, session.clone()).await?;
        Ok(session)
    }

    pub async fn logout_user(db: &DatabaseConnection, session_key: Uuid) -> Result<(), DbErr> {
        SessionRepository::delete_session(&db, session_key).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::async_test;
    use sea_orm::{Database, DatabaseConnection};
    use migration::{Migrator, MigratorTrait};
    use crate::auth::model::user::User;

    async fn setup() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();

        db
    }

    #[async_test]
    async fn test_register_user() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let result = AuthService::register_user(&db, user.clone()).await;
        assert!(result.is_ok());

        let registered_user = result.unwrap();
        assert_eq!(registered_user.username, user.username);
        assert_eq!(registered_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_login_valid_user_credentials() {
        let db = setup().await;
        let username = "test_user".to_string();
        let password = "password".to_string();
        let mut user = User::new(username.clone(), password.clone(), false);

        user = AuthService::register_user(&db, user.clone()).await.unwrap();
        let result = AuthService::login_user(&db, username.clone(), password.clone()).await;
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.user_id, user.id);
    }

    #[async_test]
    async fn test_login_invalid_username() {
        let db = setup().await;
        let username = "test_user".to_string();
        let password = "password".to_string();
        let user = User::new(username.clone(), password.clone(), false);

        AuthService::register_user(&db, user.clone()).await.unwrap();

        let result = AuthService::login_user(&db, "dummy".to_string(), password.clone()).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_login_invalid_password() {
        let db = setup().await;
        let username = "test_user".to_string();
        let password = "password".to_string();
        let user = User::new(username.clone(), password.clone(), false);

        AuthService::register_user(&db, user.clone()).await.unwrap();

        let result = AuthService::login_user(&db, username.clone(), "dummypass".to_string()).await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_logout_user() {
        let db = setup().await;
        let username = "test_user".to_string();
        let password = "password".to_string();
        let user = User::new(username.clone(), password.clone(), false);

        AuthService::register_user(&db, user.clone()).await.unwrap();
        let session = AuthService::login_user(&db, username.clone(), password.clone()).await.unwrap();
        let result = AuthService::logout_user(&db, Uuid::try_parse(&session.session_key).unwrap()).await;
        assert!(result.is_ok());
    }
}