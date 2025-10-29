// Shell executor module
// Handles command execution through bash

pub mod executor;
pub mod process;

pub use executor::{OutputLine, ShellExecutor};
pub use process::{ProcessHandle, ProcessStatus};
