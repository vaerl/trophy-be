use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: bool,
}

#[derive(Deserialize)]
pub struct Year {
    year: i32,
}

impl std::ops::Deref for Year {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.year
    }
}
