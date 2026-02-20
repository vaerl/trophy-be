use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{Salt, SaltString},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::fmt::{self, Display};
use uuid::Uuid;

use crate::{ApiResult, model::Game};

use super::{CreateToken, CustomError, TypeInfo, UserToken};

#[derive(Serialize, Deserialize, sqlx::Type, PartialEq, Debug)]
#[sqlx(type_name = "user_role")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Referee,
    Visualizer,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Admin => write!(f, "Admin"),
            UserRole::Referee => write!(f, "Referee"),
            UserRole::Visualizer => write!(f, "Visualizer"),
        }
    }
}

#[derive(Serialize, FromRow, Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub password: String,
    pub role: UserRole,
    pub session: Option<Uuid>,
    pub game_id: Option<Uuid>,
    pub game_name: Option<String>,
}

#[derive(Serialize)]
// this syntax is brilliant!
pub struct UserVec(Vec<User>);

#[derive(Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub password: String,
    pub role: UserRole,
    pub game_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdateUser {
    pub name: String,
    pub role: UserRole,
    pub password: Option<String>,
    pub game_id: Option<Uuid>,
}

impl fmt::Display for CreateUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CreateUser(name: {}, password: {}, role: {})",
            self.name, self.password, self.role
        )
    }
}

#[derive(Deserialize)]
pub struct CreateLogin {
    pub name: String,
    pub password: String,
}

/// NOTE `LEFT JOIN`s are correct here - we don't know if users.game_id is null or not.
/// By using `field as "field?"` we make sqlx clear that it's potentially nullable - this solved an issue where sqlx would complain even though
/// the receiving field was accepting an `Option` - more [here](https://github.com/launchbadge/sqlx/issues/1852).
impl User {
    /// Find all [User]s.
    pub async fn find_all(pool: &PgPool) -> ApiResult<UserVec> {
        let users = sqlx::query_as!(
            User,
            r#"SELECT users.id, users.name, password, role as "role: UserRole", game_id, games.name as "game_name?", session FROM users
            LEFT JOIN games ON games.id=users.game_id
            ORDER BY id"#
        )
        .fetch_all(pool)
        .await?;

        Ok(UserVec(users))
    }

    /// Try to get the [User] with the specified ID.
    pub async fn find(id: Uuid, pool: &PgPool) -> ApiResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT users.id, users.name, password, role as "role: UserRole", game_id, games.name as "game_name?", session FROM users
            LEFT JOIN games ON games.id=users.game_id
            WHERE users.id = $1"#, id
        )
        .fetch_optional(pool)
        .await?;

        user.ok_or(CustomError::NotFoundError {
            message: format!("User {} could not be found.", id),
        })
    }

    /// Try to get the [User] with the specified name.
    pub async fn find_by_name(name: &String, pool: &PgPool) -> ApiResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT users.id, users.name, users.password, users.role as "role: UserRole", game_id, games.name as "game_name?", users.session FROM users
            LEFT JOIN games ON users.game_id=games.id
            WHERE users.name = $1"#, name
        ).fetch_optional(pool)
        .await?;

        user.ok_or(CustomError::NotFoundError {
            message: format!("User {} could not be found.", name),
        })
    }

    /// Try to get the game of the specified [User].
    pub async fn find_game_for_ref(user_id: Uuid, pool: &PgPool) -> ApiResult<Game> {
        let user = User::find(user_id, pool).await?;

        match user.game_id {
            Some(game_id) => Game::find(game_id, pool).await,
            None => Err(CustomError::NotFoundError {
                message: format!("Game for user {} could not be found!", user_id),
            }),
        }
    }

    pub async fn create(create_user: CreateUser, pool: &PgPool) -> ApiResult<User> {
        if User::find_by_name(&create_user.name, pool).await.is_ok() {
            return Err(CustomError::AlreadyExistsError {
                message: format!("User {} already exists!", create_user.name),
            });
        }

        // if a game is specified, check if it exists
        if create_user.game_id.is_some()
            && Game::find(create_user.game_id.unwrap(), pool)
                .await
                .is_err()
        {
            return Err(CustomError::NotFoundError {
                message: format!(
                    "Game {} does not exist!",
                    create_user
                        .game_id
                        .map(|id| id.to_string())
                        .unwrap_or("<no id>".to_string())
                ),
            });
        }

        info!("Creating new user.");

        // taken from https://github.com/launchbadge/sqlx/pull/3931#discussion_r2214203657
        let salt: [u8; Salt::RECOMMENDED_LENGTH] = rand::random();
        let salt = SaltString::encode_b64(&salt)
            .expect("Should not fail since we generated a salt of recommended length.");

        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(create_user.password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        let mut tx = pool.begin().await?;

        // insert the user - sqlx doesn't handle LEFT JOIN correctly as it infers fields of the non-joined part to also be optional
        let inserted_user = sqlx::query!(
            r#"INSERT INTO users (id, name, password, role, game_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, password, role as "role: UserRole", game_id, session"#,
            Uuid::now_v7(),
            create_user.name,
            password_hash,
            create_user.role as UserRole,
            create_user.game_id
        )
        .fetch_one(&mut *tx)
        .await?;

        // Then fetch the game name if there's a game_id
        let game_name = if let Some(game_id) = inserted_user.game_id {
            sqlx::query!("SELECT name FROM games WHERE id = $1", game_id)
                .fetch_optional(&mut *tx)
                .await?
                .map(|row| row.name)
        } else {
            None
        };

        tx.commit().await?;

        // construct the User struct manually
        let user = User {
            id: inserted_user.id,
            name: inserted_user.name,
            password: inserted_user.password,
            role: inserted_user.role,
            session: inserted_user.session,
            game_id: inserted_user.game_id,
            game_name,
        };

        Ok(user)
    }

    /// Update an existing [User].
    /// Passing a new password updates the password.
    pub async fn update(id: Uuid, altered_user: UpdateUser, pool: &PgPool) -> ApiResult<User> {
        match altered_user.password {
            Some(password) => {
                // taken from https://github.com/launchbadge/sqlx/pull/3931#discussion_r2214203657
                let salt: [u8; Salt::RECOMMENDED_LENGTH] = rand::random();
                let salt = SaltString::encode_b64(&salt)
                    .expect("Should not fail since we generated a salt of recommended length.");

                let argon2 = Argon2::default();
                let password_hash = argon2
                    .hash_password(password.as_bytes(), &salt)
                    .unwrap()
                    .to_string();

                let mut tx = pool.begin().await?;
                let user = sqlx::query_as!(
                    User,
                    r#"
                    WITH updated AS (UPDATE users SET name = $1, password = $2, role = $3, game_id = $4 WHERE id = $5 RETURNING *)
                    SELECT
                        updated.id as "id!",
                        updated.name as "name!",
                        password as "password!",
                        role as "role!: UserRole",
                        game_id,
                        games.name as "game_name?",
                        session FROM updated
                    RIGHT JOIN games ON games.id=updated.game_id"#,
                    altered_user.name, password_hash, altered_user.role as UserRole, altered_user.game_id, id
                )
                .fetch_one(&mut *tx)
                .await?;

                tx.commit().await?;
                Ok(user)
            }
            None => {
                let mut tx = pool.begin().await?;
                let user = sqlx::query_as!(
                    User,
                     r#"
                    WITH updated AS (UPDATE users SET name = $1, role = $2, game_id = $3 WHERE id = $4 RETURNING *)
                    SELECT
                        updated.id as "id!",
                        updated.name as "name!",
                        password as "password!",
                        role as "role!: UserRole",
                        game_id,
                        games.name as "game_name?",
                        session FROM updated
                    LEFT JOIN games ON games.id=updated.game_id"#,
                    altered_user.name, altered_user.role as UserRole, altered_user.game_id, id
                )
                .fetch_one(&mut *tx)
                .await?;
                tx.commit().await?;
                Ok(user)
            }
        }
    }

    async fn update_session(user_id: Uuid, session: Option<Uuid>, pool: &PgPool) -> ApiResult<()> {
        let mut tx = pool.begin().await?;

        sqlx::query_as!(
            User,
            r#"UPDATE users SET session = $1 WHERE id = $2"#,
            session,
            user_id
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(())
    }

    /// Delete the specified [User].
    pub async fn delete(id: Uuid, pool: &PgPool) -> ApiResult<User> {
        let mut tx = pool.begin().await?;

        // NOTE since I'm using LEFT JOIN, I have to assert that the fields are actually present
        let user = sqlx::query_as!(
            User,
            r#"
            WITH deleted AS (
                DELETE FROM users WHERE id = $1
                RETURNING *
            )
            SELECT
                deleted.id as "id!",
                deleted.name as "name!",
                deleted.password as "password!",
                deleted.role as "role!: UserRole",
                deleted.game_id,
                games.name as "game_name?",
                deleted.session
            FROM deleted
            LEFT JOIN games ON deleted.game_id = games.id
            "#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(user)
    }

    pub async fn login(login: CreateLogin, pool: &PgPool) -> ApiResult<String> {
        let user = User::find_by_name(&login.name, pool).await?;
        let argon2 = Argon2::default();
        let password_hash = PasswordHash::new(&user.password)?;

        if user.password.is_empty()
            || argon2
                .verify_password(login.password.as_bytes(), &password_hash)
                .is_err()
        {
            Err(CustomError::BadPasswordError {
                message: "Token is invalid!".to_string(),
            })
        } else {
            let session = User::generate_session();
            User::update_session(user.id, Some(session), pool).await?;
            Ok(UserToken::generate_token(
                &CreateToken {
                    user_id: user.id,
                    session,
                },
                user,
            ))
        }
    }

    /// Logout the specified [User].
    pub async fn logout(id: Uuid, pool: &PgPool) -> ApiResult<()> {
        User::update_session(id, None, pool).await
    }

    /// Generate a new v4-[Uuid] to use as our session-identifier.
    /// Also see: https://en.wikipedia.org/wiki/Universally_unique_identifier#Version_4_(random)
    pub fn generate_session() -> Uuid {
        Uuid::new_v4()
    }
}

impl Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Game(id: {}, name: {}, role: {}, game_id: {:#?})",
            self.id, self.name, self.role, self.game_id
        )
    }
}

impl TypeInfo for User {
    fn type_name(&self) -> String {
        "User".to_string()
    }
}

impl Display for UserVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UserVec[{}]",
            self.0.iter().map(|g| g.to_string()).collect::<String>()
        )
    }
}

impl TypeInfo for UserVec {
    fn type_name(&self) -> String {
        "UserVec".to_string()
    }
}
