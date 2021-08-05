mod error;
mod game;
mod history;
mod outcome;
mod parsed_outcome;
mod team;
mod user;
mod user_token;

use actix_web::{body::Body, http::header::ContentType, HttpRequest, HttpResponse, Responder};
pub use error::*;
pub use game::*;
pub use history::*;
pub use outcome::*;
pub use parsed_outcome::*;
use serde::Serialize;
use std::fmt::Display;
pub use team::*;
pub use user::*;
pub use user_token::*;

use crate::derive_responder::Responder;

#[derive(Serialize, Responder)]
pub struct Amount(pub usize);

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Amount({})", self.0)
    }
}

impl TypeInfo for Amount {
    fn type_name(&self) -> String {
        format!("Amount")
    }
}
