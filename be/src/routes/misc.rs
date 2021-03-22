use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::{
    model::{History, UserRole, UserToken},
    ApiResult,
};

#[get("/ping")]
async fn ping() -> impl Responder {
    debug!("Received new request: ping.");
    HttpResponse::Ok().body("pong")
}

#[post("/reset/database")]
async fn reset_database(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<HttpResponse> {
    // this resets the database COMPLETELY - use with care!
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::log(user.id, format!("reset database"), db_pool.get_ref()).await?;

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
    cfg.service(reset_database);
}
