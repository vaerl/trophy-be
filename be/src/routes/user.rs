use actix_web::{
    cookie::Cookie, delete, get, post, put, web, HttpResponse, Responder, ResponseError,
};
use sqlx::PgPool;

use crate::model::{CreateLogin, CreateUser, CustomError, User, UserRole, UserToken};

#[get("/users")]
async fn find_all_users(
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, CustomError> {
    token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    Ok(HttpResponse::Ok().json(User::find_all(db_pool.get_ref()).await?))
}

#[get("/users/{id}")]
async fn find_user(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> Result<User, CustomError> {
    token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;

    User::find(id.into_inner(), db_pool.get_ref()).await
}

#[post("/users")]
async fn create_user(
    create_user: web::Json<CreateUser>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> Result<User, CustomError> {
    token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    User::create(create_user.into_inner(), db_pool.get_ref()).await
}

#[put("/users/{id}")]
async fn update_user(
    id: web::Path<i32>,
    user: web::Json<CreateUser>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> Result<User, CustomError> {
    token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    User::update(id.into_inner(), user.into_inner(), db_pool.get_ref()).await
}

#[delete("/users/{id}")]
async fn delete_user(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> Result<User, CustomError> {
    token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    User::delete(id.into_inner(), db_pool.get_ref()).await
}

#[post("/login")]
async fn login(login: web::Json<CreateLogin>, db_pool: web::Data<PgPool>) -> impl Responder {
    match User::login(login.into_inner(), db_pool.get_ref()).await {
        Ok(token_string) => {
            let cookie = Cookie::build("session", token_string)
                .domain("/")
                .secure(true)
                .http_only(true)
                .finish();
            HttpResponse::Ok().cookie(cookie).finish()
        }
        Err(err) => err.error_response(),
    }
}

#[post("/logout")]
async fn logout(token: UserToken, db_pool: web::Data<PgPool>) -> Result<HttpResponse, CustomError> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    match User::logout(user.id, db_pool.get_ref()).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Err(err),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_users);
    cfg.service(find_user);
    cfg.service(create_user);
    cfg.service(update_user);
    cfg.service(delete_user);
    cfg.service(login);
    cfg.service(logout);
}
