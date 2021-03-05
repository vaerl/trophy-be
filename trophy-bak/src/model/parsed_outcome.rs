use super::{DataBaseError, DbResult, GameKind, Outcome, Team};
use anyhow::Result;
use humantime::parse_duration;
use sqlx::PgPool;
use std::time::Duration;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Points(Duration),
    Time(i32),
}

pub struct ParsedOutcome {
    pub game_id: i32,
    pub team: Team,
    pub value: Value,
}

impl ParsedOutcome {
    pub async fn from(
        game_kind: &GameKind,
        outcome: Outcome,
        pool: &PgPool,
    ) -> Result<ParsedOutcome, DataBaseError> {
        let value: Value = match game_kind {
            super::GameKind::Points => Value::Time(outcome.data.unwrap().parse::<i32>()?),
            super::GameKind::Time => Value::Points(parse_duration(&outcome.data.unwrap())?),
        };
        Ok(ParsedOutcome {
            game_id: outcome.game_id,
            team: Team::find(outcome.team_id, pool).await?,
            value,
        })
    }
}
