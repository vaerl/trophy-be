use serde::Serialize;

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: bool,
}
