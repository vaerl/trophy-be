use actix_web::HttpRequest;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use super::{AuthenticationError, DataBaseError};

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

pub struct CreateSession {}
pub struct SessionInfo {}

// TODO
// - implement User: auth, hash pw
// - supply User-endpoints
// - write user.http
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
        // TODO hash password
        if User::find_by_name(&create_user.username, pool)
            .await
            .is_err()
        {
            let mut tx = pool.begin().await?;
            let user = sqlx::query_as!( User, 
                r#"INSERT INTO users (username, password, role) VALUES ($1, $2, $3) RETURNING id, username, password, role as "role: UserRole", session"#,
                create_user.username, create_user.password,  create_user.role as UserRole
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

    pub fn login() {}
    pub fn logout() {}
    pub fn generate_session() {}
    pub fn is_valid_session() {}

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
        // let token = UserToken::decode_token(raw_token.to_string())?;
        // Ok(token.uid)
        todo!()
    }
}
