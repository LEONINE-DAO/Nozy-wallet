use thiserror::Error;

pub type IndexerResult<T> = Result<T, IndexerError>;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("config: {0}")]
    Config(String),
    #[error("rpc: {0}")]
    Rpc(String),
    #[error("storage: {0}")]
    Storage(String),
    #[error("compact encode: {0}")]
    Compact(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
