// Core data structures module
// Contains Block, Session, and BlockManager implementations

pub mod block;
pub mod session;
pub mod manager;

pub use block::{Block, BlockState, BlockMetadata};
pub use session::Session;
pub use manager::BlockManager;
