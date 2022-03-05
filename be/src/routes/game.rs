use actix::Addr;
use actix_web::{
    delete, get, post, put,
    web::{self, Data},
    HttpRequest, Responder,
};
use sqlx::PgPool;

use crate::{
    model::{CreateGame, Game, Log, UserRole, UserToken},
    ws::{lobby::Lobby, socket_refresh::SendRefresh},
    ApiResult, ToJson,
};

#[get("/games")]
async fn find_all_games(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        db_pool.get_ref(),
    )
    .await?;
    Game::find_all(db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await?
        .to_json()
}

#[get("/games/amount")]
async fn games_amount(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        db_pool.get_ref(),
    )
    .await?;
    Game::amount(db_pool.get_ref())
        .await?
        .log_info(
            user.id,
            format!("get the amount of games"),
            db_pool.get_ref(),
        )
        .await?
        .to_json()
}

#[get("/games/pending")]
async fn games_pending(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        db_pool.get_ref(),
    )
    .await?;
    Game::pending(db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await?
        .to_json()
}

#[get("/games/finished")]
async fn games_finished(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        db_pool.get_ref(),
    )
    .await?;
    Game::finished(db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await?
        .to_json()
}

#[post("/games")]
async fn create_game(
    create_game: web::Json<CreateGame>,
    req: HttpRequest,
    db_pool: Data<PgPool>,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<impl Responder> {
    let user =
        UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], db_pool.get_ref()).await?;
    Game::create(create_game.into_inner(), db_pool.get_ref())
        .await?
        .log_create(user.id, db_pool.get_ref())
        .await?
        .send_refresh(lobby_addr.get_ref())?
        .to_json()
}

#[get("/games/{id}")]
async fn find_game(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        db_pool.get_ref(),
    )
    .await?;
    Game::find(id.into_inner(), db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await?
        .to_json()
}

#[put("/games/{id}")]
async fn update_game(
    id: web::Path<i32>,
    game: web::Json<CreateGame>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<impl Responder> {
    let user =
        UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], db_pool.get_ref()).await?;
    Game::update(id.into_inner(), game.into_inner(), db_pool.get_ref())
        .await?
        .log_update(user.id, db_pool.get_ref())
        .await?
        .send_refresh(lobby_addr.get_ref())?
        .to_json()
}

#[delete("/games/{id}")]
async fn delete_game(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<impl Responder> {
    let user =
        UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], db_pool.get_ref()).await?;
    Game::delete(id.into_inner(), db_pool.get_ref())
        .await?
        .log_delete(user.id, db_pool.get_ref())
        .await?
        .send_refresh(lobby_addr.get_ref())?
        .to_json()
}

#[get("/games/{id}/pending")]
async fn pending_teams(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        db_pool.get_ref(),
    )
    .await?;
    Game::pending_teams(id.into_inner(), db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await?
        .to_json()
}

#[get("/games/{id}/pending/amount")]
async fn pending_teams_amount(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        db_pool.get_ref(),
    )
    .await?;
    Game::pending_teams_amount(id.into_inner(), db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await?
        .to_json()
}

#[get("/games/{id}/finished")]
async fn finished_teams(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        db_pool.get_ref(),
    )
    .await?;
    Game::finished_teams(id.into_inner(), db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await?
        .to_json()
}

// NOTE order matters!
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_games);
    cfg.service(games_amount);
    cfg.service(create_game);
    cfg.service(games_pending);
    cfg.service(games_finished);
    cfg.service(find_game);
    cfg.service(update_game);
    cfg.service(delete_game);
    cfg.service(pending_teams);
    cfg.service(pending_teams_amount);
    cfg.service(finished_teams);
}
