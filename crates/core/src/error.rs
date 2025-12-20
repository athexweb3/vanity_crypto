use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Invalid hex string: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    #[error("Cryptography error: {0}")]
    CryptoError(String),

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}
