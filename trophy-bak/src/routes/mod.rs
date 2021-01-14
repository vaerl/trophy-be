mod game_routes;
mod misc_routes;
mod outcome_routes;
mod team_routes;

use actix_web::web;
// make all routes publicly available
// TODO do I need this?
pub use game_routes::*;
pub use misc_routes::*;
pub use outcome_routes::*;
pub use team_routes::*;

pub fn init(cfg: &mut web::ServiceConfig) {
    // This function calls all init-functions to mount the module's routes.
    misc_routes::init(cfg);
    game_routes::init(cfg);
    team_routes::init(cfg);
    outcome_routes::init(cfg);
}
