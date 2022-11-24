use ruc::*;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LocalConfig {
    pub server: Server,
    pub redis: Redis,
    pub bootstrap: Bootstrap,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Redis {
    pub addr: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Server {
    pub addr: String,
    pub port: u64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Bootstrap {
    pub start: i64,
}

impl LocalConfig {
    pub fn new(path: &str) -> Result<Self> {
        let mut file = File::open(path).c(d!())?;

        let mut str = String::new();
        file.read_to_string(&mut str).c(d!())?;

        let config: LocalConfig = toml::from_str(&str).c(d!())?;
        Ok(config)
    }
}
