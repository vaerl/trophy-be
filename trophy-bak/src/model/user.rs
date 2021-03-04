use actix_web::HttpRequest;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

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
// - improve error-handling: custom errors, check if they convert automatically when returning an error, use custom errors CURRENT
// - supply User-endpoints
// - write user.http
// - tests
// - update /reset/database

impl User {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            r#"SELECT id, username, password, role as "role: UserRole", session FROM users ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    pub async fn find(id: i32, pool: &PgPool) -> Result<User> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, username, password, role as "role: UserRole", session FROM users WHERE id = $1"#, id
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_name(name: &String, pool: &PgPool) -> Result<User> {
        let users = User::find_all(pool).await?;
        for user in users {
            if user.username.eq(name) {
                return Ok(user);
            }
        }
        Err(anyhow!("User {} does not exist!", name))
    }

    pub async fn create(create_user: CreateUser, pool: &PgPool) -> Result<User> {
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
            Err(anyhow!(
                "User {} is already registered!",
                create_user.username
            ))
        }
    }

    pub async fn update(id: i32, altered_user: CreateUser, pool: &PgPool) -> Result<User> {
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

    pub async fn delete(id: i32, pool: &PgPool) -> Result<User> {
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

    pub async fn from_request(request: &HttpRequest, pool: &PgPool) -> Result<User> {
        // let authen_header = match request.headers().get("Authorization") {
        //     Some(authen_header) => authen_header,
        //     None => {
        //         return Err();
        //     }
        // };
        // let authen_str = authen_header.to_str()?;
        // if !authen_str.starts_with("bearer") && !authen_str.starts_with("Bearer") {
        //     return Err();
        // }
        // let raw_token = authen_str[6..authen_str.len()].trim();
        // let token = UserToken::decode_token(raw_token.to_string())?;
        // let uid = token.uid;
        // Ok(uid)
        todo!()
    }
}
