use entity::users::Entity as Users;
use entity::users::Model as UserModel;
use entity::users::ActiveModel as UserActiveModel;
use sea_orm::{DbErr, DatabaseConnection, EntityTrait, ActiveModelTrait, QueryFilter, ColumnTrait};
use sea_orm::ActiveValue::{Set, NotSet};
use crate::auth::model::user::User;

pub struct UserRepository;

impl UserRepository {
    pub async fn create_user(db: &DatabaseConnection, user: User) -> Result<User, DbErr> {
        let user = UserActiveModel {
            id: NotSet,
            username: Set(user.username),
            password: Set(user.password),
            is_admin: Set(user.is_admin),
        };

        let user: UserModel  = user.insert(db).await?;
        Ok(User {
            id: user.id,
            username: user.username,
            password: user.password,
            is_admin: user.is_admin,
        })
    }

    pub async fn get_user_by_username(db: &DatabaseConnection, username: &str) -> Result<User, DbErr> {
        let user = Users::find()
            .filter(entity::users::Column::Username.eq(username))
            .one(db)
            .await?;
            
        match user {
            Some(user) => Ok(User {
                id: user.id,
                username: user.username,
                password: user.password,
                is_admin: user.is_admin,
            }),
            None => Err(DbErr::Custom("User not found".to_string())),
        }
    }

    pub async fn get_user_by_id(db: &DatabaseConnection, user_id: i32) -> Result<User, DbErr> {
        let user = Users::find_by_id(user_id)
            .one(db)
            .await?;

        match user {
            Some(user) => Ok(User {
                id: user.id,
                username: user.username,
                password: user.password,
                is_admin: user.is_admin,
            }),
            None => Err(DbErr::Custom("User not found".to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::async_test;
    use sea_orm::{Database, DatabaseConnection};
    use migration::{Migrator, MigratorTrait};

    async fn setup() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();

        Migrator::up(&db, None).await.unwrap();

        db
    }

    #[async_test]
    async fn test_create_user() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let result = UserRepository::create_user(&db, user.clone()).await;
        assert!(result.is_ok());
        let created_user = result.unwrap();
        assert_eq!(created_user.username, user.username);
        assert_eq!(created_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_get_user_by_username() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        UserRepository::create_user(&db, user.clone()).await.unwrap();
        let result = UserRepository::get_user_by_username(&db, &user.username).await;
        assert!(result.is_ok());
        let fetched_user = result.unwrap();
        assert_eq!(fetched_user.username, user.username);
        assert_eq!(fetched_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_get_user_by_nonexistent_username() {
        let db = setup().await;
        let result = UserRepository::get_user_by_username(&db, "nonexistent_user").await;
        assert!(result.is_err());
    }

    #[async_test]
    async fn test_get_user_by_id() {
        let db = setup().await;
        let user = User::new("test_user".to_string(), "password".to_string(), false);

        let created_user = UserRepository::create_user(&db, user.clone()).await.unwrap();
        let result = UserRepository::get_user_by_id(&db, created_user.id).await;
        assert!(result.is_ok());
        let fetched_user = result.unwrap();
        assert_eq!(fetched_user.username, user.username);
        assert_eq!(fetched_user.is_admin, user.is_admin);
    }

    #[async_test]
    async fn test_get_user_by_nonexistent_id() {
        let db = setup().await;
        let result = UserRepository::get_user_by_id(&db, 999).await;
        assert!(result.is_err());
    }
}