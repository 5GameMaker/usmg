use std::net::SocketAddr;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Http {
    pub bind: SocketAddr,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "Default::default")]
    pub http: Vec<Http>,
}
