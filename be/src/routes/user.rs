use actix_web::{
    cookie::Cookie, delete, get, post, put, web, HttpRequest, HttpResponse, Responder,
    ResponseError,
};
use sqlx::PgPool;

use crate::model::{CreateLogin, CreateUser, User};

#[get("/users")]
async fn find_all_users(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find all users.");
    let result = User::find_all(db_pool.get_ref()).await;
    match result {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(err) => err.error_response(),
    }
}

#[get("/users/{id}")]
async fn find_user(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find user.");
    User::find(id.into_inner(), db_pool.get_ref()).await
}

#[post("/users")]
async fn create_user(
    create_user: web::Json<CreateUser>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Received new request: create user.");
    User::create(create_user.into_inner(), db_pool.get_ref()).await
}

#[put("/users/{id}")]
async fn update_user(
    id: web::Path<i32>,
    user: web::Json<CreateUser>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Received new request: update user.");
    User::update(id.into_inner(), user.into_inner(), db_pool.get_ref()).await
}

#[delete("/users/{id}")]
async fn delete_user(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: delete user.");
    User::delete(id.into_inner(), db_pool.get_ref()).await
}

#[post("/login")]
async fn login(login: web::Json<CreateLogin>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: login.");
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
async fn logout(request: HttpRequest, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: logout.");
    // TODO extract this somehow? -> services
    match User::from_request(&request, db_pool.get_ref()).await {
        Ok(user) => match User::logout(user.id, db_pool.get_ref()).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(err) => err.error_response(),
        },
        Err(err) => err.error_response(),
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
