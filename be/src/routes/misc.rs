use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;

use crate::{
    model::{LogUserAction, UserRole, UserToken},
    ApiResult,
};

#[get("/ping")]
async fn ping() -> impl Responder {
    debug!("Received new request: ping.");
    HttpResponse::Ok().body("pong")
}

#[get("/status")]
async fn status(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
    debug!("Received new request: check user-status.");
    match UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], db_pool.get_ref()).await
    {
        Ok(user) => {
            user.log_action(format!("check user-status"), db_pool.get_ref())
                .await?;
            Ok(HttpResponse::Ok().body("true"))
        }
        Err(err) => Ok(HttpResponse::Ok().body(err.to_string())),
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
