use actix::Addr;
use actix_web::{get, put, web::{self, Data}};
use sqlx::PgPool;

use crate::{ApiResult, model::{CustomError, Log, Outcome, OutcomeVec, User, UserRole, UserToken}, ws::{lobby::Lobby, socket_refresh::SendRefresh}};

#[get("/outcomes")]
async fn find_all_outcomes(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<OutcomeVec> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    Outcome::find_all(db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await
}

/// Outcomes are automatically initialized , thus we only need an update-method().
#[put("/outcomes/{id}")]
async fn update_outcome(
    token: UserToken,
    outcome: web::Json<Outcome>,
    db_pool: web::Data<PgPool>,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<Outcome> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin, UserRole::Referee], db_pool.get_ref())
        .await?;

    match user.role {
        UserRole::Admin => {
            Outcome::update(outcome.into_inner(), db_pool.get_ref())
                .await?
                .log_update(user.id, db_pool.get_ref())
                .await
        }
        UserRole::Referee => {
            // if the user is a referee, check if he is accessing the correct game
            if outcome.game_id
                == User::find_game_for_ref(user.id, db_pool.get_ref())
                    .await?
                    .id
            {
                Outcome::update(outcome.into_inner(), db_pool.get_ref())
                    .await?
                    .log_update(user.id, db_pool.get_ref())
                    .await?
                    .send_refresh(lobby_addr.get_ref())
            } else {
                Err(CustomError::AccessDeniedError)
            }
        }
        UserRole::Visualizer => Err(CustomError::AccessDeniedError),
    }
}

#[get("/outcomes/teams/{id}")]
async fn find_all_outcomes_for_team(
    token: UserToken,
    team_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> ApiResult<OutcomeVec> {
    // only admins should be able to access this information
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    Outcome::find_all_for_team(team_id.into_inner(), db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await
}

#[get("/outcomes/games/{id}")]
async fn find_all_outcomes_for_game(
    token: UserToken,
    game_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> ApiResult<OutcomeVec> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin, UserRole::Referee], db_pool.get_ref())
        .await?;
    let game_id = game_id.into_inner();

    match user.role {
        UserRole::Admin => {
            Outcome::find_all_for_game(game_id, db_pool.get_ref())
                .await?
                .log_read(user.id, db_pool.get_ref())
                .await
        }
        UserRole::Referee => {
            // if the user is a referee, check if he is accessing the correct game
            if game_id
                == User::find_game_for_ref(user.id, db_pool.get_ref())
                    .await?
                    .id
            {
                Outcome::find_all_for_game(game_id, db_pool.get_ref())
                    .await?
                    .log_read(user.id, db_pool.get_ref())
                    .await
            } else {
                Err(CustomError::AccessDeniedError)
            }
        }
        UserRole::Visualizer => Err(CustomError::AccessDeniedError),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_outcomes);
    cfg.service(update_outcome);
    cfg.service(find_all_outcomes_for_game);
    cfg.service(find_all_outcomes_for_team);
}
