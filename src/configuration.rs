#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError {
    #[error("Invalid port: {0}")]
    InvalidPort(String),
}

pub struct Configuration {
    pub port: u16,
    pub listen_address: String,
}

impl Configuration {
    pub fn from_env() -> Result<Self, ConfigurationError> {
        let port = std::env::var("PORT")
            .map(|port| port.parse::<u16>().map_err(|_| ConfigurationError::InvalidPort(port)))
            .unwrap_or(Ok(3853))?;
        let listen_address = std::env::var("LISTEN_ADDRESS")
            .unwrap_or("0.0.0.0".to_string());
        Ok(Configuration {
            port,
            listen_address,
        })
    }
}

