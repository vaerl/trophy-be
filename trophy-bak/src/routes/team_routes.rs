use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::model::Team;

#[get("/teams")]
async fn find_all_teams(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find all teams.");
    let result = Team::find_all(db_pool.get_ref()).await;
    match result {
        Ok(teams) => HttpResponse::Ok().json(teams),
        Err(err) => HttpResponse::BadRequest().body(format!(
            "Error trying to read all teams from database: {}",
            err
        )),
    }
}

#[post("/teams")]
async fn create_team(team: web::Json<Team>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: create team.");
    let result = Team::create(team.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to create new team: {}", err))
        }
    }
}

#[get("/teams/{id}")]
async fn find_team(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find team.");
    let result = Team::find(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(err) => HttpResponse::BadRequest().body(format!("Error trying to find team: {}", err)),
    }
}

#[put("/teams/{id}")]
async fn update_team(
    id: web::Path<i32>,
    team: web::Json<Team>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Received new request: update team.");
    let result = Team::update(id.into_inner(), team.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to update team: {}", err))
        }
    }
}

#[delete("/teams/{id}")]
async fn delete_team(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: delete team.");
    let result = Team::delete(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to delete team: {}", err))
        }
    }
}

// function that will be called on new Application to configure routes for this module
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_teams);
    cfg.service(create_team);
    cfg.service(find_team);
    cfg.service(update_team);
    cfg.service(delete_team);
}
