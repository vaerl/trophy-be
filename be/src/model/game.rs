use std::fmt::{self, Display};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use crate::{ApiResult, TypeInfo};
use super::{ CustomError, Outcome, Team};

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "game_kind")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum GameKind {
    Points,
    Time,
}

// Only return the name with no other information - this will be combined later.
impl fmt::Display for GameKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameKind::Points => write!(f, "Points"),
            GameKind::Time => write!(f, "Points"),
        }
    }
}

#[derive(Serialize, FromRow)]
pub struct Game {
    pub id: i32,
    pub trophy_id: i32,
    pub name: String,
    pub kind: GameKind,
    pub year: i32,
}

#[derive(Serialize)]
pub struct GameVec(pub Vec<Game>);

/// To create a new game, I have to create one user (that acts as a referee) first.
#[derive(Deserialize)]
pub struct CreateGame {
    pub trophy_id: i32,
    pub name: String,
    pub kind: GameKind,
    pub year: i32,
}

impl Game {

    pub async fn find_all(pool: &PgPool, year: i32) -> ApiResult<GameVec> {
        let games = sqlx::query_as!(
            Game,
            r#"SELECT id, trophy_id, name, kind as "kind: GameKind", year FROM games WHERE year = $1 ORDER BY id"#, year
        )
        .fetch_all(pool)
        .await?;

        Ok(GameVec(games))
    }

    pub async fn find(id: i32, pool: &PgPool) -> ApiResult<Game> {
        let game = sqlx::query_as!(
            Game,
            r#"SELECT id, trophy_id, name, kind as "kind: GameKind", year FROM games WHERE id = $1"#, id
        )
        .fetch_optional(pool)
        .await?;

        game.ok_or(CustomError::NotFoundError { message: format!("Game {} could not be found.", id) })
    }

    /// Create a new game.
    pub async fn create(create_game: CreateGame, pool: &PgPool) -> ApiResult<Game> {
        let mut tx = pool.begin().await?;
        let game: Game = sqlx::query_as!(Game, 
            r#"INSERT INTO games (trophy_id, name, kind, year) VALUES ($1, $2, $3, $4) RETURNING id, trophy_id, name, kind as "kind: GameKind", year"#,
            create_game.trophy_id, create_game.name, create_game.kind as GameKind, create_game.year
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        // create outcomes
        for team in Team::find_all(pool, create_game.year).await?.0 {
            Outcome::create(game.id, team.id, pool).await?;
        }

        Ok(game)
    }

    /// Update the specified game.
    pub async fn update(id: i32, altered_game: CreateGame, pool: &PgPool) -> ApiResult<Game> {
        // NOTE I've decided against being able to change the year of already created games (for now)
        let mut tx = pool.begin().await?;
        let game = sqlx::query_as!(
            Game, 
            r#"UPDATE games SET trophy_id = $1, name = $2, kind = $3 WHERE id = $4 RETURNING id, trophy_id, name, kind as "kind: GameKind", year"#,
            altered_game.trophy_id, altered_game.name, altered_game.kind as GameKind, id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(game)
    }

    /// Delete the specified game.
    pub async fn delete(id: i32, pool: &PgPool) -> ApiResult<Game> {
        let mut tx = pool.begin().await?;
        let game = sqlx::query_as!(
            Game,
            r#"DELETE FROM games WHERE id = $1 RETURNING id, trophy_id, name, kind as "kind: GameKind", year"#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(game)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Game(id: {}, trophy_id: {}, name: {}, kind: {})",self.id, self.trophy_id, self.name, self.kind)
    }
}

impl TypeInfo for Game {
    fn type_name(&self) -> String {
       "Game".to_string()
    }
}

impl Display for GameVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GameVec[{}]", self.0.iter().map(|g| g.to_string()).collect::<String>())
    }
}

impl TypeInfo for GameVec {
    fn type_name(&self) -> String {
       "GameVec".to_string()
    }
}
