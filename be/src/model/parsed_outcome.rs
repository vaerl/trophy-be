use super::{GameKind, Outcome, Team};
use humantime::parse_duration;
use sqlx::PgPool;
use std::time::Duration;

use crate::ApiResult;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Time(Duration),
    Points(i32),
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
    ) -> ApiResult<ParsedOutcome> {
        let value: Value = match game_kind {
            super::GameKind::Points => Value::Points(outcome.data.unwrap().parse::<i32>()?),
            super::GameKind::Time => Value::Time(parse_duration(&outcome.data.unwrap())?),
        };
        Ok(ParsedOutcome {
            game_id: outcome.game_id,
            team: Team::find(outcome.team_id, pool).await?,
            value,
        })
    }
}
