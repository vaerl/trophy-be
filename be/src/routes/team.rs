use actix::Addr;
use actix_web::{
    delete, get, post, put,
    web::{self, Data},
    Responder,
};
use sqlx::PgPool;

use crate::{
    middleware::Authenticated,
    model::{CreateTeam, Team, UserRole},
    ws::{lobby::Lobby, socket_refresh::SendRefresh},
    ApiResult, ToJson,
};

#[get("/teams")]
async fn find_all_teams(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Team::find_all(&pool).await?.to_json()
}

#[get("/teams/amount")]
async fn teams_amount(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Team::amount(&pool).await?.to_json()
}

#[get("/teams/pending")]
async fn teams_pending(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Team::pending(&pool).await?.to_json()
}

#[get("/teams/finished")]
async fn teams_finished(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Team::finished(&pool).await?.to_json()
}

#[post("/teams")]
async fn create_team(
    create_team: web::Json<CreateTeam>,
    pool: Data<PgPool>,
    auth: Authenticated,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    Team::create(create_team.into_inner(), &pool)
        .await?
        .send_refresh(&lobby_addr)?
        .to_json()
}

#[get("/teams/{id}")]
async fn find_team(
    id: web::Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Team::find(id.into_inner(), &pool).await?.to_json()
}

#[put("/teams/{id}")]
async fn update_team(
    id: web::Path<i32>,
    team: web::Json<CreateTeam>,
    pool: Data<PgPool>,
    auth: Authenticated,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    Team::update(id.into_inner(), team.into_inner(), &pool)
        .await?
        .send_refresh(&lobby_addr)?
        .to_json()
}

#[delete("/teams/{id}")]
async fn delete_team(
    id: web::Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    Team::delete(id.into_inner(), &pool)
        .await?
        .send_refresh(&lobby_addr)?
        .to_json()
}

#[get("/teams/{id}/pending")]
async fn pending_games(
    id: web::Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Team::pending_games(id.into_inner(), &pool).await?.to_json()
}

#[get("/teams/{id}/finished")]
async fn finished_games(
    id: web::Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Team::finished_games(id.into_inner(), &pool)
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
