mod error;
mod game;
mod history;
mod misc;
mod outcome;
mod parsed_outcome;
mod team;
mod user;
mod user_token;

pub use error::*;
pub use game::*;
pub use history::*;
pub use misc::*;
pub use outcome::*;
pub use parsed_outcome::*;
use serde::Serialize;
use std::fmt::Display;
pub use team::*;
pub use user::*;
pub use user_token::*;

use crate::TypeInfo;

#[derive(Serialize)]
pub struct Amount(pub i64);

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Amount({})", self.0)
    }
}

impl TypeInfo for Amount {
    fn type_name(&self) -> String {
        "Amount".to_string()
    }
}
