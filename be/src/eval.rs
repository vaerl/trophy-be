use crate::{
    model::{CustomError, Game, Outcome, ParsedOutcome, Team, Value},
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
    if outcomes.len() == 0 {
        return outcomes;
    }

    let mut current_points = MAX_POINTS;
    let mut current_gap = 1;

    match outcomes[0].value {
        Value::Time(_) => outcomes.sort_by(|a, b| a.value.cmp(&b.value)),
        Value::Points(_) => outcomes.sort_by(|a, b| a.value.cmp(&b.value).reverse()),
    }

    for i in 0..outcomes.len() {
        // set the team's points for later usage
        outcomes[i].team.points += current_points;
        outcomes[i].point_value = Some(current_points);

        // decrement current_points if the next result is less
        // NOTE using != is fine, since outcomes are already sorted
        if i + 1 < outcomes.len() && outcomes[i].value != outcomes[i + 1].value {
            current_points -= current_gap;
            // if the gap was used, it has to be reset to one
            current_gap = 1;
        } else {
            // if the current and next values match, the gap has to be increased by one
            current_gap += 1;
        }
    }

    outcomes
}

pub async fn create_xlsx_file(pool: &PgPool) -> ApiResult<ResultFile> {
    // this path uses a timestamp to distinguish between versions
    let path = format!(
        "results-{}.xlsx",
        humantime::format_rfc3339_seconds(SystemTime::now())
    );

    // create file
    File::create(&path)?;
    let workbook = Workbook::new(&path)?;
    let (female, male) = Team::find_all_by_gender(pool).await?;

    // only write teams if any exist
    if female.0.len() > 0 {
        write_teams(female.0, &workbook).await?;
    }
    if male.0.len() > 0 {
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
    sheet.write_string(0, 0, "Team", Some(&heading))?;
    sheet.write_string(0, 1, "Punkte", Some(&heading))?;

    // sort teams by points for right order in xlsx-file
    teams.sort_by(|a, b| b.points.cmp(&a.points));
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

    fn get_teams(points: Vec<i32>) -> Vec<Team> {
        let mut teams = Vec::<Team>::new();
        for i in 0..points.len() {
            teams.push(Team {
                id: (i + 1) as i32,
                trophy_id: (i + 1) as i32,
                name: format!("A"),
                gender: TeamGender::Female,
                points: points[i],
            })
        }

        teams
    }

    fn get_outcomes(game_id: i32, teams: Vec<Team>, points: Vec<Value>) -> Vec<ParsedOutcome> {
        assert!(teams.len() == points.len());
        let mut parsed_outcomes = Vec::<ParsedOutcome>::new();

        for i in 0..teams.len() {
            parsed_outcomes.push(ParsedOutcome {
                game_id,
                team: teams[i].clone(),
                value: points[i].clone(),
                point_value: None,
            });
        }

        parsed_outcomes
    }

    /// This test checks evaluate with point-values.
    #[test]
    fn test_evaluate_points() {
        let teams = get_teams(vec![0, 0, 0, 0]);
        let parsed_outcomes = get_outcomes(
            1,
            teams,
            vec![
                Value::Points(1),
                Value::Points(10),
                Value::Points(100),
                Value::Points(5),
            ],
        );

        let mut teams: Vec<Team> = evaluate(parsed_outcomes)
            .into_iter()
            .map(|e| e.team)
            .collect();
        teams.sort_by(|a, b| a.points.cmp(&b.points).reverse());
        teams.iter().for_each(|t| println!("{}", t));

        assert!(
            teams[0].id == 3,
            "Team 3 is not the first after sorting: {}",
            teams[0].id
        );
        assert!(
            teams[0].points == MAX_POINTS,
            "Team 3 is has the right number of points: {}",
            teams[0].points
        );

        assert!(
            teams[3].points == MAX_POINTS - 3,
            "Team 4 is has the right number of points: {}",
            teams[3].points
        );
    }

    // TODO figure out why this does not produce the list we made
    // TODO maybe add more tests
    #[test]
    /// This test checks evaluate with time-values.
    fn test_evaluate_time() {
        let teams = get_teams(vec![80, 20, 40, 10]);
        let parsed_outcomes = get_outcomes(
            1,
            teams,
            vec![
                Value::Time(Duration::new(120, 0)),
                Value::Time(Duration::new(60, 0)),
                Value::Time(Duration::new(40, 0)),
                Value::Time(Duration::new(80, 0)),
            ],
        );

        let mut teams: Vec<Team> = evaluate(parsed_outcomes)
            .into_iter()
            .map(|e| e.team)
            .collect();
        teams.sort_by(|a, b| a.points.cmp(&b.points).reverse());
        teams.iter().for_each(|t| println!("{}", t));

        assert!(teams[0].id == 1, "A is the first after sorting.");
        assert!(
            teams[0].points == 80 + MAX_POINTS - 3,
            "A is has the right number of points: {}",
            teams[0].points
        );
    }
}
