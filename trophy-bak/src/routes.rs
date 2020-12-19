use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::model::{Game, GameRequest, Team, TeamRequest};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(
        r#"
        trophy-bak

        This backend is currently under development

        Available routes:
        GET /teams -> list of all teams
        POST /teams -> create new team, example: { "name": "name", "gender": "gender" }
    "#,
    )
}

// Game

#[get("/games")]
async fn find_all_games(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find all games.");
    let result = Game::find_all(db_pool.get_ref()).await;
    match result {
        Ok(todos) => HttpResponse::Ok().json(todos),
        Err(err) => HttpResponse::BadRequest().body(format!(
            "Error trying to read all games from database: {}",
            err
        )),
    }
}

#[post("/games")]
async fn create_game(game: web::Json<GameRequest>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: create game.");
    let result = Game::create(game.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to create new game: {}", err))
        }
    }
}

// Team

#[get("/teams")]
async fn find_all_teams(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find all teams.");
    let result = Team::find_all(db_pool.get_ref()).await;
    match result {
        Ok(todos) => HttpResponse::Ok().json(todos),
        Err(err) => HttpResponse::BadRequest().body(format!(
            "Error trying to read all teams from database: {}",
            err
        )),
    }
}

#[post("/teams")]
async fn create_team(team: web::Json<TeamRequest>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: create team.");
    let result = Team::create(team.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(team) => HttpResponse::Ok().json(team),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to create new team: {}", err))
        }
    }
}

// function that will be called on new Application to configure routes for this module
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(find_all_games);
    cfg.service(create_game);
    cfg.service(find_all_teams);
    cfg.service(create_team);
}
