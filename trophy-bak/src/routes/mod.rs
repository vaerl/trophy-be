mod game_routes;
mod misc_routes;
mod outcome_routes;
mod team_routes;

use actix_web::web;
pub use game_routes::*;
pub use misc_routes::*;
pub use outcome_routes::*;
pub use team_routes::*;

// function that will be called on new Application to configure routes for this module
pub fn init(cfg: &mut web::ServiceConfig) {
    misc_routes::init(cfg);
    game_routes::init(cfg);
    team_routes::init(cfg);
    outcome_routes::init(cfg);
}
