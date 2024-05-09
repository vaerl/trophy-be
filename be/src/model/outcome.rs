use std::fmt::{self, Display};
use actix::Addr;
use futures::Future;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use crate::{ApiResult, model::CustomError, ws::{lobby::Lobby, socket_refresh::SendRefresh}};
use super::{Game, GameKind, ParsedOutcome, TeamGender, TypeInfo};

/// This module provides all routes concerning outcomes.
/// As the name "Result" was already taken for the programming-structure, I'm using "outcome".
/// NOTE that sqlx wants results of joins to be Optional<T> - which is the case for `game_name` and `team_name` here.
#[derive(Deserialize, Serialize, FromRow)]
#[sqlx(type_name = "game_team")]
#[sqlx(rename_all = "lowercase")]
pub struct Outcome {
    pub game_id: i32,
    pub game_trophy_id: i32,
    pub game_name: String,
    pub game_kind: GameKind,
    pub team_id: i32,
    pub team_trophy_id: i32,
    pub team_name: String,
    pub data: Option<String>,
    pub point_value: Option<i32>
}
#[derive(Serialize)]
pub struct OutcomeVec(pub Vec<Outcome>);

impl Outcome {
    pub async fn find_all(pool: &PgPool) -> ApiResult<OutcomeVec> {
        let outcomes = sqlx::query_as!(
            Outcome,
            r#"SELECT game_id, game_trophy_id, games.name as game_name, games.kind as "game_kind: GameKind", team_id, team_trophy_id, teams.name as team_name, data, point_value FROM game_team
                INNER JOIN games ON game_team.game_id=games.id
                INNER JOIN teams ON game_team.team_id=teams.id
            ORDER BY game_id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(OutcomeVec(outcomes))
    }

    pub async fn find_all_for_game(game_id: i32, pool: &PgPool) -> ApiResult<OutcomeVec> {
        let outcomes = sqlx::query_as!(
            Outcome,
            r#"SELECT game_id, game_trophy_id, games.name as game_name, games.kind as "game_kind: GameKind", team_id, team_trophy_id, teams.name as team_name, data, point_value FROM game_team
                INNER JOIN games ON game_team.game_id=games.id
                INNER JOIN teams ON game_team.team_id=teams.id
            WHERE game_id = $1 ORDER BY game_id"#,
            game_id
        )
        .fetch_all(pool)
        .await?;

        Ok(OutcomeVec(outcomes))
    }

    pub async fn find_all_for_team(team_id: i32, pool: &PgPool) -> ApiResult<OutcomeVec> {
        let outcomes = sqlx::query_as!(
            Outcome,
            r#"SELECT game_id, game_trophy_id, games.name as game_name, games.kind as "game_kind: GameKind", team_id, team_trophy_id, teams.name as team_name, data, point_value FROM game_team
                INNER JOIN games ON game_team.game_id=games.id
                INNER JOIN teams ON game_team.team_id=teams.id
            WHERE team_id = $1 ORDER BY game_id"#,
            team_id
        )
        .fetch_all(pool)
        .await?;

        Ok(OutcomeVec(outcomes))
    }

    pub async fn create(game_id: i32, game_trophy_id: i32, team_id: i32, team_trophy_id: i32, pool: &PgPool) -> ApiResult<Outcome> {
        // there is no need to check if the ids are valid here - because this is called while iterating over existing entities 
        // NOTE all needed entities are created, because if a Team or Game is created,
        // the missing outcomes are created, but not any more! 
        let mut tx = pool.begin().await?;
        let outcome = sqlx::query_as!(
            Outcome, 
            r#"WITH inserted AS (INSERT INTO game_team (game_id, game_trophy_id, team_id, team_trophy_id) VALUES ($1, $2, $3, $4) RETURNING *)
            SELECT game_id, game_trophy_id, games.name as game_name, games.kind as "game_kind: GameKind", team_id, team_trophy_id, teams.name as team_name, data, point_value FROM inserted
                INNER JOIN games ON inserted.game_id=games.id
                INNER JOIN teams ON inserted.team_id=teams.id"#, 
            game_id, game_trophy_id, team_id, team_trophy_id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(outcome)
    }

    /// This method needs the calling user as it might modify a game's state.
    pub async fn set_data(&self, lobby: &Addr<Lobby>, pool: &PgPool) -> ApiResult<Outcome> {
        match &self.data {
            Some(data) => {
                let game = Game::find(self.game_id, &pool).await?;
                
                // update the outcome, so we find it later
                let mut tx = pool.begin().await?;
                let outcome = sqlx::query_as!(
                        Outcome, 
                        r#"WITH updated AS (UPDATE game_team SET data = $1 WHERE game_id = $2 AND team_id = $3 RETURNING *)
                        SELECT game_id, game_trophy_id, games.name as game_name, games.kind as "game_kind: GameKind", team_id, team_trophy_id, teams.name as team_name, data, point_value FROM updated
                            INNER JOIN games ON updated.game_id=games.id
                            INNER JOIN teams ON updated.team_id=teams.id"#,
                        data, self.game_id, self.team_id
                    )
                    .fetch_one(&mut *tx)
                    .await?;
                tx.commit().await?;
                
                let outcomes = Outcome::find_all_for_game(outcome.game_id, &pool).await?;

                // lock the game if there are no unset outcomes
                if outcomes.0.into_iter().filter(|o| o.data.is_none()).collect::<Vec<Outcome>>().len() == 0 {
                    Game::lock(game.id, pool).await?
                    .send_refresh(lobby)?;
                }
                
                Ok(outcome)
            },
            None => Err(CustomError::NoDataSentError { message: format!("Outcome had no data!") }),
        }
    }

    pub async fn set_point_value(parsed_outcome: ParsedOutcome, pool: &PgPool) -> ApiResult<Outcome> {
        let mut tx = pool.begin().await?;
        let outcome = sqlx::query_as!(
                Outcome, 
                r#"WITH updated AS (UPDATE game_team SET point_value = $1 WHERE game_id = $2 AND team_id = $3 RETURNING *)
                SELECT game_id, game_trophy_id, games.name as game_name, games.kind as "game_kind: GameKind", team_id, team_trophy_id, teams.name as team_name, data, point_value FROM updated
                            INNER JOIN games ON updated.game_id=games.id
                            INNER JOIN teams ON updated.team_id=teams.id"#,
                parsed_outcome.point_value, parsed_outcome.game_id, parsed_outcome.team.id
            )
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;

        Ok(outcome)
    }

    pub async fn filter_for<'r, Fut>(
        find_for_all: impl Fn(i32, &'r PgPool) -> Fut,
        filter: impl Fn(& Option<String>) -> bool,
        id: i32, 
        pool: &'r PgPool
    ) -> ApiResult<OutcomeVec>
    where Fut: Future<Output = ApiResult<OutcomeVec>> // won't work without where
    {
        // find every outcome using the supplied function
        let outcomes = find_for_all(id, pool).await?.0;
        // remove every item that does not evaluate to true with filter
        Ok(OutcomeVec(outcomes.into_iter().filter(|f| filter(&f.data)).collect()))
    }

    /// Parse all outcomes for game and return as ParsedOutcome.
    pub async fn parse_by_gender_for_game(game: &Game, pool: &PgPool) -> ApiResult<(Vec::<ParsedOutcome>, Vec::<ParsedOutcome>)> {
        let mut female_outcomes = Vec::<ParsedOutcome>::new();
        let mut male_outcomes = Vec::<ParsedOutcome>::new();
        // sort outcomes by gender
        for outcome in Outcome::find_all_for_game(game.id, pool).await?.0 {
            let parsed_outcome = ParsedOutcome::from(&game.kind, outcome, pool).await?;
            match parsed_outcome.team.gender {
                TeamGender::Female => female_outcomes.push(parsed_outcome),
                TeamGender::Male => male_outcomes.push(parsed_outcome),
            }
        }

        Ok((female_outcomes, male_outcomes))
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Outcome(game_id: {}, team_id: {}, data: {:?})", self.game_id, self.team_id, self.data)
    }
}

impl TypeInfo for Outcome {
    fn type_name(&self) -> String {
       format!("Outcome")
    }
}

impl Display for OutcomeVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OutcomeVec[{}]", self.0.iter().map(|g| g.to_string()).collect::<String>())
    }
}

impl TypeInfo for OutcomeVec {
    fn type_name(&self) -> String {
       format!("OutcomeVec")
    }
}