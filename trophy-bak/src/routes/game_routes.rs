use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::model::Game;

#[get("/games")]
async fn find_all_games(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find all games.");
    let result = Game::find_all(db_pool.get_ref()).await;
    match result {
        Ok(games) => HttpResponse::Ok().json(games),
        Err(err) => HttpResponse::BadRequest().body(format!(
            "Error trying to read all games from database: {}",
            err
        )),
    }
}

#[post("/games")]
async fn create_game(game: web::Json<Game>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: create game.");
    let result = Game::create(game.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(game) => HttpResponse::Ok().json(game),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to create new game: {}", err))
        }
    }
}

#[get("/games/{id}")]
async fn find_game(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find game.");
    let result = Game::find(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(game) => HttpResponse::Ok().json(game),
        Err(err) => HttpResponse::BadRequest().body(format!(
            "Error trying to read all games from database: {}",
            err
        )),
    }
}

#[put("/games/{id}")]
async fn update_game(
    id: web::Path<i32>,
    game: web::Json<Game>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Received new request: update game.");
    let result = Game::update(id.into_inner(), game.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(game) => HttpResponse::Ok().json(game),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to update game: {}", err))
        }
    }
}

#[delete("/games/{id}")]
async fn delete_game(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: delete game.");
    let result = Game::delete(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(game) => HttpResponse::Ok().json(game),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to delete game: {}", err))
        }
    }
}

// function that will be called on new Application to configure routes for this module
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_games);
    cfg.service(create_game);
    cfg.service(find_game);
    cfg.service(update_game);
    cfg.service(delete_game);
}
