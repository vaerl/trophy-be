use std::fmt::{self, Display};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::ApiResult;
use super::{Amount, CustomError, Game, GameVec, Outcome, TypeInfo};

#[derive(Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "team_gender")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TeamGender {
    Female,
    Male,
}

impl fmt::Display for TeamGender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TeamGender::Female => write!(f, "Female"),
            TeamGender::Male => write!(f, "Male"),
        }
    }
}

#[derive(Serialize, FromRow, Clone)]
pub struct Team {
    pub id: i32,
    pub trophy_id: i32,
    pub name: String,
    pub gender: TeamGender,
    pub points: i32,
    pub year: i32,
}

#[derive(Serialize)]
pub struct TeamVec(pub Vec<Team>);

#[derive(Deserialize)]
pub struct CreateTeam {
    pub trophy_id: i32,
    pub name: String,
    pub gender: TeamGender,
    pub year: i32,
}

impl Team {

    pub async fn find_all(pool: &PgPool, year: i32) -> ApiResult<TeamVec> {
        let teams = sqlx::query_as!(
            Team,
            r#"SELECT id, trophy_id, name, gender as "gender: TeamGender", points, year FROM teams WHERE year = $1 ORDER BY id"#, year
        )
        .fetch_all(pool)
        .await?;

        Ok(TeamVec(teams))
    }

    pub async fn find_all_by_gender(pool: &PgPool, year: i32) -> ApiResult<(TeamVec, TeamVec)> {
        let teams = Team::find_all(pool, year).await?.0; 
        let mut female = Vec::<Team>::new();
        let mut male= Vec::<Team>::new();

        for team in teams {
            match team.gender {
                TeamGender::Female => female.push(team),
                TeamGender::Male => male.push(team),
            }
        }
        
        Ok((TeamVec(female), TeamVec(male)))
    }

    pub async fn find(id: i32, pool: &PgPool) -> ApiResult<Team> {
        let team = sqlx::query_as!(
            Team,
            r#"SELECT id, trophy_id, name, gender as "gender: TeamGender", points, year FROM teams WHERE id = $1"#, 
            id
        )
        .fetch_optional(pool)
        .await?;

        team.ok_or(CustomError::NotFoundError { message: format!("Team {} could not be found.", id) })
    }

    pub async fn create(create_team: CreateTeam, pool: &PgPool) -> ApiResult<Team> {
        let mut tx = pool.begin().await?;
        let team: Team = sqlx::query_as!(
            Team, 
            r#"INSERT INTO teams (trophy_id, name, gender, year) VALUES ($1, $2, $3, $4) RETURNING id, trophy_id, name, gender as "gender: TeamGender", points, year"#,
            create_team.trophy_id, create_team.name, create_team.gender as TeamGender, create_team.year
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        for game in Game::find_all(pool, create_team.year).await?.0 {
            Outcome::create(game.id, game.trophy_id, team.id, team.trophy_id, pool).await?;
        }

        Ok(team)
    }

    pub async fn update(id: i32, altered_team: CreateTeam, pool: &PgPool) -> ApiResult<Team> {
        // NOTE I've decided against being able to change the year of already created teams (for now)
        let mut tx = pool.begin().await?;
        let team = sqlx::query_as!(
            Team, 
            r#"UPDATE teams SET trophy_id = $1, name = $2, gender = $3 WHERE id = $4 RETURNING id, trophy_id, name, gender as "gender: TeamGender", points, year"#,
            altered_team.trophy_id, altered_team.name, altered_team.gender as TeamGender, id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(team)
    }

    pub async fn update_points(&self, pool: &PgPool)-> ApiResult<Team> {
        let mut tx = pool.begin().await?;
        let team = sqlx::query_as!(
            Team, 
            r#"UPDATE teams SET points = $1 WHERE id = $2 RETURNING id, trophy_id, name, gender as "gender: TeamGender", points, year"#,
            self.points, self.id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(team)
    }

    pub async fn delete(id: i32, pool: &PgPool) -> ApiResult<Team> {
        let mut tx = pool.begin().await?;
        let team = sqlx::query_as!(
            Team,
            r#"DELETE FROM teams WHERE id = $1 RETURNING id, trophy_id, name, gender as "gender: TeamGender", points, year"#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(team)
    }

    pub async fn pending_games(id: i32, pool: &PgPool) -> ApiResult<GameVec> {
        // outcome-list where no data is present
        let outcomes = Outcome::filter_for(Outcome::find_all_for_team, Option::<String>::is_none, id, pool).await?.0;
        let mut games: Vec<Game> = Vec::new();
        for game_id in outcomes.iter().map(|f| f.game_id) {
            games.push(Game::find(game_id, pool).await?);
        }
        Ok(GameVec(games))
    }

    pub async fn finished_games(id: i32, pool: &PgPool) -> ApiResult<GameVec>{
        // outcome-list where data is set
        let outcomes= Outcome::filter_for(Outcome::find_all_for_team, Option::<String>::is_some, id, pool).await?.0;
        let mut games: Vec<Game> = Vec::new();
        for game_id in outcomes.iter().map(|f| f.game_id) {
            games.push(Game::find(game_id, pool).await?);
        }
        Ok(GameVec(games))
    }

    pub async fn amount(pool: &PgPool, year: i32) -> ApiResult<Amount> {
        // This function currently calls find_all and uses its size.
        // If performance warrants a better implementation(f.e. caching the result in the db or memory), 
        // this capsules the functionality, meaning I will only need to change this method.
        Ok(Amount(Team::find_all(pool, year).await?.0.len()))
    }

    pub async fn pending(pool: &PgPool, year: i32) -> ApiResult<TeamVec> {
        let mut teams= vec!();
        for team in Team::find_all(pool, year).await?.0 {
            // I could also use pending_games_amount, but that could be removed later
            let pending_games_of_team = Team::pending_games(team.id, pool).await?.0;
            if pending_games_of_team.len() > 0 {
                teams.push(team)
            }
        }
        Ok(TeamVec(teams))
    }

    pub async fn finished(pool: &PgPool, year: i32) -> ApiResult<TeamVec> {
        let mut teams= vec!();
        for team in Team::find_all(pool, year).await?.0 {
            let pending_games_of_team = Team::pending_games(team.id, pool).await?.0;
            // if there are no pending games, the team must be finished
            if pending_games_of_team.len() == 0 {
                teams.push(team)
            }
        }
        Ok(TeamVec(teams))
    }

}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Team(id: {}, trophy_id: {}, name: {}, gender: {}, points: {})",self.id, self.trophy_id, self.name, self.gender, self.points)
    }
}

impl TypeInfo for Team {
    fn type_name(&self) -> String {
       format!("Team")
    }
}

impl Display for TeamVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TeamVec[{}]", self.0.iter().map(|g| g.to_string()).collect::<String>())
    }
}

impl TypeInfo for TeamVec {
    fn type_name(&self) -> String {
       format!("TeamVec")
    }
}