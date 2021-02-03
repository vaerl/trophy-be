use crate::model::{Game, GameKind, Outcome, ParsedOutcome, Team, TeamGender};
use actix_files::NamedFile;
use anyhow::Result;
use sqlx::PgPool;
use std::{panic, time::Duration};

// this const allows for easy changing of max-points
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
        // don't evaluate when teams are still pending
        panic!(format!("Game {} is not finished!", id));
    } else {
        let outcomes = Outcome::find_all_for_game(id, pool).await?;
        let kind: GameKind = Game::find(id, pool).await?.kind;

        match kind {
            GameKind::Time => {
                let mut female_outcomes = Vec::<ParsedOutcome<Duration>>::new();
                let mut male_outcomes = Vec::<ParsedOutcome<Duration>>::new();

                // sort outcomes by gender
                for outcome in outcomes {
                    let parsed_outcome = ParsedOutcome::<Duration>::from(outcome, pool).await?;
                    match parsed_outcome.team.gender {
                        TeamGender::Female => female_outcomes.push(parsed_outcome),
                        TeamGender::Male => male_outcomes.push(parsed_outcome),
                    }
                }

                Team::update_all(female_outcomes.evaluate(), pool).await?;
                Team::update_all(male_outcomes.evaluate(), pool).await?;
            }
            GameKind::Points => {
                let mut female_outcomes = Vec::<ParsedOutcome<i32>>::new();
                let mut male_outcomes = Vec::<ParsedOutcome<i32>>::new();

                // sort outcomes by gender
                for outcome in outcomes {
                    let parsed_outcome = ParsedOutcome::<i32>::from(outcome, pool).await?;
                    match parsed_outcome.team.gender {
                        TeamGender::Female => female_outcomes.push(parsed_outcome),
                        TeamGender::Male => male_outcomes.push(parsed_outcome),
                    }
                }

                Team::update_all(female_outcomes.evaluate(), pool).await?;
                Team::update_all(male_outcomes.evaluate(), pool).await?;
            }
        };
        Ok(())
    }
}

trait Evaluate {
    fn evaluate(self) -> Vec<Team>;
}

// this implementation is restricted to types that implement the PartialEq- and Ord-Traits
impl<T: PartialEq + Ord> Evaluate for Vec<ParsedOutcome<T>> {
    fn evaluate(mut self) -> Vec<Team> {
        let mut current_points = MAX_POINTS;

        self.sort_by(|a, b| a.value.cmp(&b.value));
        for i in 0..self.len() {
            self[i].team.points += current_points;
            // decrement current_points if the next result is less
            if i + 1 < self.len() && self[i].value != self[i + 1].value {
                current_points -= 1;
            }
        }
        self.into_iter().map(|e| e.team).collect()
    }
}

pub async fn create_xlsx_file() -> Result<NamedFile> {
    todo!()
}
