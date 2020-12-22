use actix_web::{get, post, web, HttpResponse, Responder};
use anyhow::Result;
use sqlx::{Error, PgPool};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body(
        r#"
        trophy-bak

        This backend is currently under development

        Available routes:

        Team
        GET /teams -> list of all teams
        POST /teams -> create new team, body: { "id": 1, "name": "name", "gender": "gender" }
        PUT TODO
        DELETE TODO

        Game
        GET /games -> list of all games
        POST /games -> create new game, body: { "id": 1, "name": "name", "kind": "time" }
        PUT TODO
        DELETE TODO


        Result
        GET /outcomes -> get all outcomes
        PUT /outcomes -> update outcome, body: { "game_id": 1, "team_id": 1, "data": "text"}
        GET /outcomes/games/{id} -> get outcomes for game
        GET /outcomes/teams/{id} -> get outcomes for team

        Miscellaneous
        GET / -> this index
        POST /reset -> reset database
    "#,
    )
}

#[post("/reset/database")]
async fn reset_database(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: reset database.");
    match reset_database_wrapper(db_pool.get_ref()).await {
        Ok(()) => HttpResponse::Ok().body("Successfully reset database."),
        Err(err) => {
            HttpResponse::BadRequest().body(format!("Error trying to reset database: {}", err))
        }
    }
}

async fn reset_database_wrapper(pool: &PgPool) -> Result<(), Error> {
    // TODO check whether doing this is ok
    // TODO check "execute_many"
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM game_team")
        .execute(&mut tx)
        .await?;
    sqlx::query("DELETE FROM game").execute(&mut tx).await?;
    sqlx::query("DELETE FROM team").execute(&mut tx).await?;
    tx.commit().await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(reset_database);
}
