use actix_web::{HttpRequest, HttpResponse, Responder};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use futures::future::{Ready, ready};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::ApiResult;

use super::{CustomError, CreateToken, LoginTransaction, UserToken};

#[derive(Serialize, Deserialize, sqlx::Type, PartialEq)]
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

// this syntax is brilliant!
#[derive(Serialize)]
pub struct UserVec(Vec<User>);

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub role: UserRole
}

#[derive(Deserialize)]
pub struct CreateLogin {
    pub username: String,
    pub password: String
}


// TODO
// - adjust all routes to:
//      1. check authorization!
//      2. use logging
//      3. use newtype-vecs
// - log between token and action with Transaction-History
// - log in try-into-user -> this needs a pool! 
// - check if anyhow is necessary
// - implement transaction-history -> use actix-web middleware-logger?
// -> I've almost settled on logging myself - to access the user, I have to log after getting the token!
// - write history-routes? -> I need this for showing logs in the admin-fe
//      - write history.http
// - merge branch
// - tests -> implement on separate branch
//      - this will include a lot of bugfixing!
// - update /reset/database--> what did I want to do here?

impl Responder for User {
    type Error = CustomError;
    type Future = Ready<ApiResult<HttpResponse>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}


impl Responder for UserVec {
    type Error = CustomError;
    type Future = Ready<ApiResult<HttpResponse>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl User {
    pub async fn find_all(pool: &PgPool) -> ApiResult<UserVec> {
        let users = sqlx::query_as!(
            User,
            r#"SELECT id, username, password, role as "role: UserRole", session FROM users ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(UserVec(users))
    }

    pub async fn find(id: i32, pool: &PgPool) -> ApiResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT id, username, password, role as "role: UserRole", session FROM users WHERE id = $1"#, id
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

    pub async fn create(create_user: CreateUser, pool: &PgPool) -> ApiResult<User> {
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
            Err(CustomError::AlreadyExistsError {message: format!("User {} already exists!", create_user.username)})
        }
    }

    pub async fn update(id: i32, altered_user: CreateUser, pool: &PgPool) -> ApiResult<User> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password_simple(&altered_user.password.as_bytes(), salt.as_ref()).unwrap().to_string();

        
        let mut tx = pool.begin().await?;
        let user = sqlx::query_as!(
            User, 
            r#"UPDATE users SET username = $1, password = $2, role = $3 WHERE id = $4 RETURNING id, username, password, role as "role: UserRole", session"#,
            altered_user.username, password_hash, altered_user.role as UserRole, id
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
            r#"DELETE FROM users WHERE id = $1 RETURNING id, username, password, role as "role: UserRole", session"#,
            id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    pub async fn login(login: CreateLogin, pool: &PgPool) -> ApiResult<String> {
        let user = User::find_by_name(&login.username, pool).await?;
        let argon2 = Argon2::default();

        let password_hash = PasswordHash::new(&user.password)?;

        if user.password.is_empty() || argon2.verify_password(login.password.as_bytes(), &password_hash).is_err() {
            return Err(CustomError::BadPasswordError {message: "Token is invalid!".to_string()});
        } else {
            let session = User::generate_session();
            User::update_session(user.id, &session, &pool).await?;
            LoginTransaction::create(user.id, &pool).await?;
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