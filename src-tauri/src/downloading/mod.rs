use serde::{Deserialize, Serialize};

pub mod preload;
pub mod repair;
pub mod update;
pub mod download;
pub mod misc;

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadGamePayload {
    pub install: String,
    pub biz: String,
    pub lang: String,
    pub region: String,
    pub is_latest: Option<String>,
}