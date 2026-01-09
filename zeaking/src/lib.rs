// Zeaking - Fast Local Blockchain Indexing System

pub mod error;
pub mod indexer;
pub mod traits;
pub mod types;

pub use error::{ZeakingError, ZeakingResult};
pub use indexer::{DefaultBlockParser, Zeaking};
pub use traits::{BlockParser as BlockParserTrait, BlockSource};
pub use types::{IndexStats, IndexedBlock, IndexedTransaction, OrchardActionIndex};
