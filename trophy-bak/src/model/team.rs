use std::fmt;

use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use super::{Game, Outcome};

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(rename = "team_gender")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TeamGender {
    Female,
    Male,
}

// this impl is needed for setting the tab-name when creating the xlsx-file!
impl fmt::Display for TeamGender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub gender: TeamGender,
    pub points: i32
}

#[derive(Deserialize)]
pub struct CreateTeam {
    pub name: String,
    pub gender: TeamGender,
}

impl Responder for Team {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl Team {

    pub async fn find_all(pool: &PgPool) -> Result<Vec<Team>> {
        let teams = sqlx::query_as!(
            Team,
            r#"SELECT id, name, gender as "gender: TeamGender", points FROM team ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(teams)
    }

    pub async fn find_all_by_gender(pool: &PgPool) -> Result<(Vec<Team>, Vec<Team>)> {
        let teams = Team::find_all(pool).await?; 
        let mut female = Vec::<Team>::new();
        let mut male= Vec::<Team>::new();

        for team in teams {
            match team.gender {
                TeamGender::Female => female.push(team),
                TeamGender::Male => male.push(team),
            }
        }
        
        Ok((female, male))
    }

    pub async fn find(id: i32, pool: &PgPool) -> Result<Team> {
        let teams = sqlx::query_as!(
            Team,
            r#"SELECT id, name, gender as "gender: TeamGender", points FROM team WHERE id = $1"#, 
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(teams)
    }

    pub async fn create(create_team: CreateTeam, pool: &PgPool) -> Result<Team> {
        let mut tx = pool.begin().await?;
        let team: Team = sqlx::query_as!(
            Team, 
            r#"INSERT INTO team (name, gender) VALUES ($1, $2) RETURNING id, name, gender as "gender: TeamGender", points"#,
            create_team.name, create_team.gender as TeamGender
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        for game in Game::find_all(pool).await? {
            Outcome::create(game.id, team.id, pool).await?;
        }

        Ok(team)
    }

    pub async fn update(id: i32, altered_team: Team, pool: &PgPool) -> Result<Team> {
        let mut tx = pool.begin().await?;
        let team = sqlx::query_as!(
            Team, 
            r#"UPDATE team SET id = $1, name = $2, gender = $3 WHERE id = $4 RETURNING id, name, gender as "gender: TeamGender", points"#,
            altered_team.id, altered_team.name, altered_team.gender as TeamGender, id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(team)
    }

    pub async fn update_all(teams: Vec<Team>, pool: &PgPool)-> Result<()> {
        for team in teams {
            Team::update(team.id, team, pool).await?;
        }
        Ok(())
    }

    pub async fn delete(id: i32, pool: &PgPool) -> Result<Team> {
        let mut tx = pool.begin().await?;
        let teams = sqlx::query_as!(
            Team,
            r#"DELETE FROM team WHERE id = $1 RETURNING id, name, gender as "gender: TeamGender", points"#,
            id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(teams)
    }

    pub async fn pending_games(id: i32, pool: &PgPool) -> Result<Vec<Game>> {
        // outcome-list where no data is present
        let outcomes = Outcome::filter_for(Outcome::find_all_for_team, Option::<String>::is_none, id, pool).await?;
        let mut games: Vec<Game> = Vec::new();
        for game_id in outcomes.iter().map(|f| f.game_id) {
            games.push(Game::find(game_id, pool).await?);
        }
        Ok(games)
    }

    pub async fn finished_games(id: i32, pool: &PgPool) -> Result<Vec<Game>>{
        // outcome-list where data is set
        let outcomes= Outcome::filter_for(Outcome::find_all_for_team, Option::<String>::is_some, id, pool).await?;
        let mut games: Vec<Game> = Vec::new();
        for game_id in outcomes.iter().map(|f| f.game_id) {
            games.push(Game::find(game_id, pool).await?);
        }
        Ok(games)
    }

    pub async fn pending_games_amount(id: i32, pool: &PgPool) -> Result<usize> {
        // I am choosing to not use outstanding_teams as it encompasses loading all outstanding teams before counting.

        let outcomes = Outcome::find_all_for_team(id, pool).await?;

        // filter every outcome that has data, then count the items
        Ok(outcomes.iter().filter(|e | e.data.is_none()).count())
    }

    pub async fn amount(pool: &PgPool) -> Result<usize> {
        // This function currently calls find_all and uses its size.
        // If performance warrants a better implementation(f.e. caching the result in the db or memory), 
        // this capsules the functionality, meaning I will only need to change this method.
        
        Ok(Team::find_all(pool).await?.len())
    }

}