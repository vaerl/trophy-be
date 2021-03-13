use anyhow::Result;
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::{AuthenticationError, User};

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

        jsonwebtoken::encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&KEY),
        )
        .unwrap()
    }

    pub fn decode_token(token: String) -> Result<Self, AuthenticationError> {
        let token_data = jsonwebtoken::decode::<UserToken>(
            &token,
            &DecodingKey::from_secret(&KEY),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    pub fn is_valid(&self) -> bool {
        let now = Utc::now().timestamp();
        info!("NOW: {}", now);
        info!("SELF: {}", self.exp);
        now < self.exp
    }
}
