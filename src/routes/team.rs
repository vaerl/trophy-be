use actix_web::{
    delete, get, post, put,
    web::{self, Data, Query},
    Responder,
};
use sqlx::PgPool;

use crate::{
    middleware::Authenticated,
    model::{CreateTeam, Outcome, Team, UserRole, Year},
    ApiResult, ToJson,
};

#[get("/teams")]
async fn find_all_teams(
    pool: Data<PgPool>,
    auth: Authenticated,
    year: Query<Year>,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Team::find_all(&pool, **year).await?.to_json()
}

#[get("/teams/pending/amount")]
async fn teams_pending(
    pool: Data<PgPool>,
    auth: Authenticated,
    year: Query<Year>,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Outcome::find_all_pending_teams(**year, &pool)
        .await?
        .to_json()
}

#[post("/teams")]
async fn create_team(
    create_team: web::Json<CreateTeam>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    Team::create(create_team.into_inner(), &pool)
        .await?
        .to_json()
}

#[get("/teams/{id}")]
async fn find_team(
    id: web::Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin, UserRole::Visualizer])?;
    Team::find(*id, &pool).await?.to_json()
}

#[put("/teams/{id}")]
async fn update_team(
    id: web::Path<i32>,
    team: web::Json<CreateTeam>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    Team::update(*id, team.into_inner(), &pool).await?.to_json()
}

#[delete("/teams/{id}")]
async fn delete_team(
    id: web::Path<i32>,
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    Team::delete(*id, &pool).await?.to_json()
}

// NOTE order matters!
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_teams);
    cfg.service(teams_pending);
    cfg.service(create_team);
    cfg.service(find_team);
    cfg.service(update_team);
    cfg.service(delete_team);
}
