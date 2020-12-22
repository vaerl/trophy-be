use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::model::Outcome;

#[get("/outcomes")]
async fn find_all_outcomes(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find all outcomes.");
    let result = Outcome::find_all(db_pool.get_ref()).await;
    match result {
        Ok(outcomes) => HttpResponse::Ok().json(outcomes),
        Err(err) => HttpResponse::BadRequest().body(format!(
            "Error trying to read all outcomes from database: {}",
            err
        )),
    }
}

#[put("/outcomes")]
async fn create_outcome(outcome: web::Json<Outcome>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: update outcome.");
    let result = Outcome::update(outcome.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(outcome) => HttpResponse::Ok().json(outcome),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to create outcome: {}", err))
        }
    }
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
        Err(err) => HttpResponse::BadRequest().body(format!(
            "Error trying to read all outcomes for team: {}",
            err
        )),
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
        Err(err) => HttpResponse::BadRequest().body(format!(
            "Error trying to read all outcomes for game: {}",
            err
        )),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_outcomes);
    cfg.service(create_outcome);
    cfg.service(find_all_outcomes_for_game);
    cfg.service(find_all_outcomes_for_team);
}
