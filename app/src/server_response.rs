use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum ServerResponse {
    Success(String),
    Failure(String),
}
