use std::fmt::{self, Display};
use actix_web::{HttpRequest, HttpResponse, Responder, body::Body, http::header::ContentType};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{ApiResult, derive_responder::Responder, model::{Game, LogUserAction}};

use super::{CreateToken, CustomError, TypeInfo, UserToken};

#[derive(Serialize, Deserialize, sqlx::Type, PartialEq)]
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

#[derive(Serialize, FromRow, Responder)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: UserRole,
    // TODO consider making this a vec to support multiple sessions per user!
    // -> sessions would need to match to something for this!
    pub session: String,
    pub game_id: Option<i32>
}

#[derive(Serialize, Responder)]
// this syntax is brilliant!
pub struct UserVec(Vec<User>);

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub role: UserRole,
    pub game_id: Option<i32>
}

impl fmt::Display for CreateUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CreateUser(username: {}, password: {}, role: {})", self.username, self.password, self.role)
    }
}

#[derive(Deserialize)]
pub struct CreateLogin {
    pub username: String,
    pub password: String
}

impl User {
    pub async fn find_all(pool: &PgPool) -> ApiResult<UserVec> {
        let users = sqlx::query_as!(
            User,
            r#"SELECT id, username, password, role as "role: UserRole", game_id, session FROM users ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(UserVec(users))
    }

    pub async fn find(id: i32, pool: &PgPool) -> ApiResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, username, password, role as "role: UserRole", game_id, session FROM users WHERE id = $1"#, id
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    async fn find_by_name(name: &String, pool: &PgPool) -> ApiResult<User> {
        let users = User::find_all(pool).await?.0;
        for user in users {
            if user.username.eq(name) {
                return Ok(user);
            }
        }
        Err(CustomError::NotFoundError {message: format!("User {} does not exist!", name)})
    }

    pub async fn find_game_for_ref(user_id: i32, pool: &PgPool) -> ApiResult<Game> {
        let user = User::find(user_id, pool).await?;

        match user.game_id {
            Some(game_id) => Game::find(game_id, pool).await,
            None => Err(CustomError::NotFoundError {message: format!("Game for user {} could not be found!", user_id)}),
        }
    }

    pub async fn create(create_user: CreateUser, pool: &PgPool) -> ApiResult<User> {
        if User::find_by_name(&create_user.username, pool)
            .await
            .is_err()
        {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2.hash_password(&create_user.password.as_bytes(), salt.as_ref()).unwrap().to_string();

            let mut tx = pool.begin().await?;
            let user = sqlx::query_as!( User, 
                r#"INSERT INTO users (username, password, role, game_id) VALUES ($1, $2, $3, $4) RETURNING id, username, password, role as "role: UserRole", game_id, session"#,
                create_user.username, password_hash,  create_user.role as UserRole, create_user.game_id
            )
            .fetch_one(&mut tx)
            .await?;
            tx.commit().await?;

            Ok(user)
        } else {
            Err(CustomError::AlreadyExistsError {message: format!("User {} already exists!", create_user.username)})
        }
    }

    pub async fn update(id: i32, altered_user: CreateUser, pool: &PgPool) -> ApiResult<User> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(&altered_user.password.as_bytes(), salt.as_ref()).unwrap().to_string();

        
        let mut tx = pool.begin().await?;
        let user = sqlx::query_as!(
            User, 
            r#"UPDATE users SET username = $1, password = $2, role = $3, game_id = $4 WHERE id = $5 RETURNING id, username, password, role as "role: UserRole", game_id, session"#,
            altered_user.username, password_hash, altered_user.role as UserRole, altered_user.game_id, id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    async fn update_session(id: i32, session: &String, pool: &PgPool) -> ApiResult<()> {
        let mut tx = pool.begin().await?;
        sqlx::query_as!(
            User, 
            r#"UPDATE users SET session = $1 WHERE id = $2"#,
            session, id
        ).execute(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn delete(id: i32, pool: &PgPool) -> ApiResult<User> {
        let mut tx = pool.begin().await?;
        let user = sqlx::query_as!(
            User,
            r#"DELETE FROM users WHERE id = $1 RETURNING id, username, password, role as "role: UserRole", game_id, session"#,
            id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    pub async fn login(login: CreateLogin, pool: &PgPool) -> ApiResult<String> {
        let mut user = User::find_by_name(&login.username, pool).await?;
        let argon2 = Argon2::default();
        // TODO what happens when the user is already logged in?
        // -> I currently overwrite(?). I should either support multiple sessions or just return the existing session!
        
        let password_hash = PasswordHash::new(&user.password)?;
        
        if user.password.is_empty() || argon2.verify_password(login.password.as_bytes(), &password_hash).is_err() {
            return Err(CustomError::BadPasswordError {message: "Token is invalid!".to_string()});
        } else {
            let session = User::generate_session();
            User::update_session(user.id, &session, &pool).await?;

            user = user.log_action(format!("log-in"), pool).await?;

            return Ok(UserToken::generate_token(&CreateToken {user_id: user.id, session}, user));
        }
    }

    pub async fn logout(id: i32, pool: &PgPool) -> ApiResult<()> {
        User::update_session(id, &"".to_string(), pool).await
    }

    pub fn generate_session() -> String {
        Uuid::new_v4().to_simple().to_string()
    }
}

impl Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Game(id: {}, username: {}, role: {}, game_id: {:#?})",self.id, self.username, self.role, self.game_id)
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
