use actix_web::{delete, get, post, put, web, HttpResponse, Responder, ResponseError};
use sqlx::PgPool;

use crate::model::{CreateGame, Game};

#[get("/games")]
async fn find_all_games(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find all games.");
    let result = Game::find_all(db_pool.get_ref()).await;
    match result {
        Ok(games) => HttpResponse::Ok().json(games),
        Err(err) => err.error_response(),
    }
}

#[get("/games/amount")]
async fn games_amount(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: get the amount of games.");
    let result = Game::amount(db_pool.get_ref()).await;
    match result {
        Ok(amount) => HttpResponse::Ok().json(amount),
        Err(err) => err.error_response(),
    }
}

#[post("/games")]
async fn create_game(
    create_game: web::Json<CreateGame>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Received new request: create game.");
    Game::create(create_game.into_inner(), db_pool.get_ref()).await
}

#[get("/games/{id}")]
async fn find_game(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find game.");
    Game::find(id.into_inner(), db_pool.get_ref()).await
}

#[put("/games/{id}")]
async fn update_game(
    id: web::Path<i32>,
    game: web::Json<CreateGame>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    info!("Received new request: update game.");
    Game::update(id.into_inner(), game.into_inner(), db_pool.get_ref()).await
}

#[delete("/games/{id}")]
async fn delete_game(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: delete game.");
    Game::delete(id.into_inner(), db_pool.get_ref()).await
}

#[get("/games/{id}/pending")]
async fn pending_teams(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find pending teams for game");
    let result = Game::pending_teams(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(teams) => HttpResponse::Ok().json(teams),
        Err(err) => err.error_response(),
    }
}

#[get("/games/{id}/pending/amount")]
async fn pending_teams_amount(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: get the amount of pending teams for game");
    let result = Game::pending_teams_amount(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(size) => HttpResponse::Ok().json(size),
        Err(err) => err.error_response(),
    }
}

#[get("/games/{id}/finished")]
async fn finished_teams(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: find finished teams for game");
    let result = Game::finished_teams(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(teams) => HttpResponse::Ok().json(teams),
        Err(err) => err.error_response(),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_games);
    cfg.service(games_amount);
    cfg.service(create_game);
    cfg.service(find_game);
    cfg.service(update_game);
    cfg.service(delete_game);
    cfg.service(pending_teams);
    cfg.service(pending_teams_amount);
    cfg.service(finished_teams);
}
