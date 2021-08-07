use actix::Addr;
use actix_web::{
    get, put,
    web::{self, Data},
    HttpRequest,
};
use sqlx::PgPool;

use crate::{
    model::{CustomError, Log, Outcome, OutcomeVec, User, UserRole, UserToken},
    ws::{lobby::Lobby, socket_refresh::SendRefresh},
    ApiResult,
};

#[get("/outcomes")]
async fn find_all_outcomes(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<OutcomeVec> {
    let user =
        UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], db_pool.get_ref()).await?;
    Outcome::find_all(db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await
}

/// Outcomes are automatically initialized , thus we only need an update-method().
#[put("/outcomes")]
async fn update_outcome(
    req: HttpRequest,
    outcome: web::Json<Outcome>,
    db_pool: web::Data<PgPool>,
    lobby_addr: Data<Addr<Lobby>>,
) -> ApiResult<Outcome> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Referee],
        db_pool.get_ref(),
    )
    .await?;

    match user.role {
        UserRole::Admin => {
            outcome
                .into_inner()
                .set_data(&user, lobby_addr.get_ref(), db_pool.get_ref())
                .await?
                .log_update(user.id, db_pool.get_ref())
                .await
        }
        UserRole::Referee => {
            let game = User::find_game_for_ref(user.id, db_pool.get_ref()).await?;

            // check if the ref is accessing the correct game and only allow updating if it's not locked yet
            if outcome.game_id == game.id && !game.locked {
                outcome
                    .into_inner()
                    .set_data(&user, lobby_addr.get_ref(), db_pool.get_ref())
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
    req: HttpRequest,
    team_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> ApiResult<OutcomeVec> {
    // only admins should be able to access this information
    let user =
        UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], db_pool.get_ref()).await?;
    Outcome::find_all_for_team(team_id.into_inner(), db_pool.get_ref())
        .await?
        .log_read(user.id, db_pool.get_ref())
        .await
}

#[get("/outcomes/games/{id}")]
async fn find_all_outcomes_for_game(
    req: HttpRequest,
    game_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> ApiResult<OutcomeVec> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Referee],
        db_pool.get_ref(),
    )
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
