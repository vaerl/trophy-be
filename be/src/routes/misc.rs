use actix_web::{get, web, HttpRequest, Responder};
use sqlx::PgPool;

use crate::{
    model::{History, LogLevel, LogUserAction, StatusResponse, UserRole, UserToken},
    ApiResult, ToJson,
};

#[get("/ping")]
async fn ping() -> ApiResult<impl Responder> {
    debug!("Received new request: ping.");
    Ok(web::Json(StatusResponse { status: true }))
}

// TODO move this
#[get("/status")]
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

#[get("/done")]
async fn is_done(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await?;

    let action = format!("User {} executed: check if trophy is done", user.id);
    // NOTE rather than implementing Log for () (if even possible), I decided to just update history "manually"
    History::create(user.id, LogLevel::Info, action, &db_pool).await?;

    let res = crate::eval::is_done(&db_pool).await?;
    StatusResponse { status: res }.to_json()
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(ping);
    cfg.service(status);
    cfg.service(is_done);
}
