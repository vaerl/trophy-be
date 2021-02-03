use super::{Outcome, Team};
use anyhow::Result;
use humantime::parse_duration;
use sqlx::PgPool;
use std::time::Duration;

pub struct ParsedOutcome<T> {
    pub game_id: i32,
    pub team: Team,
    pub value: T,
}

// TODO check performance
impl ParsedOutcome<Duration> {
    pub async fn from(outcome: Outcome, pool: &PgPool) -> Result<ParsedOutcome<Duration>> {
        Ok(ParsedOutcome::<Duration> {
            game_id: outcome.game_id,
            team: Team::find(outcome.team_id, pool).await?,
            value: parse_duration(&outcome.data.unwrap())?,
        })
    }
}

impl ParsedOutcome<i32> {
    pub async fn from(outcome: Outcome, pool: &PgPool) -> Result<ParsedOutcome<i32>> {
        Ok(ParsedOutcome::<i32> {
            game_id: outcome.game_id,
            team: Team::find(outcome.team_id, pool).await?,
            value: outcome.data.unwrap().parse::<i32>()?,
        })
    }
}
