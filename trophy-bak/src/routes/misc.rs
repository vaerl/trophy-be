use actix_web::{get, post, web, HttpResponse, Responder};
use anyhow::Result;
use sqlx::{Error, PgPool};

#[get("/")]
async fn index() -> impl Responder {
    // This function provides an overview of all the different routes.
    // TODO replace this with swagger-doc or similar
    HttpResponse::Ok().body(
        r#"
        KL-Bak

        This backend is currently under development

        Available routes:

        Team
        GET /teams -> list of all teams
        GET /teams/amount -> amount of all teams
        POST /teams -> create new team, body: { "id": 1, "name": "test", "gender": "male", "points": 0 }
        GET /teams/id -> find team
        PUT /teams/id -> update team, body: { "id": 1, "name": "test2", "gender": "mixed", "points": 0 }
        DELETE /teams/id -> delete team
        GET /teams/id/pending -> pending games
        GET /teams/id/pending/amount -> amount of pending games
        GET /teams/id/finished -> finished games

        Game
        GET /games -> list of all games
        GET /games/amount -> amount of all games
        POST /games -> create new game, body: { "id": 1, "name": "name", "kind": "time" }
        GET /games(id) -> find game
        PUT /games/id -> update game, body: { "id": 1, "name": "name2", "kind": "time" }
        DELETE /games -> delete game, body: { "id": 1, "name": "name2", "kind": "time" }
        GET /games/id/pending -> pending teams
        GET /games/id/pending/amount -> amount of pending teams
        GET /games/id/finished -> finished teams

        Result
        GET /outcomes -> get all outcomes
        PUT /outcomes -> update outcome, body: { "game_id": 1, "team_id": 1, "data": "text"}
        GET /outcomes/games/{id} -> get outcomes for game
        GET /outcomes/teams/{id} -> get outcomes for team

        Evaluate
        POST /eval/game_id -> checks if game is done and then evaluates said game
        GET /eval -> builds an .xlsx-file and serves it

        Miscellaneous
        GET / -> this index
        POST /reset -> reset database
    "#,
    )
}

#[post("/reset/database")]
async fn reset_database(db_pool: web::Data<PgPool>) -> impl Responder {
    // This resets the database COMPLETELY - use with care!
    // TODO do I keep this?
    warn!("Received new request: reset database.");
    match reset_database_wrapper(db_pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().body("Successfully reset database."),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to reset database: {}", err))
        }
    }
}

async fn reset_database_wrapper(pool: &PgPool) -> Result<(), Error> {
    // This is a wrapper-function for resetting the database as I wanted to use await
    // (which is only possible when Returning an Result).
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM game_team")
        .execute(&mut tx)
        .await?;
    sqlx::query("DELETE FROM game").execute(&mut tx).await?;
    sqlx::query("DELETE FROM team").execute(&mut tx).await?;
    tx.commit().await
}

#[get("/ping")]
async fn ping() -> impl Responder {
    debug!("Received new request: ping.");
    HttpResponse::Ok().body("Pong.")
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(reset_database);
}
