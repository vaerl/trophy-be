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

#[derive(Deserialize)]
pub struct CreateGame {
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

    pub async fn find(id: i32, pool: &PgPool) -> Result<Game> {
        let game = sqlx::query_as!(
            Game,
            r#"SELECT id, name, kind as "kind: GameKind" FROM game WHERE id = $1"#, id
        )
        .fetch_one(pool)
        .await?;

        Ok(game)
    }

    pub async fn create(create_game: CreateGame, pool: &PgPool) -> Result<Game> {
        let mut tx = pool.begin().await?;
        let game: Game = sqlx::query_as!( Game, 
            r#"INSERT INTO game (name, kind) VALUES ($1, $2) RETURNING id, name, kind as "kind: GameKind""#,
            create_game.name, create_game.kind as GameKind
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        // create outcomes
        for team in Team::find_all(pool).await? {
            Outcome::create(game.id, team.id, pool).await?;
        }

        Ok(game)
    }

    pub async fn update(id: i32, altered_game: Game, pool: &PgPool) -> Result<Game> {
        let mut tx = pool.begin().await?;
        let game = sqlx::query_as!(
            Game, 
            r#"UPDATE game SET id = $1, name = $2, kind = $3 WHERE id = $4 RETURNING id, name, kind as "kind: GameKind""#,
            altered_game.id, altered_game.name, altered_game.kind as GameKind, id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(game)
    }

    pub async fn delete(id: i32, pool: &PgPool) -> Result<Game> {
        let mut tx = pool.begin().await?;
        let game = sqlx::query_as!(
            Game,
            r#"DELETE FROM game WHERE id = $1 RETURNING id, name, kind as "kind: GameKind""#,
            id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(game)
    }

    pub async fn pending_teams(id: i32, pool: &PgPool) -> Result<Vec<Team>> {
        // outcome-list where no data is present
        let outcomes = Outcome::filter_for(Outcome::find_all_for_game, Option::<String>::is_none, id, pool).await?;
        let mut teams: Vec<Team> = Vec::new();
        for team_id in outcomes.iter().map(|f| f.team_id) {
            teams.push(Team::find(team_id, pool).await?);
        }
        Ok(teams)
    }

    pub async fn finished_teams(id: i32, pool: &PgPool) -> Result<Vec<Team>>{
        // outcome-list where data is set
        let outcomes= Outcome::filter_for(Outcome::find_all_for_game, Option::<String>::is_some, id, pool).await?;
        let mut teams: Vec<Team> = Vec::new();
        for team_id in outcomes.iter().map(|f| f.team_id) {
            teams.push(Team::find(team_id, pool).await?);
        }
        Ok(teams)
    }

    pub async fn pending_teams_amount(id: i32, pool: &PgPool) -> Result<usize> {
        // I am choosing to not use pending_teams as it encompasses loading all outstanding teams before counting.

        let outcomes = Outcome::find_all_for_game(id, pool).await?;

        // filter every outcome that has data, then count the items
        Ok(outcomes.iter().filter(|e | e.data.is_none()).count())
    }

    pub async fn amount(pool: &PgPool) -> Result<usize> {
        // This function currently calls find_all and uses its size.
        // If performance warrants a better implementation(f.e. caching the result in the db or memory), 
        // this capsules the functionality, meaning I will only need to change this method.
        
        Ok(Game::find_all(pool).await?.len())
    }
}
