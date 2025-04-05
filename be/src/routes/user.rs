use crate::{
    middleware::Authenticated,
    model::{CreateLogin, CreateUser, StatusResponse, UpdateUser, User, UserRole},
    ApiResult, ToJson,
};
use actix_web::{
    cookie::{Cookie, SameSite},
    delete, get, post, put,
    web::{self, Data},
    HttpResponse, Responder, ResponseError,
};
use sqlx::PgPool;

#[get("/user/status")]
async fn status(auth: Option<Authenticated>) -> ApiResult<impl Responder> {
    match auth {
        Some(_) => Ok(web::Json(StatusResponse { status: true })),
        None => Ok(web::Json(StatusResponse { status: false })),
    }
}

#[get("/users")]
async fn find_all_users(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    User::find_all(&pool).await?.to_json()
}

#[get("/users/{id}")]
async fn find_user(
    id: web::Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    User::find(*id, &pool).await?.to_json()
}

#[get("/users/{id}/game")]
async fn find_game_for_ref(
    id: web::Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Referee])?;
    User::find_game_for_ref(*id, &pool).await?.to_json()
}

#[post("/users")]
async fn create_user(
    create_user: web::Json<CreateUser>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    User::create(create_user.into_inner(), &pool)
        .await?
        .to_json()
}

#[put("/users/{id}")]
async fn update_user(
    id: web::Path<i32>,
    altered_user: web::Json<UpdateUser>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    User::update(*id, altered_user.into_inner(), &pool)
        .await?
        .to_json()
}

#[delete("/users/{id}")]
async fn delete_user(
    id: web::Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    User::delete(*id, &pool).await?.to_json()
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
    match User::login(login.into_inner(), &db_pool).await {
        Ok(token_string) => {
            let cookie = Cookie::build("session", token_string)
                .path("/")
                .secure(true)
                .http_only(true)
                .same_site(SameSite::None)
                .finish();
            HttpResponse::Ok().cookie(cookie).finish()
        }
        Err(err) => err.error_response().into(),
    }
}

#[post("/logout")]
async fn logout(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<HttpResponse> {
    // NOTE this should probably work for all user-types
    match User::logout(auth.id, &pool).await {
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
