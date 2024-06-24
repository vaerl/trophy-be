use crate::ApiResult;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: bool,
}

#[derive(Deserialize)]
pub struct Year {
    year: i32,
}

impl std::ops::Deref for Year {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.year
    }
}

#[derive(Serialize)]
pub struct YearVec(pub Vec<i32>);

impl Year {
    /// Find all currently used years.
    pub async fn find_all(pool: &PgPool) -> ApiResult<YearVec> {
        let mut team_years: Vec<i32> = sqlx::query_as!(Year, r#"SELECT year from teams"#)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|y| y.year)
            .collect();

        let mut game_years: Vec<i32> = sqlx::query_as!(Year, r#"SELECT year from games"#)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|y| y.year)
            .collect();

        team_years.append(&mut game_years);
        team_years.sort();
        team_years.dedup();
        Ok(YearVec(team_years))
    }
}
