use actix_web::{
    cookie::Cookie, delete, get, post, put, web, HttpResponse, Responder, ResponseError,
};
use sqlx::PgPool;

use crate::{
    model::{CreateLogin, CreateUser, Game, History, User, UserRole, UserToken, UserVec},
    ApiResult,
};

#[get("/users")]
async fn find_all_users(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<UserVec> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::log(user.id, format!("find all users"), db_pool.get_ref()).await?;
    User::find_all(db_pool.get_ref()).await
}

#[get("/users/{id}")]
async fn find_user(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<User> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::log(user.id, format!("get user with ID"), db_pool.get_ref()).await?;
    User::find(id.into_inner(), db_pool.get_ref()).await
}

#[get("/users/{id}/game")]
async fn find_game_for_ref(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Game> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin, UserRole::Referee], db_pool.get_ref())
        .await?;
    History::log(user.id, format!("get user with ID"), db_pool.get_ref()).await?;
    User::find_game_for_ref(id.into_inner(), db_pool.get_ref()).await
}

#[post("/users")]
async fn create_user(
    create_user: web::Json<CreateUser>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<User> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::log(user.id, format!("create user"), db_pool.get_ref()).await?;
    User::create(create_user.into_inner(), db_pool.get_ref()).await
}

#[put("/users/{id}")]
async fn update_user(
    id: web::Path<i32>,
    altered_user: web::Json<CreateUser>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<User> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::log(user.id, format!("update user"), db_pool.get_ref()).await?;
    User::update(
        id.into_inner(),
        altered_user.into_inner(),
        db_pool.get_ref(),
    )
    .await
}

#[delete("/users/{id}")]
async fn delete_user(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<User> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::log(user.id, format!("delete user"), db_pool.get_ref()).await?;
    User::delete(id.into_inner(), db_pool.get_ref()).await
}

#[post("/login")]
async fn login(login: web::Json<CreateLogin>, db_pool: web::Data<PgPool>) -> impl Responder {
    // loggin is done in ::login()!
    match User::login(login.into_inner(), db_pool.get_ref()).await {
        Ok(token_string) => {
            let cookie = Cookie::build("session", token_string)
                .domain("/")
                .secure(true)
                .http_only(true)
                .finish();
            HttpResponse::Ok().cookie(cookie).finish()
        }
        Err(err) => err.error_response().into(),
    }
}

#[post("/logout")]
async fn logout(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::log(user.id, format!("logged out"), db_pool.get_ref()).await?;
    match User::logout(user.id, db_pool.get_ref()).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Err(err),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_users);
    cfg.service(find_user);
    cfg.service(find_game_for_ref);
    cfg.service(create_user);
    cfg.service(update_user);
    cfg.service(delete_user);
    cfg.service(login);
    cfg.service(logout);
}
