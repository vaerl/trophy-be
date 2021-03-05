use actix_web::{get, put, web, HttpResponse, Responder, ResponseError};
use sqlx::PgPool;

use crate::model::Outcome;

/// This module provides all routes concerning outcomes.
/// As the name "Result" was already taken for the programming-structure, I'm using "outcome".

#[get("/outcomes")]
async fn find_all_outcomes(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find all outcomes.");
    let result = Outcome::find_all(db_pool.get_ref()).await;
    match result {
        Ok(outcomes) => HttpResponse::Ok().json(outcomes),
        Err(err) => err.error_response(),
    }
}

#[put("/outcomes")]
async fn create_outcome(outcome: web::Json<Outcome>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: update outcome.");
    Outcome::update(outcome.into_inner(), db_pool.get_ref()).await
}

#[get("/outcomes/teams/{id}")]
async fn find_all_outcomes_for_team(
    team_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Received new request: find all outcomes for team.");
    let result = Outcome::find_all_for_team(team_id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(outcomes) => HttpResponse::Ok().json(outcomes),
        Err(err) => err.error_response(),
    }
}

#[get("/outcomes/games/{id}")]
async fn find_all_outcomes_for_game(
    game_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Received new request: find all outcomes for game.");
    let result = Outcome::find_all_for_game(game_id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(outcomes) => HttpResponse::Ok().json(outcomes),
        Err(err) => err.error_response(),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_outcomes);
    cfg.service(create_outcome);
    cfg.service(find_all_outcomes_for_game);
    cfg.service(find_all_outcomes_for_team);
}
