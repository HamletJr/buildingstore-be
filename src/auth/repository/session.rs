use entity::sessions::Entity as Sessions;
use entity::sessions::Model as SessionModel;
use entity::sessions::ActiveModel as SessionActiveModel;
use sea_orm::{DbErr, DatabaseConnection, EntityTrait, ActiveModelTrait};
use sea_orm::ActiveValue::Set;
use uuid::Uuid;
use crate::auth::model::session::Session;

pub struct SessionRepository;

impl SessionRepository {
    pub async fn create_session(db: &DatabaseConnection, session: Session) -> Result<Session, DbErr> {
        let session = SessionActiveModel {
            session_key: Set(Uuid::try_parse(&session.session_key).unwrap()),
            user_id: Set(session.user_id),
            expires_at: Set(session.expires_at),
        };

        let session: SessionModel = session.insert(db).await?;

        Ok(Session {
            session_key: session.session_key.to_string(),
            user_id: session.user_id,
            expires_at: session.expires_at,
        })
    }

    pub async fn get_session_by_key(db: &DatabaseConnection, session_key: Uuid) -> Result<Session, DbErr> {
        let session = Sessions::find_by_id(session_key)
            .one(db)
            .await?;

        match session {
            Some(session) => Ok(Session {
                session_key: session.session_key.to_string(),
                user_id: session.user_id,
                expires_at: session.expires_at,
            }),
            None => Err(DbErr::Custom("Session not found".to_string())),
        }
    }

    pub async fn delete_session(db: &DatabaseConnection, session_key: Uuid) -> Result<(), DbErr> {
        let res = Sessions::delete_by_id(session_key)
            .exec(db)
            .await?;

        match res.rows_affected {
            0 => Err(DbErr::Custom("Session not found".to_string())),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::async_test;
    use sea_orm::{Database, DatabaseConnection};
    use migration::{Migrator, MigratorTrait};
    use crate::auth::model::user::User;
    use crate::auth::repository::user::UserRepository;

    async fn setup() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        Migrator::up(&db, None).await.unwrap();

        db
    }

    #[async_test]
    async fn test_create_session() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);
        let result = UserRepository::create_user(&db, user.clone()).await.unwrap();
        
        let session = Session::new(result.clone());
        let result = SessionRepository::create_session(&db, session.clone()).await;
        assert!(result.is_ok());
        
        let fetched_session = result.unwrap();
        assert_eq!(fetched_session.session_key, session.session_key);
        assert_eq!(fetched_session.user_id, session.user_id);
        assert_eq!(fetched_session.expires_at, session.expires_at);
    }

    #[async_test]
    async fn test_get_session_by_key() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);
        let result = UserRepository::create_user(&db, user.clone()).await.unwrap();

        let session = Session::new(result.clone());
        SessionRepository::create_session(&db, session.clone()).await.unwrap();
        let retrieved_session = SessionRepository::get_session_by_key(&db, Uuid::parse_str(&session.session_key).unwrap()).await;
        assert!(retrieved_session.is_ok());
        
        let fetched_session = retrieved_session.unwrap();
        assert_eq!(fetched_session.session_key, session.session_key);
        assert_eq!(fetched_session.user_id, session.user_id);
        assert_eq!(fetched_session.expires_at, session.expires_at);
    }

    #[async_test]
    async fn test_get_session_by_nonexistent_key() {
        let db = setup().await;
        let result = UserRepository::get_user_by_username(&db, "nonexistent_user").await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_delete_session() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);
        let result = UserRepository::create_user(&db, user.clone()).await.unwrap();

        let session = Session::new(result.clone());
        SessionRepository::create_session(&db, session.clone()).await.unwrap();
        let result = SessionRepository::delete_session(&db, Uuid::parse_str(&session.session_key).unwrap()).await;
        assert!(result.is_ok());
        
        let retrieved_session = SessionRepository::get_session_by_key(&db, Uuid::parse_str(&session.session_key).unwrap()).await;
        assert!(retrieved_session.is_err());
    }
}