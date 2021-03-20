use actix_web::{get, put, web};
use sqlx::PgPool;

use crate::{
    model::{History, Outcome, OutcomeVec, UserRole, UserToken},
    ApiResult,
};

#[get("/outcomes")]
async fn find_all_outcomes(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<OutcomeVec> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::log(
        user.id,
        format!("get the finished teams for a game"),
        db_pool.get_ref(),
    )
    .await?;
    Outcome::find_all(db_pool.get_ref()).await
}

#[put("/outcomes")]
async fn create_outcome(
    token: UserToken,
    outcome: web::Json<Outcome>,
    db_pool: web::Data<PgPool>,
) -> ApiResult<Outcome> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin, UserRole::Referee], db_pool.get_ref())
        .await?;
    History::log(
        user.id,
        format!("create outcome for team"),
        db_pool.get_ref(),
    )
    .await?;
    Outcome::update(outcome.into_inner(), db_pool.get_ref()).await
}

#[get("/outcomes/teams/{id}")]
async fn find_all_outcomes_for_team(
    token: UserToken,
    team_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> ApiResult<OutcomeVec> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::log(
        user.id,
        format!("find all outcomes for team"),
        db_pool.get_ref(),
    )
    .await?;
    Outcome::find_all_for_team(team_id.into_inner(), db_pool.get_ref()).await
}

#[get("/outcomes/games/{id}")]
async fn find_all_outcomes_for_game(
    token: UserToken,
    game_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> ApiResult<OutcomeVec> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::log(
        user.id,
        format!("find all outcomes for game"),
        db_pool.get_ref(),
    )
    .await?;
    Outcome::find_all_for_game(game_id.into_inner(), db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_outcomes);
    cfg.service(create_outcome);
    cfg.service(find_all_outcomes_for_game);
    cfg.service(find_all_outcomes_for_team);
}
