use super::{CustomError, Outcome, Team};
use crate::{ApiResult, TypeInfo, model::Amount};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::fmt::{self, Display};
use uuid::Uuid;

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
    pub id: Uuid,
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
    /// Find all [Game]s.
    pub async fn find_all(pool: &PgPool, year: i32) -> ApiResult<GameVec> {
        let games = sqlx::query_as!(
            Game,
            r#"SELECT id, trophy_id, name, kind as "kind: GameKind", year FROM games WHERE year = $1 ORDER BY id"#, year
        )
        .fetch_all(pool)
        .await?;

        Ok(GameVec(games))
    }

    /// Find all pending [Game]s.
    pub async fn find_all_pending(year: i32, pool: &PgPool) -> ApiResult<Amount> {
        let amount = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM (
                SELECT DISTINCT game_id FROM game_team
                    INNER JOIN games ON game_team.game_id=games.id
                    INNER JOIN teams ON game_team.team_id=teams.id
                WHERE data IS NULL AND games.year = $1) AS temp"#,
            year
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        Ok(Amount(amount))
    }
    /// Try to get the [Game] of the specified ID.
    pub async fn find(id: Uuid, pool: &PgPool) -> ApiResult<Game> {
        let game = sqlx::query_as!(
            Game,
            r#"SELECT id, trophy_id, name, kind as "kind: GameKind", year FROM games WHERE id = $1"#, id
        )
        .fetch_optional(pool)
        .await?;

        game.ok_or(CustomError::NotFoundError {
            message: format!("Game {} could not be found.", id),
        })
    }

    /// Create a new [Game].
    pub async fn create(create_game: CreateGame, pool: &PgPool) -> ApiResult<Game> {
        let mut tx = pool.begin().await?;
        let game: Game = sqlx::query_as!(
            Game,
            r#"INSERT INTO games (id, trophy_id, name, kind, year)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, trophy_id, name, kind as "kind: GameKind", year"#,
            Uuid::now_v7(),
            create_game.trophy_id,
            create_game.name,
            create_game.kind as GameKind,
            create_game.year
        )
        .fetch_one(&mut *tx)
        .await?;

        // create outcomes
        for team in Team::find_all(pool, create_game.year).await?.0 {
            Outcome::create(game.id, team.id, &mut tx).await?;
        }

        tx.commit().await?;
        Ok(game)
    }

    /// Update the specified [Game].
    pub async fn update(id: Uuid, altered_game: CreateGame, pool: &PgPool) -> ApiResult<Game> {
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

    /// Delete the specified [Game].
    pub async fn delete(id: Uuid, pool: &PgPool) -> ApiResult<Game> {
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

    pub async fn is_pending(&self, pool: &PgPool) -> ApiResult<bool> {
        let amount = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM (
                SELECT DISTINCT game_id FROM game_team
                    INNER JOIN games ON game_team.game_id=games.id
                    INNER JOIN teams ON game_team.team_id=teams.id
                WHERE data IS NULL
                AND games.year = $1
                AND games.id = $2)
            AS temp"#,
            self.year,
            self.id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        Ok(amount > 0)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Game(id: {}, trophy_id: {}, name: {}, kind: {})",
            self.id, self.trophy_id, self.name, self.kind
        )
    }
}

impl TypeInfo for Game {
    fn type_name(&self) -> String {
        "Game".to_string()
    }
}

impl Display for GameVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GameVec[{}]",
            self.0.iter().map(|g| g.to_string()).collect::<String>()
        )
    }
}

impl TypeInfo for GameVec {
    fn type_name(&self) -> String {
        "GameVec".to_string()
    }
}
