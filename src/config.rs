use std::{env, error::Error, io, net::SocketAddr};

pub struct Config {
    pub bind_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let value = match env::var("BIND_ADDR") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => "127.0.0.1:3000".to_owned(),
            Err(error) => return Err(error.into()),
        };
        Self::from_bind_addr(&value)
    }

    pub fn from_bind_addr(value: &str) -> Result<Self, Box<dyn Error>> {
        let bind_addr: SocketAddr = value.parse().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "BIND_ADDR must be an IP:port socket address")
        })?;
        if bind_addr.port() == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "BIND_ADDR port must be nonzero").into());
        }
        Ok(Self { bind_addr })
    }
}
