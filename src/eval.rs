use crate::{
    model::{CustomError, Game, GenderOutcomes, Outcome, ParsedOutcome, Team, Value},
    ApiResult, TypeInfo,
};
use actix_files::NamedFile;
use sqlx::PgPool;
use std::{
    fmt::{self, Display},
    fs::File,
    time::SystemTime,
};
use xlsxwriter::*;

// TODO add all teams to the result-list
// TODO check algorithm again - males was pretty good, females did not match
// -> discrepancies are to be expected, because I deleted two teams completely

pub struct ResultFile(pub NamedFile);

const MAX_POINTS: i32 = 50;

/// Checks whether all games have finished.
pub async fn is_done(pool: &PgPool, year: i32) -> ApiResult<bool> {
    let pending_games = Outcome::find_all_pending_games(year, pool).await?.0;
    Ok(pending_games == 0)
}

/// Checks whether all teams have points assigned.
pub async fn is_evaluated(pool: &PgPool, year: i32) -> ApiResult<bool> {
    let teams = Team::find_all(pool, year).await?.0;
    for team in teams {
        if team.points == 0 {
            return Ok(false);
        }
    }

    Ok(true)
}

pub async fn evaluate_trophy(pool: &PgPool, year: i32) -> ApiResult<()> {
    if !is_done(pool, year).await? {
        return Err(CustomError::EarlyEvaluationError {
            message: "Tried to evaluate while teams are still playing!".to_string(),
        });
    }

    if is_evaluated(pool, year).await? {
        return Err(CustomError::EarlyEvaluationError {
            message: "Already evaluated.".to_string(),
        });
    }

    // I cannot use locked here, as locked might be changed arbitrarily by admins(me)
    for game in Game::find_all(pool, year).await?.0 {
        evaluate_game(game, pool).await?;
    }
    Ok(())
}

async fn evaluate_game(game: Game, pool: &PgPool) -> ApiResult<()> {
    // TODO shouldn't this only check if game is done?
    if !is_done(pool, game.year).await? {
        return Err(CustomError::EarlyEvaluationError {
            message: "Tried to evaluate while teams are still playing!".to_string(),
        });
    }

    // get all Outcomes as ParsedOutcomes for game separated by gender
    let GenderOutcomes {
        male_outcomes,
        female_outcomes,
    } = Outcome::parse_by_gender_for_game(&game, pool).await?;

    // persist all changes from evaluate()
    // -> update_points and set_point_value write the current values of team and outcome (that have been assigned by evaluate) to the database
    for outcome in evaluate(female_outcomes) {
        outcome.team.update_points(pool).await?;
        Outcome::set_point_value(outcome, pool).await?;
    }

    for outcome in evaluate(male_outcomes) {
        outcome.team.update_points(pool).await?;
        Outcome::set_point_value(outcome, pool).await?;
    }

    Ok(())
}

/// Evaluate a [Game] by its [ParsedOutcome]s.
fn evaluate(mut outcomes: Vec<ParsedOutcome>) -> Vec<ParsedOutcome> {
    if outcomes.is_empty() {
        return outcomes;
    }

    // NOTE with this algorithm, we might assign negative points if there's ever more than 50 teams of a single gender.
    // This could be circumvented by having MAX_POINTS be the number of teams - or simply not decreasing current_points below 1.
    // Reaching this scenario is unlikely since the trophy is currently capped at 50 to 60 teams.
    let mut current_points = MAX_POINTS;
    let mut current_gap = 1;

    match outcomes[0].value {
        Value::Time(_) => outcomes.sort_by(|a, b| a.value.cmp(&b.value)),
        // we have to reverse this vec so the shortest time has the first position
        Value::Points(_) => outcomes.sort_by(|a, b| a.value.cmp(&b.value).reverse()),
    }

    // NOTE I've decided against using iter() and map() - this was causing more hassle than good here.
    for i in 0..outcomes.len() {
        // set the team's points for later usage
        outcomes[i].team.points += current_points;
        outcomes[i].point_value = Some(current_points);

        // reduce current_points if the next result is less
        // NOTE we have to use != here because the next value may be smaller or bigger, depending on whether value is a time or points
        if i + 1 < outcomes.len() && outcomes[i].value != outcomes[i + 1].value {
            current_points -= current_gap;
            // if the gap was used, it has to be reset to one
            current_gap = 1;
        } else {
            // if the current and next values are equal, the gap has to be increased by one
            current_gap += 1;
        }
    }

    outcomes
}

pub async fn create_xlsx_file(pool: &PgPool, year: i32) -> ApiResult<ResultFile> {
    // this path uses a timestamp to distinguish between versions
    let path = format!(
        "results-{}.xlsx",
        humantime::format_rfc3339_seconds(SystemTime::now())
    );

    // create file
    File::create(&path)?;
    let workbook = Workbook::new(&path)?;
    let (female, male) = Team::find_all_by_gender(pool, year).await?;

    // only write teams if any exist
    if !female.0.is_empty() {
        write_teams(female.0, &workbook).await?;
    }
    if !male.0.is_empty() {
        write_teams(male.0, &workbook).await?;
    }
    workbook.close()?;

    // open and return file
    Ok(ResultFile(NamedFile::open(path)?))
}

async fn write_teams(mut teams: Vec<Team>, workbook: &Workbook) -> ApiResult<()> {
    // create fonts
    let mut heading = Format::new();
    heading.set_bold().set_font_size(20.0);

    let mut values = Format::new();
    values.set_font_size(12.0);

    // :create initial structure
    let mut sheet = workbook.add_worksheet(Some(&teams[0].gender.to_string()))?;
    sheet.write_string(0, 0, "Platz", Some(&heading))?;
    sheet.write_string(0, 1, "Team", Some(&heading))?;
    sheet.write_string(0, 2, "Punkte", Some(&heading))?;

    // sort teams by points for right order in xlsx-file
    teams.sort_by(|a, b| b.points.cmp(&a.points));
    let mut row = 1;
    for (id, team) in teams.iter().enumerate() {
        sheet.write_string(row, 0, &format!("{}", id + 1), Some(&values))?;
        sheet.write_string(row, 1, &team.name, Some(&values))?;
        sheet.write_string(row, 2, &team.points.to_string(), Some(&values))?;
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
        "ResultFile".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TeamGender;
    use std::time::Duration;

    fn get_teams() -> Vec<Team> {
        vec![
            Team {
                id: 1,
                trophy_id: 1,
                name: "A".to_string(),
                gender: TeamGender::Female,
                points: 0,
                year: 2024,
            },
            Team {
                id: 2,
                trophy_id: 2,
                name: "B".to_string(),
                gender: TeamGender::Female,
                points: 0,
                year: 2024,
            },
            Team {
                id: 3,
                trophy_id: 3,
                name: "C".to_string(),
                gender: TeamGender::Female,
                points: 0,
                year: 2024,
            },
            Team {
                id: 4,
                trophy_id: 4,
                name: "D".to_string(),
                gender: TeamGender::Female,
                points: 0,
                year: 2024,
            },
            Team {
                id: 5,
                trophy_id: 5,
                name: "E".to_string(),
                gender: TeamGender::Female,
                points: 0,
                year: 2024,
            },
        ]
    }

    fn get_outcomes(teams: Vec<Team>, values: Vec<Value>) -> Vec<ParsedOutcome> {
        assert!(teams.len() == values.len());
        let mut parsed_outcomes = Vec::<ParsedOutcome>::new();

        for (team, value) in teams.iter().zip(values) {
            parsed_outcomes.push(ParsedOutcome {
                game_id: 1,
                team: team.clone(),
                value: value.clone(),
                point_value: None,
            });
        }

        parsed_outcomes
    }

    /// Checks [evaluate] with point-values without a gap.
    #[test]
    fn evaluate_points_simple() {
        let teams = get_teams();
        let parsed_outcomes = get_outcomes(
            teams,
            vec![
                Value::Points(1),
                Value::Points(10),
                Value::Points(100),
                Value::Points(5),
                Value::Points(1000),
            ],
        );

        let teams: Vec<Team> = evaluate(parsed_outcomes)
            .into_iter()
            .map(|e| e.team)
            .collect();
        teams.iter().for_each(|f| println!("{}", f));

        assert!(
            teams[0].points == MAX_POINTS,
            "Team 5 is has the correct number of points: {}",
            teams[0].points
        );
        assert!(
            teams[1].points == MAX_POINTS - 1,
            "Team 4 is has the correct number of points: {}",
            teams[1].points
        );
        assert!(
            teams[2].points == MAX_POINTS - 2,
            "Team 3 is has the correct number of points: {}",
            teams[2].points
        );
        assert!(
            teams[3].points == MAX_POINTS - 3,
            "Team 2 is has the correct number of points: {}",
            teams[3].points
        );
        assert!(
            teams[4].points == MAX_POINTS - 4,
            "Team 1 is has the correct number of points: {}",
            teams[4].points
        );
    }

    /// Checks [evaluate] with points and a more complex scenario.
    #[test]
    fn evaluate_points_gap() {
        let teams = get_teams();
        let parsed_outcomes = get_outcomes(
            teams,
            vec![
                Value::Points(100),
                Value::Points(100),
                Value::Points(100),
                Value::Points(5),
                Value::Points(1000),
            ],
        );

        let teams: Vec<Team> = evaluate(parsed_outcomes)
            .into_iter()
            .map(|e| e.team)
            .collect();
        teams.iter().for_each(|f| println!("{}", f));

        assert!(
            teams[0].points == MAX_POINTS,
            "Team 5 is has the correct number of points: {}",
            teams[0].points
        );
        assert!(
            teams[1].points == MAX_POINTS - 1,
            "Team 4 is has the correct number of points: {}",
            teams[1].points
        );
        assert!(
            teams[2].points == MAX_POINTS - 1,
            "Team 3 is has the correct number of points: {}",
            teams[2].points
        );
        assert!(
            teams[3].points == MAX_POINTS - 1,
            "Team 4 is has the correct number of points: {}",
            teams[3].points
        );

        assert!(
            teams[4].points == MAX_POINTS - 4,
            "Team 1 is has the correct number of points: {}",
            teams[4].points
        );
    }

    /// Checks [evaluate] with points and a more complex scenario.
    #[test]
    fn evaluate_time_simple() {
        let teams = get_teams();
        let parsed_outcomes = get_outcomes(
            teams,
            vec![
                Value::Time(Duration::new(120, 0)),
                Value::Time(Duration::new(60, 0)),
                Value::Time(Duration::new(40, 0)),
                Value::Time(Duration::new(80, 0)),
                Value::Time(Duration::new(10, 0)),
            ],
        );

        let teams: Vec<Team> = evaluate(parsed_outcomes)
            .into_iter()
            .map(|e| e.team)
            .collect();
        teams.iter().for_each(|f| println!("{}", f));

        assert!(
            teams[0].points == MAX_POINTS,
            "Team 5 is has the correct number of points: {}",
            teams[0].points
        );
        assert!(
            teams[1].points == MAX_POINTS - 1,
            "Team 4 is has the correct number of points: {}",
            teams[1].points
        );
        assert!(
            teams[2].points == MAX_POINTS - 2,
            "Team 3 is has the correct number of points: {}",
            teams[2].points
        );
        assert!(
            teams[3].points == MAX_POINTS - 3,
            "Team 4 is has the correct number of points: {}",
            teams[3].points
        );

        assert!(
            teams[4].points == MAX_POINTS - 4,
            "Team 1 is has the correct number of points: {}",
            teams[4].points
        );
    }

    #[test]
    fn evaluate_time_gap() {
        let teams = get_teams();
        let parsed_outcomes = get_outcomes(
            teams,
            vec![
                Value::Time(Duration::new(80, 0)),
                Value::Time(Duration::new(80, 0)),
                Value::Time(Duration::new(800, 0)),
                Value::Time(Duration::new(80, 0)),
                Value::Time(Duration::new(10, 0)),
            ],
        );

        let teams: Vec<Team> = evaluate(parsed_outcomes)
            .into_iter()
            .map(|e| e.team)
            .collect();
        teams.iter().for_each(|f| println!("{}", f));

        assert!(
            teams[0].points == MAX_POINTS,
            "Team 5 is has the correct number of points: {}",
            teams[0].points
        );
        assert!(
            teams[1].points == MAX_POINTS - 1,
            "Team 4 is has the correct number of points: {}",
            teams[1].points
        );
        assert!(
            teams[2].points == MAX_POINTS - 1,
            "Team 3 is has the correct number of points: {}",
            teams[2].points
        );
        assert!(
            teams[3].points == MAX_POINTS - 1,
            "Team 4 is has the correct number of points: {}",
            teams[3].points
        );

        assert!(
            teams[4].points == MAX_POINTS - 4,
            "Team 1 is has the correct number of points: {}",
            teams[4].points
        );
    }
}
