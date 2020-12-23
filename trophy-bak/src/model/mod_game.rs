use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use super::{Outcome, Team};

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename = "game_kind")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GameKind {
    Points,
    Time,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct Game {
    pub id: i32,
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
        let games = sqlx::query_as!(
            Game,
            r#"SELECT id, name, kind as "kind: GameKind" FROM game ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(games)
    }

    pub async fn create(game: Game, pool: &PgPool) -> Result<Game> {
        let mut tx = pool.begin().await?;
        let game = sqlx::query_as!( Game, 
            r#"INSERT INTO game (id, name, kind) VALUES ($1, $2, $3) RETURNING id, name, kind as "kind: GameKind""#,
            game.id, game.name, game.kind as GameKind
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        for team in Team::find_all(pool).await? {
            Outcome::create(game.id, team.id, pool).await?;
        }

        Ok(game)
    }

    pub async fn update(game: Game, pool: &PgPool) -> Result<Game> {
        let mut tx = pool.begin().await?;
        let game = sqlx::query_as!(
            Game, 
            r#"UPDATE game SET id = $1, name = $2, kind = $3 WHERE id = $4 RETURNING id, name, kind as "kind: GameKind""#,
            game.id, game.name, game.kind as GameKind, game.id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(game)
    }

    pub async fn delete(game: Game, pool: &PgPool) -> Result<Game> {
        let mut tx = pool.begin().await?;
        let game = sqlx::query_as!(
            Game,
            r#"DELETE FROM game WHERE id = $1 RETURNING id, name, kind as "kind: GameKind""#,
            game.id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(game)
    }
}
