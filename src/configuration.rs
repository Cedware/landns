use std::net::IpAddr;

#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Invalid value for PORT: {0}")]
    InvalidPort(String),
    #[error("Invalid value for LOCAL_ADDRESS: {0}")]
    InvalidLocalAddress(String),
    #[error("key file at '{0}' does not exist")]
    KeyFileDoesNotExist(String),
}

pub struct Configuration {
    pub port: u16,
    pub local_address: IpAddr,
    pub key_path: Option<String>
}

impl Configuration {
    pub fn from_env() -> Result<Self, ConfigurationError> {

        let port = std::env::var("PORT")
            .map(|port| port.parse().map_err(|_| ConfigurationError::InvalidPort(port)))
            .unwrap_or(Ok(3853))?;

        let listen_address = std::env::var("LOCAL_ADDRESS")
            .map(|address| address.parse().map_err(|_| ConfigurationError::InvalidLocalAddress(address)))
            .unwrap_or(Ok(IpAddr::from([0, 0, 0, 0])))?;

        let key_path = std::env::var("KEY_PATH")
            .ok();

        if let Some(key_path) = &key_path {
            if !std::path::Path::new(key_path).exists() {
                return Err(ConfigurationError::KeyFileDoesNotExist(key_path.to_string()));
            }
        }

        Ok(Configuration {
            port,
            local_address: listen_address,
            key_path
        })
    }
}

