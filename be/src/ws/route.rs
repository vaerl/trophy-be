use actix::Addr;
use actix_web::{
    get,
    web::{self, Data, Payload},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws;

use crate::ApiResult;

use super::{lobby::Lobby, ws::WsConn};

// NOTE to use this socket, forward the app's port!
#[get("/socket")]
pub async fn start_connection(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<Lobby>>,
) -> ApiResult<HttpResponse> {
    // TODO check for role
    // TODO log connections with user_id -> probably don't use history!
    debug!("User connected to socket.");
    let ws = WsConn::new(srv.get_ref().clone());

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)

    // TODO use only one room that everybody can listen to
    // -> send messages when I receive CREATE, DELETE or UPDATE at the REST-endpoints
    // -> send only refresh-messages: { subject: "Team" } -> only refresh teams here

    // TODO maybe keep room-architecture for future uses???
    // -> I could generate a Uuid and pass everybody to that room!
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(start_connection);
}
