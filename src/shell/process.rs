use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Completed(i32),
    Failed(i32),
    Killed,
}

pub struct ProcessHandle {
    pub id: Uuid,
    pub command: String,
    pub status: Arc<std::sync::Mutex<ProcessStatus>>,
    pub should_cancel: Arc<AtomicBool>,
}

impl ProcessHandle {
    pub fn new(command: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            command,
            status: Arc::new(std::sync::Mutex::new(ProcessStatus::Running)),
            should_cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.should_cancel.store(true, Ordering::SeqCst);
        if let Ok(mut status) = self.status.lock() {
            *status = ProcessStatus::Killed;
        }
    }

    pub fn is_running(&self) -> bool {
        if let Ok(status) = self.status.lock() {
            matches!(*status, ProcessStatus::Running)
        } else {
            false
        }
    }

    pub fn get_status(&self) -> ProcessStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn set_status(&self, new_status: ProcessStatus) {
        if let Ok(mut status) = self.status.lock() {
            *status = new_status;
        }
    }
}
