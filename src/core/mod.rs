// Core data structures module
// Contains Block, Session, BlockManager, and database implementations

pub mod block;
pub mod database;
pub mod manager;
pub mod session;
pub mod session_manager;

pub use block::{Block, BlockMetadata, BlockState};
pub use database::Database;
pub use manager::BlockManager;
pub use session::Session;
pub use session_manager::{SessionInfo, SessionManager};
