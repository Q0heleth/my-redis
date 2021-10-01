pub mod server;
mod db;
mod shutdown;
mod connection;

use connection::{Connection};
use shutdown::{Shutdown};
use db::{Db,DbDropGuard};


pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T,Error>;
pub const DEFAULT_PORT: &str = "6378";