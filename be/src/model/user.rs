use std::fmt::{self, Display};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{ApiResult, model::Game};

use super::{CreateToken, CustomError, TypeInfo, UserToken};

#[derive(Serialize, Deserialize, sqlx::Type, PartialEq, Debug)]
#[sqlx(type_name = "user_role")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Referee,
    Visualizer,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self{
            UserRole::Admin => write!(f, "Admin"),
            UserRole::Referee => write!(f, "Referee"),
            UserRole::Visualizer => write!(f, "Visualizer"),
        }
    }
}

#[derive(Serialize, FromRow, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub role: UserRole,
    pub session: Option<String>,
    pub game_id: Option<i32>,
    pub game_name: Option<String>
}

#[derive(Serialize)]
// this syntax is brilliant!
pub struct UserVec(Vec<User>);

#[derive(Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub password: String,
    pub role: UserRole,
    pub game_id: Option<i32>
}

#[derive(Deserialize)]
pub struct UpdateUser {
    pub name: String,
    pub role: UserRole,
    pub password: Option<String>,
    pub game_id: Option<i32>
}

impl fmt::Display for CreateUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CreateUser(name: {}, password: {}, role: {})", self.name, self.password, self.role)
    }
}

#[derive(Deserialize)]
pub struct CreateLogin {
    pub name: String,
    pub password: String
}


/// NOTE `LEFT JOIN`s are correct here - we don't know if users.game_id is null or not.
/// By using `field as "field?"` we make sqlx clear that it's potentially nullable - this solved an issue where sqlx would complain even though
/// the receiving field was accepting an `Option` - more [here](https://github.com/launchbadge/sqlx/issues/1852).
impl User {
    pub async fn find_all(pool: &PgPool) -> ApiResult<UserVec> {
        let users = sqlx::query_as!(
            User,
            r#"SELECT users.id, users.name, password, role as "role: UserRole", game_id, games.name as "game_name?", session FROM users
            LEFT JOIN games ON games.id=users.game_id
            ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(UserVec(users))
    }

    pub async fn find(id: i32, pool: &PgPool) -> ApiResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT users.id, users.name, password, role as "role: UserRole", game_id, games.name as "game_name?", session FROM users
            LEFT JOIN games ON games.id=users.game_id
            WHERE users.id = $1"#, id
        )
        .fetch_optional(pool)
        .await?;

        user.ok_or(CustomError::NotFoundError { message: format!("User {} could not be found.", id) })
    }

    pub async fn find_by_name(name: &String, pool: &PgPool) -> ApiResult<User> {
        info!("{}", name);
        let user = sqlx::query_as!(
            User,
            r#"SELECT users.id, users.name, users.password, users.role as "role: UserRole", game_id, games.name as "game_name?", users.session FROM users
            LEFT JOIN games ON users.game_id=games.id
            WHERE users.name = $1"#, name
        ).fetch_optional(pool)
        .await?;

        user.ok_or(CustomError::NotFoundError { message: format!("User {} could not be found.", name) })
    }

    pub async fn find_game_for_ref(user_id: i32, pool: &PgPool) -> ApiResult<Game> {
        let user = User::find(user_id, pool).await?;

        match user.game_id {
            Some(game_id) => Game::find(game_id, pool).await,
            None => Err(CustomError::NotFoundError {message: format!("Game for user {} could not be found!", user_id)}),
        }
    }

    pub async fn create(create_user: CreateUser, pool: &PgPool) -> ApiResult<User> {
        if User::find_by_name(&create_user.name, pool)
            .await
            .is_err()
        {
            info!("Creating new user.");
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2.hash_password(&create_user.password.as_bytes(), &salt).unwrap().to_string();

            let mut tx = pool.begin().await?;
            let user = sqlx::query_as!( User, 
                r#"WITH inserted AS (INSERT INTO users (name, password, role, game_id) VALUES ($1, $2, $3, $4) RETURNING id, name, password, role as "role: UserRole", game_id, session)
                SELECT inserted.id, inserted.name, password, "role: UserRole", game_id, games.name as "game_name?", session FROM inserted
                    LEFT JOIN games ON games.id=inserted.game_id"#,
                create_user.name, password_hash, create_user.role as UserRole, create_user.game_id
            )
            .fetch_one(&mut *tx)
            .await?;
            tx.commit().await?;

            Ok(user)
        } else {
            Err(CustomError::AlreadyExistsError {message: format!("User {} already exists!", create_user.name)})
        }
    }

    /// Passing a new password updates the password.
    pub async fn update(id: i32, altered_user: UpdateUser, pool: &PgPool) -> ApiResult<User> {
        match altered_user.password {
            Some(password) => {
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                let password_hash = argon2.hash_password(&password.as_bytes(), &salt).unwrap().to_string();
        
                
                let mut tx = pool.begin().await?;
                let user = sqlx::query_as!(
                    User, 
                    r#"WITH updated AS (UPDATE users SET name = $1, password = $2, role = $3, game_id = $4 WHERE id = $5 RETURNING id, name, password, role as "role: UserRole", game_id, session)
                    SELECT updated.id, updated.name, password, "role: UserRole", game_id, games.name as "game_name?", session FROM updated
                            LEFT JOIN games ON games.id=updated.game_id"#,
                    altered_user.name, password_hash, altered_user.role as UserRole, altered_user.game_id, id
                )
                .fetch_one(&mut *tx)
                .await?;
        
                tx.commit().await?;
                Ok(user)
            },
            None => {
                let mut tx = pool.begin().await?;
                let user = sqlx::query_as!(
                    User, 
                    r#"WITH updated AS (UPDATE users SET name = $1, role = $2, game_id = $3 WHERE id = $4 RETURNING id, name, password, role as "role: UserRole", game_id, session)
                    SELECT updated.id, updated.name, password, "role: UserRole", game_id, games.name as "game_name?", session FROM updated
                            LEFT JOIN games ON games.id=updated.game_id"#,
                    altered_user.name, altered_user.role as UserRole, altered_user.game_id, id
                )
                .fetch_one(&mut *tx)
                .await?;
                tx.commit().await?;
                Ok(user)
            }
        }
    }

    async fn update_session(id: i32, session: &String, pool: &PgPool) -> ApiResult<()> {
        let mut tx = pool.begin().await?;
        sqlx::query_as!(
            User, 
            r#"UPDATE users SET session = $1 WHERE id = $2"#,
            session, id
        ).execute(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn delete(id: i32, pool: &PgPool) -> ApiResult<User> {
        let mut tx = pool.begin().await?;
        let user = sqlx::query_as!(
            User,
            r#"WITH deleted AS (DELETE FROM users WHERE id = $1 RETURNING id, name, password, role as "role: UserRole", game_id, session)
            SELECT deleted.id, deleted.name, password, "role: UserRole", game_id, games.name as "game_name?", session FROM deleted
                    LEFT JOIN games ON games.id=deleted.game_id"#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    pub async fn login(login: CreateLogin, pool: &PgPool) -> ApiResult<String> {
        let user = User::find_by_name(&login.name, pool).await?;
        let argon2 = Argon2::default();
        let password_hash = PasswordHash::new(&user.password)?;
        
        if user.password.is_empty() || argon2.verify_password(login.password.as_bytes(), &password_hash).is_err() {
            return Err(CustomError::BadPasswordError {message: "Token is invalid!".to_string()});
        } else {
            let session = User::generate_session();
            User::update_session(user.id, &session, &pool).await?;
            return Ok(UserToken::generate_token(&CreateToken {user_id: user.id, session}, user));
        }
    }

    pub async fn logout(id: i32, pool: &PgPool) -> ApiResult<()> {
        User::update_session(id, &"".to_string(), pool).await
    }

    pub fn generate_session() -> String {
        Uuid::new_v4().as_simple().to_string()
    }
}

impl Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Game(id: {}, name: {}, role: {}, game_id: {:#?})",self.id, self.name, self.role, self.game_id)
    }
}

impl TypeInfo for User {
    fn type_name(&self) -> String {
       format!("User")
    }
}

impl Display for UserVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UserVec[{}]", self.0.iter().map(|g| g.to_string()).collect::<String>())
    }
}

impl TypeInfo for UserVec {
    fn type_name(&self) -> String {
       format!("UserVec")
    }
}
