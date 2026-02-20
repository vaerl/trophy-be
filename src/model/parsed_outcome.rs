use super::{GameKind, Outcome, Team};
use humantime::parse_duration;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

use crate::ApiResult;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Value {
    Time(Duration),
    Points(i32),
}

pub struct ParsedOutcome {
    pub game_id: Uuid,
    pub team: Team,
    pub value: Value,
    pub point_value: Option<i32>,
}

impl ParsedOutcome {
    pub async fn from(game_kind: &GameKind, outcome: Outcome, pool: &PgPool) -> ApiResult<Self> {
        let value: Value = match game_kind {
            super::GameKind::Points => Value::Points(outcome.data.unwrap().parse::<i32>()?),
            super::GameKind::Time => {
                let data = outcome.data.unwrap();
                let items: Vec<&str> = data.split(":").collect();
                assert!(
                    items.len() == 2,
                    "Found more than two parts when splitting a time!"
                );
                Value::Time(parse_duration(
                    format!("{}min {}s", items[0], items[1]).as_str(),
                )?)
            }
        };
        Ok(ParsedOutcome {
            game_id: outcome.game_id,
            team: Team::find(outcome.team_id, pool).await?,
            value,
            point_value: None,
        })
    }
}
