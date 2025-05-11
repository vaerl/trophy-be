use crate::{
    middleware::Authenticated,
    model::{CreateGame, Game, Outcome, UserRole, Year},
    ApiResult, ToJson,
};
use actix_web::{
    delete, get, post, put,
    web::{self, Data, Json, Path, Query},
    Responder,
};
use sqlx::PgPool;

#[get("/games")]
async fn find_all_games(
    pool: Data<PgPool>,
    auth: Authenticated,
    year: Query<Year>,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Game::find_all(&pool, **year).await?.to_json()
}

#[get("/games/pending/amount")]
async fn games_pending(
    pool: Data<PgPool>,
    auth: Authenticated,
    year: Query<Year>,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Outcome::find_all_pending_games(**year, &pool)
        .await?
        .to_json()
}

#[post("/games")]
async fn create_game(
    create_game: Json<CreateGame>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    Game::create(create_game.into_inner(), &pool)
        .await?
        .to_json()
}

#[get("/games/{id}")]
async fn find_game(
    id: Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Game::find(*id, &pool).await?.to_json()
}

#[put("/games/{id}")]
async fn update_game(
    id: Path<i32>,
    game: Json<CreateGame>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    Game::update(*id, game.into_inner(), &pool).await?.to_json()
}

#[delete("/games/{id}")]
async fn delete_game(
    id: Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    Game::delete(*id, &pool).await?.to_json()
}

#[get("/games/{id}/pending/amount")]
async fn pending_teams_amount(
    id: web::Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Outcome::find_all_pending_teams_for_game(*id, &pool)
        .await?
        .to_json()
}

// NOTE order matters!
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_games);
    cfg.service(create_game);
    cfg.service(games_pending);
    cfg.service(find_game);
    cfg.service(update_game);
    cfg.service(delete_game);
    cfg.service(pending_teams_amount);
}
