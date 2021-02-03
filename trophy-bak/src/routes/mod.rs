mod eval;
mod game;
mod misc;
mod outcome;
mod team;

use actix_web::web;
// make all routes publicly available
// TODO do I need this?
pub use eval::*;
pub use game::*;
pub use misc::*;
pub use outcome::*;
pub use team::*;

pub fn init(cfg: &mut web::ServiceConfig) {
    // This function calls all init-functions to mount the module's routes.
    misc::init(cfg);
    game::init(cfg);
    team::init(cfg);
    outcome::init(cfg);
    eval::init(cfg);
}
