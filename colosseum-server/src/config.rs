// Copyright 2021 Chay Nabors.

use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Config {
    pub address: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Self { address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 20000)) }
    }
}

mod test {
    #[test]
    fn create_default_config() {
        use std::fs;
        use std::path::Path;

        use super::Config;

        let path = Path::new("config.json");

        if !path.exists() {
            let config = Config::default();
            fs::write(path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
        }
    }
}
