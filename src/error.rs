use thiserror::Error;

#[derive(Error, Debug)]
pub enum HarnessError {
    #[error("Network Request failed {0}")] 
    RequestFailed(#[from] reqwest::Error),

    #[error("Failed to parse bytes to UTF-8: {0}")] 
    ParseFailed(#[from] std::str::Utf8Error),

    #[error("Unable to find API Key in .env, {0}")]
    ApiKeyError(#[from] std::env::VarError),

    
    #[error("Error deserializing json into struct: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Could not flush to terminal: {0}")]
    FlushError(#[from] std::io::Error)
    
}
