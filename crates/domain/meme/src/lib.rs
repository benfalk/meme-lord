pub mod adapter;
pub mod entity;
pub mod port;
pub mod types;

pub mod command;
pub mod query;

pub use error::Error;
pub use sevice::Service;

mod error;
mod sevice;
#[cfg(test)]
mod support;
