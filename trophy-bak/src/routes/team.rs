use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use sqlx::PgPool;

use crate::model::{CreateTeam, Team};

#[get("/teams")]
async fn find_all_teams(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find all teams.");
    let result = Team::find_all(db_pool.get_ref()).await;
    match result {
        Ok(teams) => HttpResponse::Ok().json(teams),
        Err(err) => err.error_response(),
    }
}

#[get("/teams/amount")]
async fn teams_amount(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: get the amount of teams.");
    let result = Team::amount(db_pool.get_ref()).await;
    match result {
        Ok(amount) => HttpResponse::Ok().json(amount),
        Err(err) => err.error_response(),
    }
}

#[post("/teams")]
async fn create_team(
    create_team: web::Json<CreateTeam>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Received new request: create team.");
    Team::create(create_team.into_inner(), db_pool.get_ref()).await
}

#[get("/teams/{id}")]
async fn find_team(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find team.");
    Team::find(id.into_inner(), db_pool.get_ref()).await
}

#[put("/teams/{id}")]
async fn update_team(
    id: web::Path<i32>,
    team: web::Json<CreateTeam>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Received new request: update team.");
    Team::update(id.into_inner(), team.into_inner(), db_pool.get_ref()).await
}

#[delete("/teams/{id}")]
async fn delete_team(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: delete team.");
    Team::delete(id.into_inner(), db_pool.get_ref()).await
}

#[get("/teams/{id}/pending")]
async fn pending_games(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find pending games for team");
    let result = Team::pending_games(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(games) => HttpResponse::Ok().json(games),
        Err(err) => err.error_response(),
    }
}

#[get("/teams/{id}/pending/amount")]
async fn pending_games_amount(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: get the amount of pending teams for game");
    let result = Team::pending_games_amount(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(size) => HttpResponse::Ok().json(size),
        Err(err) => err.error_response(),
    }
}

#[get("/teams/{id}/finished")]
async fn finished_games(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find finished teams for game");
    let result = Team::finished_games(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(games) => HttpResponse::Ok().json(games),
        Err(err) => err.error_response(),
    }
}

// TODO: check_status -> respond with int

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_teams);
    cfg.service(create_team);
    cfg.service(find_team);
    cfg.service(update_team);
    cfg.service(delete_team);
    cfg.service(pending_games);
    cfg.service(pending_games_amount);
    cfg.service(finished_games);
}
