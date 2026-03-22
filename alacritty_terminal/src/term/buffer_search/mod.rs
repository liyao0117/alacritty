//! Buffer Search module using nucleo-matcher's native pattern syntax.

pub mod extractor;
pub mod matcher;

pub use extractor::BufferExtractor;
pub use matcher::{match_query, MatchResult, SearchConfig, SearchLine};
