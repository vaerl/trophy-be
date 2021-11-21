// TODO consider moving the tests to a different file/module because of the unresolved imports

use crate::{
    model::{CustomError, Game, Outcome, ParsedOutcome, Team, TypeInfo, Value},
    ApiResult,
};
use actix_files::NamedFile;
use sqlx::PgPool;
use std::{
    fmt::{self, Display},
    time::SystemTime,
};
use xlsxwriter::*;

pub struct ResultFile(pub NamedFile);

const MAX_POINTS: i32 = 50;

pub async fn evaluate_trophy(pool: &PgPool) -> ApiResult<()> {
    // I cannot use locked here, as locked might be changed arbitrarily by admins(me)
    for game in Game::find_all(pool).await?.0 {
        evaluate_game(game, pool).await?;
    }
    Ok(())
}

async fn evaluate_game(game: Game, pool: &PgPool) -> ApiResult<()> {
    let pending_amount = Game::pending_teams_amount(game.id, pool).await?.0;
    if pending_amount > 0 {
        // Don't evaluate when teams are still playing - this should never happen!
        Err(CustomError::EarlyEvaluationError {
            message: "Tried to evaluate while teams are still playing!".to_string(),
        })
    } else {
        // get all Outcomes as ParsedOutcomes for game separated by gender
        let (female, male) = Outcome::parse_by_gender_for_game(&game, pool).await?;

        // using for-loops allows using await and ?
        for outcome in evaluate(female) {
            outcome.team.update_points(pool).await?;
            Outcome::set_point_value(outcome, pool).await?;
        }

        for outcome in evaluate(male) {
            outcome.team.update_points(pool).await?;
            Outcome::set_point_value(outcome, pool).await?;
        }

        Ok(())
    }
}

/// Evaluate a game by its ParsedOutcomes.
fn evaluate(mut outcomes: Vec<ParsedOutcome>) -> Vec<ParsedOutcome> {
    let mut current_points = MAX_POINTS;

    match outcomes[0].value {
        Value::Time(_) => outcomes.sort_by(|a, b| a.value.cmp(&b.value)),
        Value::Points(_) => outcomes.sort_by(|a, b| a.value.cmp(&b.value).reverse()),
    }

    for i in 0..outcomes.len() {
        // set the team's points for later usage
        outcomes[i].team.points += current_points;
        outcomes[i].point_value = Some(current_points);

        // decrement current_points if the next result is less
        if i + 1 < outcomes.len() && outcomes[i].value != outcomes[i + 1].value {
            current_points -= 1;
        }
    }

    outcomes
}

pub async fn create_xlsx_file(pool: &PgPool) -> ApiResult<ResultFile> {
    // this path uses a timestamp to distinguish between versions
    let path = "./static/results-".to_owned()
        + &humantime::format_rfc3339_seconds(SystemTime::now()).to_string()
        + &".xlsx".to_owned();

    // create file
    let workbook = Workbook::new(&path);
    let (female, male) = Team::find_all_by_gender(pool).await?;
    write_teams(female.0, &workbook).await?;
    write_teams(male.0, &workbook).await?;
    workbook.close()?;

    // open and return file
    Ok(ResultFile(NamedFile::open(path)?))
}

async fn write_teams(mut teams: Vec<Team>, workbook: &Workbook) -> ApiResult<()> {
    // create fonts
    let heading = workbook.add_format().set_bold().set_font_size(20.0);
    let values = workbook.add_format().set_font_size(12.0);

    // :create initial structure
    let mut sheet = workbook.add_worksheet(Some(&teams[0].gender.to_string()))?;
    sheet.write_string(0, 0, "Team", Some(&heading))?;
    sheet.write_string(0, 1, "Punkte", Some(&heading))?;

    // sort teams by points for right order in xlsx-file
    teams.sort_by(|a, b| a.points.cmp(&b.points));
    let mut row = 1;
    for team in teams {
        sheet.write_string(row, 0, &team.name, Some(&values))?;
        sheet.write_string(row, 1, &team.points.to_string(), Some(&values))?;
        row += 1;
    }

    Ok(())
}

impl Display for ResultFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // this is a slight hack, but I'm okay with that
        write!(f, "ResultFile({:#?})", self.0)
    }
}

impl TypeInfo for ResultFile {
    fn type_name(&self) -> String {
        format!("ResultFile")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TeamGender;
    use std::time::Duration;

    /// This test checks evaluate with point-values.
    #[test]
    fn test_evaluate_points() {
        let a = Team {
            id: 1,
            trophy_id: 1,
            name: format!("A"),
            gender: TeamGender::Female,
            points: 0,
        };
        let b = Team {
            id: 2,
            trophy_id: 2,
            name: format!("B"),
            gender: TeamGender::Female,
            points: 0,
        };
        let c = Team {
            id: 3,
            trophy_id: 1,
            name: format!("C"),
            gender: TeamGender::Female,
            points: 0,
        };
        let d = Team {
            id: 4,
            trophy_id: 1,
            name: format!("D"),
            gender: TeamGender::Female,
            points: 0,
        };

        let game_id = 1;

        let mut parsed_outcomes = Vec::<ParsedOutcome>::new();
        parsed_outcomes.push(ParsedOutcome {
            game_id,
            team: a,
            value: Value::Points(1),
            point_value: None,
        });
        parsed_outcomes.push(ParsedOutcome {
            game_id,
            team: b,
            value: Value::Points(10),
            point_value: None,
        });
        parsed_outcomes.push(ParsedOutcome {
            game_id,
            team: c,
            value: Value::Points(100),
            point_value: None,
        });
        parsed_outcomes.push(ParsedOutcome {
            game_id,
            team: d,
            value: Value::Points(5),
            point_value: None,
        });

        let mut teams: Vec<Team> = evaluate(parsed_outcomes)
            .into_iter()
            .map(|e| e.team)
            .collect();
        teams.sort_by(|a, b| a.points.cmp(&b.points).reverse());

        assert!(
            teams[0].name == format!("C"),
            "C is the first after sorting."
        );
        assert!(
            teams[0].points == MAX_POINTS,
            "C is has the right number of points: {}",
            teams[0].points
        );

        assert!(
            teams[3].points == MAX_POINTS - 3,
            "A is has the right number of points: {}",
            teams[3].points
        );

        // TODO
        // - do the same for more teams of both genders with existing points
    }

    #[test]
    /// This test checks evaluate with time-values.
    fn test_evaluate_time() {
        let a = Team {
            id: 1,
            trophy_id: 1,
            name: format!("A"),
            gender: TeamGender::Male,
            points: 80,
        };
        let b = Team {
            id: 2,
            trophy_id: 2,
            name: format!("B"),
            gender: TeamGender::Male,
            points: 20,
        };
        let c = Team {
            id: 3,
            trophy_id: 1,
            name: format!("C"),
            gender: TeamGender::Male,
            points: 40,
        };
        let d = Team {
            id: 4,
            trophy_id: 1,
            name: format!("D"),
            gender: TeamGender::Male,
            points: 10,
        };

        let game_id = 1;

        let mut parsed_outcomes = Vec::<ParsedOutcome>::new();
        parsed_outcomes.push(ParsedOutcome {
            game_id,
            team: a,
            value: Value::Time(Duration::new(120, 0)),
            point_value: None,
        });
        parsed_outcomes.push(ParsedOutcome {
            game_id,
            team: b,
            value: Value::Time(Duration::new(60, 0)),
            point_value: None,
        });
        parsed_outcomes.push(ParsedOutcome {
            game_id,
            team: c,
            value: Value::Time(Duration::new(40, 0)),
            point_value: None,
        });
        parsed_outcomes.push(ParsedOutcome {
            game_id,
            team: d,
            value: Value::Time(Duration::new(80, 0)),
            point_value: None,
        });

        let mut teams: Vec<Team> = evaluate(parsed_outcomes)
            .into_iter()
            .map(|e| e.team)
            .collect();
        teams.sort_by(|a, b| a.points.cmp(&b.points).reverse());

        println!("{}", teams[0].name);
        println!("{}", teams[0].points);
        println!("{}", teams[3].name);
        println!("{}", teams[3].points);

        assert!(
            teams[0].name == format!("A"),
            "A is the first after sorting."
        );
        assert!(
            teams[0].points == 80 + MAX_POINTS - 3,
            "A is has the right number of points: {}",
            teams[0].points
        );
    }
}
