use thiserror::Error;

#[derive(Error, Debug)]
pub enum VKeyError {
    #[error("Input error: {0}")]
    InputError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("System error: {0}")]
    SystemError(String),
}

pub type Result<T> = std::result::Result<T, VKeyError>; 