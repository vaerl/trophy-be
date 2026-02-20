use actix_web::{
    Responder, get, put,
    web::{self, Data},
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    ApiResult, ToJson,
    middleware::Authenticated,
    model::{CustomError, Outcome, User, UserRole},
};

#[get("/outcomes")]
async fn find_all_outcomes(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    Outcome::find_all(&pool).await?.to_json()
}

/// Outcomes are automatically initialized , thus we only need an update-method().
#[put("/outcomes")]
async fn update_outcome(
    pool: Data<PgPool>,
    auth: Authenticated,
    outcome: web::Json<Outcome>,
) -> ApiResult<impl Responder> {
    match auth.role {
        UserRole::Admin => outcome.into_inner().set_data(&pool).await?.to_json(),
        UserRole::Referee => {
            let game = User::find_game_for_ref(auth.id, &pool).await?;

            if outcome.game_id == game.id {
                outcome.into_inner().set_data(&pool).await?.to_json()
            } else {
                Err(CustomError::AccessDeniedError)
            }
        }
        UserRole::Visualizer => Err(CustomError::AccessDeniedError),
    }
}

#[get("/outcomes/teams/{id}")]
async fn find_all_outcomes_for_team(
    pool: Data<PgPool>,
    auth: Authenticated,
    team_id: web::Path<Uuid>,
) -> ApiResult<impl Responder> {
    // only admins should be able to access this information
    auth.has_roles(vec![UserRole::Admin])?;
    Outcome::find_all_for_team(*team_id, &pool).await?.to_json()
}

#[get("/outcomes/games/{id}")]
async fn find_all_outcomes_for_game(
    pool: Data<PgPool>,
    auth: Authenticated,
    game_id: web::Path<Uuid>,
) -> ApiResult<impl Responder> {
    match auth.role {
        UserRole::Admin => Outcome::find_all_for_game(*game_id, &pool).await?.to_json(),
        UserRole::Referee => {
            // if the user is a referee, check if they are accessing the correct game
            if *game_id == User::find_game_for_ref(auth.id, &pool).await?.id {
                Outcome::find_all_for_game(*game_id, &pool).await?.to_json()
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
