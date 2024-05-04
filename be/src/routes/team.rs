use actix::Addr;
use actix_web::{
    delete, get, post, put,
    web::{self, Data},
    HttpRequest, Responder,
};
use sqlx::PgPool;

use crate::{
    model::{CreateTeam, Log, Team, UserRole, UserToken},
    ws::{lobby::Lobby, socket_refresh::SendRefresh},
    ApiResult, ToJson,
};

#[get("/teams")]
async fn find_all_teams(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        &db_pool,
    )
    .await?;
    Team::find_all(&db_pool)
        .await?
        .log_read(user.id, &db_pool)
        .await?
        .to_json()
}

#[get("/teams/amount")]
async fn teams_amount(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        &db_pool,
    )
    .await?;
    Team::amount(&db_pool)
        .await?
        .log_read(user.id, &db_pool)
        .await?
        .to_json()
}

#[get("/teams/pending")]
async fn teams_pending(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        &db_pool,
    )
    .await?;
    Team::pending(&db_pool)
        .await?
        .log_read(user.id, &db_pool)
        .await?
        .to_json()
}

#[get("/teams/finished")]
async fn teams_finished(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        &db_pool,
    )
    .await?;
    Team::finished(&db_pool)
        .await?
        .log_read(user.id, &db_pool)
        .await?
        .to_json()
}

#[post("/teams")]
async fn create_team(
    create_team: web::Json<CreateTeam>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await?;
    Team::create(create_team.into_inner(), &db_pool)
        .await?
        .log_create(user.id, &db_pool)
        .await?
        .send_refresh(lobby_addr.get_ref())?
        .to_json()
}

#[get("/teams/{id}")]
async fn find_team(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        &db_pool,
    )
    .await?;
    Team::find(id.into_inner(), &db_pool)
        .await?
        .log_read(user.id, &db_pool)
        .await?
        .to_json()
}

#[put("/teams/{id}")]
async fn update_team(
    id: web::Path<i32>,
    team: web::Json<CreateTeam>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await?;
    Team::update(id.into_inner(), team.into_inner(), &db_pool)
        .await?
        .log_update(user.id, &db_pool)
        .await?
        .send_refresh(lobby_addr.get_ref())?
        .to_json()
}

#[delete("/teams/{id}")]
async fn delete_team(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await?;
    Team::delete(id.into_inner(), &db_pool)
        .await?
        .log_delete(user.id, &db_pool)
        .await?
        .send_refresh(lobby_addr.get_ref())?
        .to_json()
}

#[get("/teams/{id}/pending")]
async fn pending_games(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        &db_pool,
    )
    .await?;
    Team::pending_games(id.into_inner(), &db_pool)
        .await?
        .log_read(user.id, &db_pool)
        .await?
        .to_json()
}

#[get("/teams/{id}/finished")]
async fn finished_games(
    id: web::Path<i32>,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Visualizer],
        &db_pool,
    )
    .await?;
    Team::finished_games(id.into_inner(), &db_pool)
        .await?
        .log_read(user.id, &db_pool)
        .await?
        .to_json()
}

// NOTE order matters!
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_teams);
    cfg.service(teams_amount);
    cfg.service(teams_pending);
    cfg.service(teams_finished);
    cfg.service(create_team);
    cfg.service(find_team);
    cfg.service(update_team);
    cfg.service(delete_team);
    cfg.service(pending_games);
    cfg.service(finished_games);
}
