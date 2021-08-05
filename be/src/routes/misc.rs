use actix_web::{
    body::Body, get, http::header::ContentType, post, web, HttpRequest, HttpResponse, Responder,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    derive_responder::Responder,
    model::{LogUserAction, UserRole, UserToken},
    ApiResult,
};

#[derive(Serialize, Responder)]
pub struct StatusResponse {
    status: bool,
}

#[get("/ping")]
async fn ping() -> ApiResult<StatusResponse> {
    debug!("Received new request: ping.");
    Ok(StatusResponse { status: true })
}

#[get("/status")]
async fn status(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<StatusResponse> {
    debug!("Received new request: check user-status.");
    match UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], db_pool.get_ref()).await
    {
        Ok(user) => {
            user.log_action(format!("check user-status"), db_pool.get_ref())
                .await?;
            Ok(StatusResponse { status: true })
        }
        Err(_err) => Ok(StatusResponse { status: false }),
    }
}

#[post("/reset/database")]
async fn reset_database(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
    // this resets the database COMPLETELY - use with care!
    let _user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], db_pool.get_ref())
        .await?
        .log_action(format!("reset database"), db_pool.get_ref())
        .await?;

    let mut tx = db_pool.get_ref().begin().await?;
    sqlx::query("DELETE FROM game_team")
        .execute(&mut tx)
        .await?;
    sqlx::query("DELETE FROM games").execute(&mut tx).await?;
    sqlx::query("DELETE FROM teams").execute(&mut tx).await?;
    sqlx::query("DELETE FROM users").execute(&mut tx).await?;
    sqlx::query("DELETE FROM transaction_history")
        .execute(&mut tx)
        .await?;
    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(ping);
    cfg.service(status);
    cfg.service(reset_database);
}
