use actix_web::HttpRequest;
use anyhow::{Result};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use super::{AuthenticationError, CreateToken, DataBaseError, UserToken};

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename = "user_role")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Referee,
    Visualizer,
}

#[derive(Serialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: UserRole,
    pub session: String,
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub role: UserRole
}

pub struct CreateLogin {
    pub username: String,
    pub password: String
}


// TODO
// - implement User: auth CHECK
// - initially create admin-user
// - use thiserror for errors -> should provide default-messages
// - supply User-endpoints
// - write user.http
// - start checks
//      - authentication -> pass method and allowed roles to service to check
//      - logs -> log transaction; from auth_service?
// - tests
// - update /reset/database

impl User {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<User>, DataBaseError> {
        let users = sqlx::query_as!(
            User,
            r#"SELECT id, username, password, role as "role: UserRole", session FROM users ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn find(id: i32, pool: &PgPool) -> Result<User, DataBaseError> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, username, password, role as "role: UserRole", session FROM users WHERE id = $1"#, id
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_name(name: &String, pool: &PgPool) -> Result<User, DataBaseError> {
        let users = User::find_all(pool).await?;
        for user in users {
            if user.username.eq(name) {
                return Ok(user);
            }
        }
        Err(DataBaseError::NotFoundError {message: format!("User {} does not exist!", name)})
    }

    pub async fn create(create_user: CreateUser, pool: &PgPool) -> Result<User, DataBaseError> {
        if User::find_by_name(&create_user.username, pool)
            .await
            .is_err()
        {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2.hash_password_simple(&create_user.password.as_bytes(), salt.as_ref()).unwrap().to_string();

            let mut tx = pool.begin().await?;
            let user = sqlx::query_as!( User, 
                r#"INSERT INTO users (username, password, role) VALUES ($1, $2, $3) RETURNING id, username, password, role as "role: UserRole", session"#,
                create_user.username, password_hash,  create_user.role as UserRole
            )
            .fetch_one(&mut tx)
            .await?;
            tx.commit().await?;

            Ok(user)
        } else {
            Err(DataBaseError::AlreadyExistsError {message: format!("User {} already exists!", create_user.username)})
        }
    }

    pub async fn update(id: i32, altered_user: CreateUser, pool: &PgPool) -> Result<User, DataBaseError> {
        let mut tx = pool.begin().await?;
        let user = sqlx::query_as!(
            User, 
            r#"UPDATE users SET username = $1, password = $2, role = $3 WHERE id = $4 RETURNING id, username, password, role as "role: UserRole", session"#,
            altered_user.username, altered_user.password, altered_user.role as UserRole, id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    pub async fn update_session(id: i32, session: &String, pool: &PgPool) -> Result<(), DataBaseError> {
        let mut tx = pool.begin().await?;
        let user = sqlx::query_as!(
            User, 
            r#"UPDATE users SET session = $1 WHERE id = $2"#,
            session, id
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(())
    }

    pub async fn delete(id: i32, pool: &PgPool) -> Result<User, DataBaseError> {
        let mut tx = pool.begin().await?;
        let user = sqlx::query_as!(
            User,
            r#"DELETE FROM users WHERE id = $1 RETURNING id, username, password, role as "role: UserRole", session"#,
            id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    pub async fn login(login: CreateLogin, pool: &PgPool) -> Result<String, AuthenticationError> {
        let user = User::find_by_name(&login.username, pool).await?;
        let argon2 = Argon2::default();

        let password_hash = PasswordHash::new(&user.password)?;

        if !user.password.is_empty() || argon2.verify_password(login.password.as_bytes(), &password_hash).is_err() {
            return Err(AuthenticationError::BadPasswordError {message: "Token is invalid!".to_string()});
        } else {
            let session = User::generate_session();
            User::update_session(user.id, &session, &pool).await?;
            return Ok(UserToken::generate_token(&CreateToken {user_id: user.id, session}, user));
        }
    }

    pub async fn logout(id: i32, pool: &PgPool) -> Result<(), DataBaseError> {
        User::update_session(id, &"".to_string(), pool).await
    }

    pub fn generate_session() -> String {
        Uuid::new_v4().to_simple().to_string()
    }

    pub async fn from_request(request: &HttpRequest, pool: &PgPool) -> Result<User, AuthenticationError> {
        let authn_header = match request.headers().get("Authorization") {
            Some(authn_header) => authn_header,
            None => {
                // explicit return so that it's not assigned to authn_header
                return Err(AuthenticationError::NoTokenError {
                    message: "There was no authorization-header in the request!".to_string()
                });
            }
        };
        let authn_str = authn_header.to_str()?;
        if !authn_str.starts_with("bearer") && !authn_str.starts_with("Bearer") {
            return Err(AuthenticationError::NoTokenError {
                message: "There was no bearer-header in the request!".to_string()
            });
        }
        let raw_token = authn_str[6..authn_str.len()].trim();
        let token = UserToken::decode_token(raw_token.to_string())?;
        if token.is_valid() {
            let user = Self::find(token.user_id, pool).await?;
            Ok(user)
        } else {
            Err(AuthenticationError::NoTokenError {message: "Token is invalid!".to_string()})
        }
    }
}
