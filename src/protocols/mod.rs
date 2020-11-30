use std::fmt;

use derive_more::Display;
use thiserror::Error;

use crate::protocols::gossip::gossip::*;
use crate::protocols::swim::messages::*;

pub mod swim;
pub mod gossip;
pub mod nat;

/// A `Result` alias where the `Err` case is `protocol::ProtocolError`.
// pub type Result<T> = std::result::Result<T, ProtocolError>;
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("Swim protocol error: {0}")]
    Swim(String),
    #[error("unknown data store error")]
    Unknown,
}
