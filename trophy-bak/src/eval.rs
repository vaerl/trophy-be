use crate::model::{Game, Outcome, ParsedOutcome, Team};
use actix_files::NamedFile;
use anyhow::Result;
use sqlx::PgPool;
use std::{panic, time::SystemTime};
use xlsxwriter::*;

const MAX_POINTS: i32 = 50;

pub async fn evaluate_trophy(pool: &PgPool) -> Result<()> {
    for game in Game::find_all(pool).await? {
        evaluate_game(game.id, pool).await?;
    }
    Ok(())
}

async fn evaluate_game(id: i32, pool: &PgPool) -> Result<()> {
    let pending_amount = Game::pending_teams_amount(id, pool).await?;
    if pending_amount > 0 {
        // Don't evaluate when teams are still playing - this should never happen.
        panic!(format!("Game {} is not finished!", id));
    } else {
        let game_kind = Game::find(id, pool).await?.kind;
        let (female, male) = Outcome::parse_by_gender(&game_kind, pool).await?;

        Team::update_all(evaluate_team(female).await?, pool).await?;
        Team::update_all(evaluate_team(male).await?, pool).await?;

        Ok(())
    }
}

async fn evaluate_team(mut team: Vec<ParsedOutcome>) -> Result<Vec<Team>> {
    let mut current_points = MAX_POINTS;

    team.sort_by(|a, b| a.value.cmp(&b.value));
    for i in 0..team.len() {
        team[i].team.points += current_points;
        // decrement current_points if the next result is less
        if i + 1 < team.len() && team[i].value != team[i + 1].value {
            current_points -= 1;
        }
    }
    Ok(team.into_iter().map(|e| e.team).collect())
}

pub async fn create_xlsx_file(pool: &PgPool) -> Result<NamedFile> {
    // this path uses a timestamp to distinguish between versions
    let path = "./static/results-".to_owned()
        + &humantime::format_rfc3339_seconds(SystemTime::now()).to_string()
        + &".xlsx".to_owned();

    // create file
    let workbook = Workbook::new(&path);
    let (female, male) = Team::find_all_by_gender(pool).await?;
    write_teams(female, &workbook).await?;
    write_teams(male, &workbook).await?;
    workbook.close()?;

    // open and return file
    Ok(NamedFile::open(path)?)
}

async fn write_teams(teams: Vec<Team>, workbook: &Workbook) -> Result<()> {
    // create fonts
    let heading = workbook.add_format().set_bold().set_font_size(20.0);
    let values = workbook.add_format().set_font_size(12.0);

    // create initial structure
    let mut sheet = workbook.add_worksheet(Some(&teams[0].gender.to_string()))?;
    sheet.write_string(0, 0, "Team", Some(&heading))?;
    sheet.write_string(0, 1, "Punkte", Some(&heading))?;

    let mut row = 1;
    for team in teams {
        sheet.write_string(row, 0, &team.name, Some(&values))?;
        sheet.write_string(row, 1, &team.points.to_string(), Some(&values))?;
        row += 1;
    }

    Ok(())
}
