mod error;
mod game;
mod history;
mod outcome;
mod parsed_outcome;
mod team;
mod user;
mod user_token;

use actix_web::{HttpRequest, HttpResponse, Responder};
pub use error::*;
use futures::future::{ready, Ready};
pub use game::*;
pub use history::*;
pub use outcome::*;
pub use parsed_outcome::*;
use serde::Serialize;
pub use team::*;
pub use user::*;
pub use user_token::*;

use crate::{derive_responder::Responder, ApiResult};

#[derive(Serialize, Responder)]
pub struct Amount(pub usize);
