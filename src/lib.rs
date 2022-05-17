//! Library for creating an index for a canonical poker hand... especially holdem.
//!
//! This is a wrapper for Kevin Waugh's [poker hand isomorphisms](https://github.com/kdub0/hand-isomorphism).
//!
//! We export the friendly Indexer type, which makes using Kevin's algorithm
//! pretty easy for the rust using poker GTO set.
//!

mod indexer;
pub use indexer::{CardVec, IResult, IndexVec, Indexer, IndexerError, LazyIndexer};
