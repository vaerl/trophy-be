use crate::{
    model::{
        CreateLogin, CreateUser, Log, LogUserAction, StatusResponse, User, UserRole, UserToken,
    },
    ApiResult, ToJson,
};
use actix_web::{
    cookie::{Cookie, SameSite},
    delete, get, post, put, web, HttpRequest, HttpResponse, Responder, ResponseError,
};
use sqlx::PgPool;
use std::env;

#[get("/user/status")]
async fn status(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    debug!("Received new request: check user-status.");
    match UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await {
        Ok(user) => {
            user.log_action(format!("check user-status"), &db_pool)
                .await?;
            Ok(web::Json(StatusResponse { status: true }))
        }
        Err(_err) => Ok(web::Json(StatusResponse { status: false })),
    }
}

#[get("/users")]
async fn find_all_users(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await?;
    User::find_all(&db_pool)
        .await?
        .log_read(user.id, &db_pool)
        .await?
        .to_json()
}

#[get("/users/{id}")]
async fn find_user(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await?;
    User::find(id.into_inner(), &db_pool)
        .await?
        .log_read(user.id, &db_pool)
        .await?
        .to_json()
}

#[get("/users/{id}/game")]
async fn find_game_for_ref(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Referee],
        &db_pool,
    )
    .await?;
    User::find_game_for_ref(id.into_inner(), &db_pool)
        .await?
        .log_read(user.id, &db_pool)
        .await?
        .to_json()
}

#[post("/users")]
async fn create_user(
    create_user: web::Json<CreateUser>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await?;
    User::create(create_user.into_inner(), &db_pool)
        .await?
        .log_create(user.id, &db_pool)
        .await?
        .to_json()
}

#[put("/users/{id}")]
async fn update_user(
    id: web::Path<i32>,
    altered_user: web::Json<CreateUser>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await?;
    User::update(id.into_inner(), altered_user.into_inner(), &db_pool)
        .await?
        .log_update(user.id, &db_pool)
        .await?
        .to_json()
}

#[delete("/users/{id}")]
async fn delete_user(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await?;
    User::delete(id.into_inner(), &db_pool)
        .await?
        .log_delete(user.id, &db_pool)
        .await?
        .to_json()
}

/// # Behavior when logging in twice:
///
/// Logging in from a different browser or device changes the users' session.
/// This results in the old session being invalid. The specific check is done
/// in `try_into_authorized_user` in [UserToken].
/// _Supporting multiple sessions per user should be unnecessary, just add more users.
/// If this bit is ever needed the [user](User) needs to support multiple sessions._
#[post("/login")]
async fn login(login: web::Json<CreateLogin>, db_pool: web::Data<PgPool>) -> impl Responder {
    let secure = env::var("COOKIE_SECURE").expect("COOKIE_SECURE is not set in .env file!");

    // NOTE logging is done in ::login()!
    match User::login(login.into_inner(), &db_pool).await {
        Ok(token_string) => {
            let cookie = Cookie::build("session", token_string)
                .path("/")
                .secure(secure.parse::<bool>().unwrap())
                .http_only(true)
                .same_site(SameSite::None)
                .finish();
            HttpResponse::Ok().cookie(cookie).finish()
        }
        Err(err) => err.error_response().into(),
    }
}

#[post("/logout")]
async fn logout(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool)
        .await?
        .log_action(format!("logged out"), &db_pool)
        .await?;
    match User::logout(user.id, &db_pool).await {
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
    cfg.service(status);
}
