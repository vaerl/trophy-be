use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Row};

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
    // TODO DELETE
    // TODO UPDATE

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
        let team = sqlx::query(
            "INSERT INTO team (id, name, gender) VALUES ($1, $2, $3) RETURNING id, name, gender",
        )
        .bind(team.id)
        .bind(&team.name)
        .bind(team.gender)
        .map(|row: PgRow| Team {
            id: row.get(0),
            name: row.get(1),
            gender: row.get(2),
        })
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        for game in Game::find_all(pool).await? {
            Outcome::create(game.id, team.id, pool).await?;
        }

        Ok(team)
    }
}
