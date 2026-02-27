use super::{GameKind, Team};
use humantime::parse_duration;
use std::time::Duration;
use uuid::Uuid;

use crate::ApiResult;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Value {
    Seconds(Duration),
    Points(i32),
}

pub struct ParsedOutcome {
    pub game_id: Uuid,
    pub team: Team,
    pub value: Value,
    pub point_value: Option<i32>,
}

impl ParsedOutcome {
    pub fn from(data: String, game_kind: &GameKind, game_id: Uuid, team: Team) -> ApiResult<Self> {
        let value: Value = match game_kind {
            super::GameKind::Points => Value::Points(data.parse::<i32>()?),
            super::GameKind::Time => Value::Seconds(parse_duration(format!("{}s", data).as_str())?),
        };

        Ok(ParsedOutcome {
            game_id,
            team,
            value,
            point_value: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ParsedOutcome;
    use crate::model::{Team, TeamGender};
    use chrono::{Datelike, Local};
    use std::time::Duration;
    use uuid::Uuid;

    fn get_team() -> Team {
        Team {
            id: Uuid::now_v7(),
            trophy_id: 1,
            name: "Team".to_string(),
            gender: TeamGender::Female,
            points: 0,
            year: Local::now().year(),
        }
    }

    #[test]
    fn parse_points_positive() {
        let data = "10".to_string();
        let expected = super::Value::Points(10);
        let actual =
            ParsedOutcome::from(data, &super::GameKind::Points, Uuid::now_v7(), get_team())
                .unwrap();

        assert_eq!(expected, actual.value);
    }

    #[test]
    fn parse_points_negative() {
        let data = "-10".to_string();
        let expected = super::Value::Points(-10);
        let actual =
            ParsedOutcome::from(data, &super::GameKind::Points, Uuid::now_v7(), get_team())
                .unwrap();

        assert_eq!(expected, actual.value);
    }

    #[test]
    fn parse_seconds() {
        let data = "90".to_string();
        let expected = super::Value::Seconds(Duration::from_secs(90));
        let actual =
            ParsedOutcome::from(data, &super::GameKind::Time, Uuid::now_v7(), get_team()).unwrap();

        assert_eq!(expected, actual.value);
    }
}
