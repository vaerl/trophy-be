use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Row};

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename = "gender")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Female,
    Male,
    Mixed,
}

#[derive(Serialize, FromRow)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub gender: Gender,
}

#[derive(Deserialize)]
pub struct TeamRequest {
    pub name: String,
    pub gender: Gender,
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
        let mut teams = vec![];
        teams = sqlx::query_as!(
            Team,
            r#"SELECT id, name, gender as "gender: Gender" FROM team ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(teams)
    }

    pub async fn create(team: TeamRequest, pool: &PgPool) -> Result<Team> {
        let mut tx = pool.begin().await?;
        let team = sqlx::query(
            "INSERT INTO team (name, gender) VALUES ($1, $2) RETURNING id, name, gender",
        )
        .bind(&team.name)
        .bind(team.gender)
        .map(|row: PgRow| Team {
            id: row.get(0),
            name: row.get(1),
            gender: row.get(2),
        })
        .fetch_one(&mut tx)
        .await?;

        // TODO populate n:m

        tx.commit().await?;
        Ok(team)
    }
}
