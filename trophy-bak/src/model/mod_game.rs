use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Row};

use super::Team;

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename = "game_kind")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GameKind {
    Points,
    Time,
}

#[derive(Serialize, FromRow)]
pub struct Game {
    pub id: i32,
    pub name: String,
    pub kind: GameKind,
}

#[derive(Deserialize)]
pub struct GameRequest {
    pub name: String,
    pub kind: GameKind,
}

impl Responder for Game {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl Game {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Game>> {
        let mut games = vec![];
        games = sqlx::query_as!(
            Game,
            r#"SELECT id, name, kind as "kind: GameKind" FROM game ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(games)
    }

    pub async fn create(game: GameRequest, pool: &PgPool) -> Result<Game> {
        let mut tx = pool.begin().await?;
        let game =
            sqlx::query("INSERT INTO game (name, kind) VALUES ($1, $2) RETURNING id, name, kind")
                .bind(&game.name)
                .bind(game.kind)
                .map(|row: PgRow| Game {
                    id: row.get(0),
                    name: row.get(1),
                    kind: row.get(2),
                })
                .fetch_one(&mut tx)
                .await?;

        // TODO populate n:m
        // get all teams
        // add entry for every team
        for team in Team::find_all(pool).await {}

        tx.commit().await?;
        Ok(game)
    }
}
