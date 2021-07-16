use actix_web::{FromRequest, HttpRequest};
use chrono::Utc;
use futures::future::{err, ready, Ready};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::ApiResult;

use super::{CustomError, User, UserRole};

pub static KEY: [u8; 16] = *include_bytes!("../../secret.key");

// values in seconds!
static ONE_DAY: i64 = 60 * 60 * 24;
static TWELVE_HOURS: i64 = 60 * 60 * 24;
static TWO_HOURS: i64 = 60 * 2 * 24;

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    // iat- and exp-names are required by jsonwebtoken!
    pub iat: i64,
    pub exp: i64,
    pub user_id: i32,
    pub login_session: String,
}

pub struct CreateToken {
    pub user_id: i32,
    pub session: String,
}

impl UserToken {
    pub fn generate_token(login: &CreateToken, user: User) -> String {
        // this should be in seconds!
        let now = Utc::now().timestamp();

        let expiration = match user.role {
            super::UserRole::Admin => now + TWO_HOURS,
            super::UserRole::Referee => now + TWELVE_HOURS,
            super::UserRole::Visualizer => now + ONE_DAY,
        };

        let payload = UserToken {
            iat: now,
            exp: expiration,
            user_id: login.user_id,
            login_session: login.session.clone(),
        };

        jsonwebtoken::encode::<UserToken>(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&KEY),
        )
        .unwrap()
    }

    pub fn decode_token(token: String) -> ApiResult<Self> {
        let token_data = jsonwebtoken::decode::<UserToken>(
            &token,
            &DecodingKey::from_secret(&KEY),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    pub fn is_valid(&self) -> bool {
        let now = Utc::now().timestamp();
        now < self.exp
    }

    /// Loads the user specified in the token, if:
    /// a) the token is not expired
    /// b) the user is logged in
    /// c) the user has one of the specified roles
    pub async fn try_into_authorized_user(
        req: &HttpRequest,
        roles: Vec<UserRole>,
        pool: &PgPool,
    ) -> ApiResult<User> {
        match req.cookie("session") {
            Some(cookie) => {
                let token = cookie.value();
                let token = jsonwebtoken::decode::<UserToken>(
                    token,
                    &DecodingKey::from_secret(&KEY),
                    &Validation::default(),
                )?
                .claims;
                // NOTE I've not found a way to get rid of the if-cascade - because I want specific errors!
                // 1: check if token is valid
                if token.is_valid() {
                    let user = User::find(token.user_id, pool).await?;
                    // 2: check if user is logged in
                    if !user.session.is_empty() {
                        // 3: check if user is allowed to access the resource
                        if roles.contains(&user.role) {
                            Ok(user)
                        } else {
                            Err(CustomError::AccessDeniedError)
                        }
                    } else {
                        Err(CustomError::UnauthorizedError)
                    }
                } else {
                    Err(CustomError::NoTokenError {
                        message: "Token is expired!".to_string(),
                    })
                }
            }
            None => Err(CustomError::NoTokenError {
                message: "No cookie provided!".to_string(),
            }),
        }
    }
}

impl FromRequest for UserToken {
    type Error = CustomError;
    type Future = Ready<ApiResult<UserToken>>;
    type Config = ();

    fn from_request(request: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_header = match request.headers().get("Authorization") {
            Some(auth_header) => auth_header,
            None => {
                // explicit return so that it's not assigned to auth_header
                return err(CustomError::NoTokenError {
                    message: "There was no authorization-header in the request!".to_string(),
                });
            }
        };
        let auth_str = auth_header.to_str().unwrap();
        if !auth_str.starts_with("bearer") && !auth_str.starts_with("Bearer") {
            return err(CustomError::NoTokenError {
                message: "There was no bearer-header in the request!".to_string(),
            });
        }
        let raw_token = auth_str[6..auth_str.len()].trim();
        ready(UserToken::decode_token(raw_token.to_string()))
    }
}
