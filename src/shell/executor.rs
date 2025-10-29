use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::Read;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::task;

#[derive(Debug, Clone)]
pub enum OutputLine {
    Stdout(String),
    Stderr(String),
    Exit(i32),
}

pub struct ShellExecutor {
    shell_path: String,
    working_directory: PathBuf,
}

impl ShellExecutor {
    pub fn new(shell_path: String) -> Result<Self> {
        let working_directory = std::env::current_dir()
            .context("Failed to get current directory")?;
        
        Ok(Self {
            shell_path,
            working_directory,
        })
    }

    pub fn set_working_directory(&mut self, path: PathBuf) {
        self.working_directory = path;
    }

    pub fn get_working_directory(&self) -> &PathBuf {
        &self.working_directory
    }

    /// Execute a command and return a channel for streaming output
    pub async fn execute(
        &self,
        command: String,
    ) -> Result<mpsc::UnboundedReceiver<OutputLine>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let shell_path = self.shell_path.clone();
        let working_dir = self.working_directory.clone();

        // Spawn blocking task for PTY operations
        task::spawn_blocking(move || {
            if let Err(e) = Self::execute_blocking(shell_path, working_dir, command, tx.clone()) {
                tracing::error!("Command execution error: {}", e);
                let _ = tx.send(OutputLine::Exit(-1));
            }
        });

        Ok(rx)
    }

    fn execute_blocking(
        shell_path: String,
        working_dir: PathBuf,
        command: String,
        tx: mpsc::UnboundedSender<OutputLine>,
    ) -> Result<()> {
        let pty_system = NativePtySystem::default();

        // Create a PTY pair
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to create PTY")?;

        // Create command that sources .bashrc first
        let mut cmd = CommandBuilder::new(&shell_path);
        cmd.arg("-c");
        
        // Source .bashrc (if it exists) before executing the command
        // Use login shell behavior to pick up environment
        let full_command = format!(
            "[ -f ~/.bashrc ] && source ~/.bashrc; {}",
            command
        );
        cmd.arg(&full_command);
        cmd.cwd(&working_dir);

        // Spawn the child process
        let mut child = pair
            .slave
            .spawn_command(cmd)
            .context("Failed to spawn command")?;

        // Drop the slave to close it in the parent process
        drop(pair.slave);

        // Read output from the master
        let mut reader = pair.master.try_clone_reader()?;
        let mut buffer = Vec::new();
        let mut temp_buf = [0u8; 8192];

        loop {
            match reader.read(&mut temp_buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    buffer.extend_from_slice(&temp_buf[..n]);
                    
                    // Process complete lines
                    while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                        let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<_>>();
                        if let Ok(line) = String::from_utf8(line_bytes) {
                            if tx.send(OutputLine::Stdout(line)).is_err() {
                                return Ok(()); // Receiver dropped
                            }
                        }
                    }

                    // Send partial line if buffer is getting large
                    if buffer.len() > 4096 {
                        if let Ok(line) = String::from_utf8(buffer.drain(..).collect()) {
                            let _ = tx.send(OutputLine::Stdout(line));
                        } else {
                            buffer.clear();
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    tracing::error!("Error reading from PTY: {}", e);
                    break;
                }
            }
        }

        // Send any remaining buffer
        if !buffer.is_empty() {
            if let Ok(line) = String::from_utf8(buffer) {
                let _ = tx.send(OutputLine::Stdout(line));
            }
        }

        // Wait for child to exit
        let exit_status = child
            .wait()
            .context("Failed to wait for child process")?;

        let exit_code = exit_status.exit_code() as i32;
        tracing::debug!("Command exited with code: {}", exit_code);
        let _ = tx.send(OutputLine::Exit(exit_code));

        Ok(())
    }

    /// Execute a simple command synchronously (for testing)
    pub fn execute_sync(&self, command: String) -> Result<(String, i32)> {
        let output = std::process::Command::new(&self.shell_path)
            .arg("-c")
            .arg(&command)
            .current_dir(&self.working_directory)
            .output()
            .context("Failed to execute command")?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{}{}", stdout, stderr);
        let exit_code = output.status.code().unwrap_or(-1);

        Ok((combined, exit_code))
    }
}

impl Default for ShellExecutor {
    fn default() -> Self {
        Self::new("/bin/bash".to_string()).expect("Failed to create default shell executor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_executor_creation() {
        let executor = ShellExecutor::new("/bin/bash".to_string());
        assert!(executor.is_ok());
    }

    #[test]
    fn test_simple_command() {
        let executor = ShellExecutor::default();
        let (output, exit_code) = executor.execute_sync("echo 'Hello, World!'".to_string()).unwrap();
        assert!(output.contains("Hello, World!"));
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_failed_command() {
        let executor = ShellExecutor::default();
        let (_, exit_code) = executor.execute_sync("false".to_string()).unwrap();
        assert_ne!(exit_code, 0);
    }

    #[tokio::test]
    async fn test_async_command() {
        let executor = ShellExecutor::default();
        let mut rx = executor.execute("echo 'Test'".to_string()).await.unwrap();
        
        let mut output = String::new();
        let mut exit_code = None;
        
        while let Some(line) = rx.recv().await {
            match line {
                OutputLine::Stdout(s) => output.push_str(&s),
                OutputLine::Exit(code) => exit_code = Some(code),
                _ => {}
            }
        }
        
        assert!(output.contains("Test"));
        assert_eq!(exit_code, Some(0));
    }
}
