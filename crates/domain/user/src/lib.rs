pub mod adapter;
pub mod command;
pub mod entity;
pub mod port;
pub mod query;
pub mod types;

pub use error::Error;
pub use service::Service;

mod error;
mod service;
#[cfg(test)]
mod support;
