pub mod server;
mod shutdown;
mod db;
use db::Db;
use db::DbDropGuard;
pub mod cmd;
 use cmd::Command;
 use cmd::Set;
 use cmd::Get;
mod connection;
mod parse;
use connection::Connection;
pub const DEFAULT_PORT: &str = "36379";
mod frame;
pub mod client;
use client::{Client};
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type  Result<T> = std::result::Result<T,Error>;