use std::fmt::format;

use actix_web::{delete, get, post, put, web, Responder};
use sqlx::PgPool;

use crate::{
    model::{Amount, CreateGame, Game, GameVec, History, TeamVec, UserRole, UserToken},
    ApiResult,
};

#[get("/games")]
async fn find_all_games(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<GameVec> {
    let user = token
        .try_into_authorized_user(
            vec![UserRole::Admin, UserRole::Visualizer],
            db_pool.get_ref(),
        )
        .await?;
    History::debug(user.id, format!("find all games"), db_pool.get_ref()).await?;
    Game::find_all(db_pool.get_ref()).await
}

#[get("/games/amount")]
async fn games_amount(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<Amount> {
    let user = token
        .try_into_authorized_user(
            vec![UserRole::Admin, UserRole::Visualizer],
            db_pool.get_ref(),
        )
        .await?;
    History::debug(
        user.id,
        format!("get the amount of games"),
        db_pool.get_ref(),
    )
    .await?;
    Game::amount(db_pool.get_ref()).await
}

#[post("/games")]
async fn create_game(
    create_game: web::Json<CreateGame>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Game> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::info(
        user.id,
        format!("create a new game"),
        // apparently calling fmt() on Json<T> delegates the call to T
        create_game.to_string(),
        db_pool.get_ref(),
    )
    .await?;
    Game::create(create_game.into_inner(), db_pool.get_ref()).await
}

#[get("/games/{id}")]
async fn find_game(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Game> {
    let user = token
        .try_into_authorized_user(
            vec![UserRole::Admin, UserRole::Visualizer],
            db_pool.get_ref(),
        )
        .await?;
    History::debug(user.id, format!("find game {}", id), db_pool.get_ref()).await?;
    Game::find(id.into_inner(), db_pool.get_ref()).await
}

#[put("/games/{id}")]
async fn update_game(
    id: web::Path<i32>,
    game: web::Json<CreateGame>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Game> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::info(
        user.id,
        format!("update game {}", id),
        game.to_string(),
        db_pool.get_ref(),
    )
    .await?;
    Game::update(id.into_inner(), game.into_inner(), db_pool.get_ref()).await
}

#[delete("/games/{id}")]
async fn delete_game(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Game> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::info(
        user.id,
        format!("delete game"),
        format!("{}", id),
        db_pool.get_ref(),
    )
    .await?;
    Game::delete(id.into_inner(), db_pool.get_ref()).await
}

#[get("/games/{id}/pending")]
async fn pending_teams(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<TeamVec> {
    let user = token
        .try_into_authorized_user(
            vec![UserRole::Admin, UserRole::Visualizer],
            db_pool.get_ref(),
        )
        .await?;
    History::debug(
        user.id,
        format!("get the pending teams for game {}", id),
        db_pool.get_ref(),
    )
    .await?;
    Game::pending_teams(id.into_inner(), db_pool.get_ref()).await
}

#[get("/games/{id}/pending/amount")]
async fn pending_teams_amount(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Amount> {
    let user = token
        .try_into_authorized_user(
            vec![UserRole::Admin, UserRole::Visualizer],
            db_pool.get_ref(),
        )
        .await?;
    History::debug(
        user.id,
        format!("get the amount of pending teams for game {}", id),
        db_pool.get_ref(),
    )
    .await?;
    Game::pending_teams_amount(id.into_inner(), db_pool.get_ref()).await
}

#[get("/games/{id}/finished")]
async fn finished_teams(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let user = token
        .try_into_authorized_user(
            vec![UserRole::Admin, UserRole::Visualizer],
            db_pool.get_ref(),
        )
        .await?;
    History::debug(
        user.id,
        format!("get the finished teams for game {}", id),
        db_pool.get_ref(),
    )
    .await?;
    Game::finished_teams(id.into_inner(), db_pool.get_ref()).await
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
