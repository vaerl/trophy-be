use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use anyhow::Result;
use sqlx::PgPool;

use crate::model::CustomError;

#[get("/ping")]
async fn ping() -> impl Responder {
    debug!("Received new request: ping.");
    HttpResponse::Ok().body("pong")
}

#[post("/reset/database")]
async fn reset_database(db_pool: web::Data<PgPool>) -> impl Responder {
    // This resets the database COMPLETELY - use with care!
    // TODO remove this (at least comment it) before moving to production
    warn!("Received new request: reset database.");
    match reset_database_wrapper(db_pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().body("Successfully reset database."),
        Err(err) => err.error_response(),
    }
}

async fn reset_database_wrapper(pool: &PgPool) -> Result<(), CustomError> {
    // This is a wrapper-function for resetting the database as I wanted to use await
    // (which is only possible when Returning an Result).
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM game_team")
        .execute(&mut tx)
        .await?;
    sqlx::query("DELETE FROM game").execute(&mut tx).await?;
    sqlx::query("DELETE FROM team").execute(&mut tx).await?;
    Ok(tx.commit().await?)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(ping);
    cfg.service(reset_database);
}
