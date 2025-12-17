// Zeaking - Fast Local Blockchain Indexing System


pub mod error;
pub mod indexer;
pub mod traits;
pub mod types;

pub use error::{ZeakingError, ZeakingResult};
pub use indexer::{Zeaking, DefaultBlockParser};
pub use traits::{BlockSource, BlockParser as BlockParserTrait};
pub use types::{IndexedBlock, IndexedTransaction, IndexStats, OrchardActionIndex};

