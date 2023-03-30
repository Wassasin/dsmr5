//! Convenience structs to get and keep the current state of the meter in memory.
//!
//! Usage of these types is entirely optional.
//! When only needing a few or single record, it is more efficient to directly filter on the
//! telegram objects iterator.

pub mod common;
pub mod dsmr4;
pub mod dsmr5;
pub mod e_mucs;
