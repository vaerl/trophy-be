use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::fmt::{self, Display};
use uuid::Uuid;

use super::{CustomError, Game, Outcome, TypeInfo};
use crate::{ApiResult, model::Amount};

#[derive(Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "team_gender")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TeamGender {
    #[serde(alias = "f")]
    #[serde(alias = "w")]
    Female,

    #[serde(alias = "m")]
    #[serde(alias = "g")]
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
    pub id: Uuid,
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

#[derive(Deserialize)]
pub struct ImportTeam {
    #[serde(alias = "trophy-id")]
    #[serde(alias = "Trophy-Id")]
    #[serde(alias = "Trophy-ID")]
    #[serde(alias = "ID")]
    #[serde(alias = "Id")]
    #[serde(alias = "Nr.")]
    #[serde(alias = "Nr")]
    #[serde(alias = "NR")]
    #[serde(alias = "NR.")]
    pub id: i32,

    #[serde(alias = "Name")]
    #[serde(alias = "Teamname")]
    pub name: String,

    #[serde(alias = "Typ")]
    #[serde(alias = "Geschlecht")]
    #[serde(alias = "Gender")]
    pub gender: TeamGender,
}

impl ImportTeam {
    pub fn with_year(self, year: i32) -> CreateTeam {
        CreateTeam {
            trophy_id: self.id,
            name: self.name,
            gender: self.gender,
            year,
        }
    }
}

impl Team {
    /// Find all [Team]s.
    pub async fn find_all(pool: &PgPool, year: i32) -> ApiResult<TeamVec> {
        let teams = sqlx::query_as!(
            Team,
            r#"SELECT id, trophy_id, name, gender as "gender: TeamGender", points, year FROM teams WHERE year = $1 ORDER BY id"#, year
        )
        .fetch_all(pool)
        .await?;

        Ok(TeamVec(teams))
    }

    /// Find all pending [Team]s.
    pub async fn find_all_pending(year: i32, pool: &PgPool) -> ApiResult<Amount> {
        let amount = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM (
                SELECT DISTINCT team_id FROM game_team
                    INNER JOIN games ON game_team.game_id=games.id
                    INNER JOIN teams ON game_team.team_id=teams.id
                WHERE data IS NULL AND teams.year = $1) AS temp"#,
            year
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        Ok(Amount(amount))
    }

    /// Find all pending [Team]s for the specified [Game].
    pub async fn find_all_pending_for_game(game_id: Uuid, pool: &PgPool) -> ApiResult<Amount> {
        let amount = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM (
                SELECT DISTINCT team_id FROM game_team
                    INNER JOIN games ON game_team.game_id=games.id
                    INNER JOIN teams ON game_team.team_id=teams.id
                WHERE data IS NULL AND game_id = $1) AS temp"#,
            game_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        Ok(Amount(amount))
    }

    /// Find all [Team]s split into a tuple by their gender.
    pub async fn find_all_by_gender(pool: &PgPool, year: i32) -> ApiResult<(TeamVec, TeamVec)> {
        let teams = Team::find_all(pool, year).await?.0;
        let mut female = Vec::<Team>::new();
        let mut male = Vec::<Team>::new();

        for team in teams {
            match team.gender {
                TeamGender::Female => female.push(team),
                TeamGender::Male => male.push(team),
            }
        }

        Ok((TeamVec(female), TeamVec(male)))
    }

    /// Try to get the [Team] of the specified ID.
    pub async fn find(id: Uuid, pool: &PgPool) -> ApiResult<Team> {
        let team = sqlx::query_as!(
            Team,
            r#"SELECT id, trophy_id, name, gender as "gender: TeamGender", points, year FROM teams WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await?;

        team.ok_or(CustomError::NotFoundError {
            message: format!("Team {} could not be found.", id),
        })
    }

    /// Create a new [Team].
    pub async fn create(create_team: CreateTeam, pool: &PgPool) -> ApiResult<Team> {
        let mut tx = pool.begin().await?;
        let team: Team = sqlx::query_as!(
            Team,
            r#"INSERT INTO teams (id, trophy_id, name, gender, year)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, trophy_id, name, gender as "gender: TeamGender", points, year"#,
            Uuid::now_v7(),
            create_team.trophy_id,
            create_team.name,
            create_team.gender as TeamGender,
            create_team.year
        )
        .fetch_one(&mut *tx)
        .await?;

        for game in Game::find_all(pool, create_team.year).await?.0 {
            Outcome::create(game.id, team.id, &mut *tx).await?;
        }

        tx.commit().await?;
        Ok(team)
    }

    /// Update the specified [Team]. Does not set points.
    pub async fn update(id: Uuid, altered_team: CreateTeam, pool: &PgPool) -> ApiResult<Team> {
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

    /// Write the points of this [Team] to the database.
    pub async fn update_points(&self, pool: &PgPool) -> ApiResult<Team> {
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

    /// Delete the specified [Team].
    pub async fn delete(id: Uuid, pool: &PgPool) -> ApiResult<Team> {
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
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Team(id: {}, trophy_id: {}, name: {}, gender: {}, points: {})",
            self.id, self.trophy_id, self.name, self.gender, self.points
        )
    }
}

impl TypeInfo for Team {
    fn type_name(&self) -> String {
        "Team".to_string()
    }
}

impl Display for TeamVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TeamVec[{}]",
            self.0.iter().map(|g| g.to_string()).collect::<String>()
        )
    }
}

impl TypeInfo for TeamVec {
    fn type_name(&self) -> String {
        "TeamVec".to_string()
    }
}
