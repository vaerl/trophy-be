use actix_web::{delete, get, post, put, web, Responder};
use sqlx::PgPool;

use crate::{
    model::{Amount, CreateTeam, History, Team, TeamVec, UserRole, UserToken},
    ApiResult,
};

#[get("/teams")]
async fn find_all_teams(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<TeamVec> {
    let user = token
        .try_into_authorized_user(
            vec![UserRole::Admin, UserRole::Visualizer],
            db_pool.get_ref(),
        )
        .await?;
    History::read(user.id, format!("find all teams"), db_pool.get_ref()).await?;
    Team::find_all(db_pool.get_ref()).await
}

// TODO determine if this is useful
#[get("/teams/amount")]
async fn teams_amount(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<Amount> {
    let user = token
        .try_into_authorized_user(
            vec![UserRole::Admin, UserRole::Visualizer],
            db_pool.get_ref(),
        )
        .await?;
    History::read(
        user.id,
        format!("get the amount of all teams"),
        db_pool.get_ref(),
    )
    .await?;
    Team::amount(db_pool.get_ref()).await
}

#[post("/teams")]
async fn create_team(
    create_team: web::Json<CreateTeam>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Team> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::action(
        user.id,
        format!("create team"),
        create_team.to_string(),
        db_pool.get_ref(),
    )
    .await?;
    Team::create(create_team.into_inner(), db_pool.get_ref()).await
}

#[get("/teams/{id}")]
async fn find_team(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Team> {
    let user = token
        .try_into_authorized_user(
            vec![UserRole::Admin, UserRole::Visualizer],
            db_pool.get_ref(),
        )
        .await?;
    History::read(user.id, format!("find team {}", id), db_pool.get_ref()).await?;
    Team::find(id.into_inner(), db_pool.get_ref()).await
}

#[put("/teams/{id}")]
async fn update_team(
    id: web::Path<i32>,
    team: web::Json<CreateTeam>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Team> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::action(
        user.id,
        format!("update team {}", id),
        team.to_string(),
        db_pool.get_ref(),
    )
    .await?;
    Team::update(id.into_inner(), team.into_inner(), db_pool.get_ref()).await
}

#[delete("/teams/{id}")]
async fn delete_team(
    id: web::Path<i32>,
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Team> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::action(
        user.id,
        format!("delete team"),
        id.to_string(),
        db_pool.get_ref(),
    )
    .await?;
    Team::delete(id.into_inner(), db_pool.get_ref()).await
}

#[get("/teams/{id}/pending")]
async fn pending_games(
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
    History::read(
        user.id,
        format!("get all pending games for team {}", id),
        db_pool.get_ref(),
    )
    .await?;
    Team::pending_games(id.into_inner(), db_pool.get_ref()).await
}

// TODO check if this is useful
#[get("/teams/{id}/pending/amount")]
async fn pending_games_amount(
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
    History::read(
        user.id,
        format!("get the amount of all pending games for team {}", id),
        db_pool.get_ref(),
    )
    .await?;
    Team::pending_games_amount(id.into_inner(), db_pool.get_ref()).await
}

#[get("/teams/{id}/finished")]
async fn finished_games(
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
    History::read(
        user.id,
        format!("get the all finished games for team {}", id),
        db_pool.get_ref(),
    )
    .await?;
    Team::finished_games(id.into_inner(), db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_teams);
    cfg.service(create_team);
    cfg.service(find_team);
    cfg.service(update_team);
    cfg.service(delete_team);
    cfg.service(pending_games);
    cfg.service(pending_games_amount);
    cfg.service(finished_games);
}
