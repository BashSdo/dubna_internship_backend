use std::{net, time};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub db: Db,
    pub http: Http,
    pub jwt: Jwt,
}

#[derive(Deserialize)]
pub struct Db {
    pub url: String,
}

#[derive(Deserialize)]
pub struct Http {
    pub server: Server,
    pub cors: Cors,
}

#[derive(Deserialize)]
pub struct Server {
    pub addr: net::SocketAddr,
}

#[derive(Deserialize)]
pub struct Cors {
    pub allowed_origins: Vec<String>,
}

#[derive(Deserialize)]
pub struct Jwt {
    pub secret: String,
    #[serde(with = "humantime_serde")]
    pub expiration_time: time::Duration,
}
