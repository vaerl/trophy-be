use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::{Future, future::{ready, Ready}};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use super::{CustomError, GameKind, ParsedOutcome, TeamGender};



#[derive(Deserialize, Serialize, FromRow)]
#[sqlx(rename = "game_team")]
#[sqlx(rename_all = "lowercase")]
pub struct Outcome {
    pub game_id: i32,
    pub team_id: i32,
    pub data: Option<String>,
}

impl Responder for Outcome {
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

impl Outcome {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Outcome>, CustomError> {
        let outcomes = sqlx::query_as!(
            Outcome,
            r#"SELECT game_id, team_id, data FROM game_team ORDER BY game_id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(outcomes)
    }

    pub async fn find_all_for_game(game_id: i32, pool: &PgPool) -> Result<Vec<Outcome>, CustomError> {
        let outcomes = sqlx::query_as!(
            Outcome,
            "SELECT game_id, team_id, data FROM game_team WHERE game_id = $1 ORDER BY game_id",
            game_id
        )
        .fetch_all(pool)
        .await?;

        Ok(outcomes)
    }

    pub async fn find_all_for_team(team_id: i32, pool: &PgPool) -> Result<Vec<Outcome>, CustomError> {
        let outcomes = sqlx::query_as!(
            Outcome,
            "SELECT game_id, team_id, data FROM game_team WHERE team_id = $1 ORDER BY game_id",
            team_id
        )
        .fetch_all(pool)
        .await?;

        Ok(outcomes)
    }

    pub async fn create(game_id: i32, team_id: i32, pool: &PgPool) -> Result<Outcome, CustomError> {
        // there is no need to check if the ids are valid here - because this is called while iterating over existing entities 
        let mut tx = pool.begin().await?;
        let outcome = sqlx::query_as!(
            Outcome, 
            "INSERT INTO game_team (game_id, team_id) VALUES ($1, $2) RETURNING game_id, team_id, data", 
            game_id, team_id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(outcome)
    }

    pub async fn update(outcome: Outcome, pool: &PgPool) -> Result<Outcome, CustomError> {
        let mut tx = pool.begin().await?;
        let outcome = sqlx::query_as!(
            Outcome, 
            "UPDATE game_team SET data = $1 WHERE game_id = $2 AND team_id = $3 RETURNING game_id, team_id, data",
            outcome.data, outcome.game_id, outcome.team_id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(outcome)
    }

    pub async fn filter_for<'r, Fut>(
        find_for_all: impl Fn(i32, &'r PgPool) -> Fut,
        filter: impl Fn(& Option<String>) -> bool,
        id: i32, 
        pool: &'r PgPool
    ) -> Result<Vec<Outcome>, CustomError>
    where Fut: Future<Output = Result<Vec<Outcome>, CustomError>> // won't work without
    {
        // find every outcome using the supplied function
        let outcomes = find_for_all(id, pool).await?;
        // remove every item that does not evaluate to true with filter
        Ok(outcomes.into_iter().filter(|f| filter(&f.data)).collect())
    }

    pub async fn parse_by_gender(game_kind: &GameKind, pool: &PgPool) -> Result<(Vec::<ParsedOutcome>, Vec::<ParsedOutcome>), CustomError> {
        let mut female_outcomes = Vec::<ParsedOutcome>::new();
        let mut male_outcomes = Vec::<ParsedOutcome>::new();
        // sort outcomes by gender
        for outcome in Outcome::find_all(pool).await? {
            let parsed_outcome = ParsedOutcome::from(&game_kind, outcome, pool).await?;
            match parsed_outcome.team.gender {
                TeamGender::Female => female_outcomes.push(parsed_outcome),
                TeamGender::Male => male_outcomes.push(parsed_outcome),
            }
        }

        Ok((female_outcomes, male_outcomes))
    }
}