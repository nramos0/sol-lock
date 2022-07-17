//! A program that locks SOL in an account until a unix timestamp
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod entrypoint;
mod error;
pub mod instruction;
mod pack_utils;
pub mod processor;
mod state;
mod validation_utils;
