use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::model::Game;

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

#[put("/games")]
async fn update_game(game: web::Json<Game>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: update game.");
    let result = Game::update(game.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(game) => HttpResponse::Ok().json(game),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to update game: {}", err))
        }
    }
}

#[delete("/games")]
async fn delete_game(game: web::Json<Game>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: delete game.");
    let result = Game::delete(game.into_inner(), db_pool.get_ref()).await;
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
    cfg.service(update_game);
    cfg.service(delete_game);
}
