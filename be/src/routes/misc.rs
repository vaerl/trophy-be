use crate::{
    middleware::Authenticated,
    model::{StatusResponse, UserRole},
    ApiResult, ToJson,
};
use actix_web::{
    get,
    web::{self, Data},
    Responder,
};
use sqlx::PgPool;

#[get("/ping")]
async fn ping() -> ApiResult<impl Responder> {
    Ok(web::Json(StatusResponse { status: true }))
}

#[get("/done")]
async fn is_done(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    StatusResponse {
        status: crate::eval::is_done(&pool).await?,
    }
    .to_json()
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(ping);
    cfg.service(is_done);
}
