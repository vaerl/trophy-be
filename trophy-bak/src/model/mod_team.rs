use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use super::{Game, Outcome};

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename = "team_gender")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TeamGender {
    Female,
    Male,
    Mixed,
}

#[derive(Deserialize, Serialize, FromRow)]
pub struct Team {
    pub id: i32,
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
            r#"SELECT id, name, gender as "gender: TeamGender" FROM team ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(teams)
    }

    pub async fn create(team: Team, pool: &PgPool) -> Result<Team> {
        let mut tx = pool.begin().await?;
        let team = sqlx::query_as!(
            Team, 
            r#"INSERT INTO team (id, name, gender) VALUES ($1, $2, $3 ) RETURNING id, name, gender as "gender: TeamGender""#,
            team.id, team.name, team.gender as TeamGender
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        for game in Game::find_all(pool).await? {
            Outcome::create(game.id, team.id, pool).await?;
        }

        Ok(team)
    }

    pub async fn update(team: Team, pool: &PgPool) -> Result<Team> {
        let mut tx = pool.begin().await?;
        let team = sqlx::query_as!(
            Team, 
            r#"UPDATE team SET id = $1, name = $2, gender = $3 WHERE id = $4 RETURNING id, name, gender as "gender: TeamGender""#,
            team.id, team.name, team.gender as TeamGender, team.id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(team)
    }

    pub async fn delete(team: Team, pool: &PgPool) -> Result<Team> {
        let mut tx = pool.begin().await?;
        let teams = sqlx::query_as!(
            Team,
            r#"DELETE FROM team WHERE id = $1 RETURNING id, name, gender as "gender: TeamGender""#,
            team.id
        )
        .fetch_one(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(teams)
    }
}
