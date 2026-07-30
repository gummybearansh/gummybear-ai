use thiserror::Error;

#[derive(Error, Debug)]
pub enum HarnessError {
   #[error("Network Request failed")] 
    RequestFailed(#[from] reqwest::Error),

   #[error("Failed to parse bytes to UTF-8: {0}")] 
    ParseFailed(#[from] std::str::Utf8Error),
}
