use super::{Block, BlockState, Database, Session};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionManager {
    db: Arc<Database>,
}

impl SessionManager {
    pub async fn new(db: Database) -> Result<Self> {
        Ok(Self { db: Arc::new(db) })
    }

    /// Create a new session and save it to the database
    pub async fn create_session(&self, session: &Session) -> Result<()> {
        let env_json = serde_json::to_string(&session.environment)?;
        
        sqlx::query(
            r#"
            INSERT INTO sessions (id, name, created_at, updated_at, working_directory, environment, is_active)
            VALUES (?, ?, ?, ?, ?, ?, 1)
            "#
        )
        .bind(session.id.to_string())
        .bind(&session.name)
        .bind(session.created_at.to_rfc3339())
        .bind(session.updated_at.to_rfc3339())
        .bind(session.working_directory.to_string_lossy().to_string())
        .bind(env_json)
        .execute(self.db.pool())
        .await
        .context("Failed to create session")?;

        tracing::info!("Created session: {} ({})", session.name, session.id);
        Ok(())
    }

    /// Load a session by ID
    pub async fn load_session(&self, session_id: &Uuid) -> Result<Session> {
        let row = sqlx::query(
            "SELECT id, name, created_at, updated_at, working_directory, environment FROM sessions WHERE id = ?"
        )
        .bind(session_id.to_string())
        .fetch_one(self.db.pool())
        .await
        .context("Failed to load session")?;

        let id: String = row.get("id");
        let name: String = row.get("name");
        let created_at: String = row.get("created_at");
        let updated_at: String = row.get("updated_at");
        let working_directory: String = row.get("working_directory");
        let environment_json: Option<String> = row.get("environment");

        let environment: HashMap<String, String> = environment_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        let mut session = Session {
            id: Uuid::parse_str(&id)?,
            name,
            created_at: DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at)?.with_timezone(&Utc),
            working_directory: PathBuf::from(working_directory),
            environment,
            blocks: Vec::new(),
        };

        // Load blocks for this session
        session.blocks = self.load_blocks(session_id).await?;

        Ok(session)
    }

    /// Get all sessions (without loading blocks)
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let rows = sqlx::query(
            "SELECT id, name, created_at, updated_at, is_active FROM sessions ORDER BY updated_at DESC"
        )
        .fetch_all(self.db.pool())
        .await?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(SessionInfo {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                name: row.get("name"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))?.with_timezone(&Utc),
                is_active: row.get("is_active"),
            });
        }

        Ok(sessions)
    }

    /// Save a block to the database
    pub async fn save_block(&self, session_id: &Uuid, block: &Block, order: i32) -> Result<()> {
        let env_json = serde_json::to_string(&block.metadata.environment)?;
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO blocks 
            (id, session_id, timestamp, command, output, exit_code, state, working_directory, 
             environment, started_at, completed_at, duration_ms, is_collapsed, block_order)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(block.id.to_string())
        .bind(session_id.to_string())
        .bind(block.timestamp.to_rfc3339())
        .bind(&block.command)
        .bind(&block.output)
        .bind(block.exit_code)
        .bind(format!("{:?}", block.state))
        .bind(block.metadata.working_directory.to_string_lossy().to_string())
        .bind(env_json)
        .bind(block.metadata.started_at.map(|dt| dt.to_rfc3339()))
        .bind(block.metadata.completed_at.map(|dt| dt.to_rfc3339()))
        .bind(block.metadata.duration.map(|d| d.as_millis() as i64))
        .bind(block.is_collapsed)
        .bind(order)
        .execute(self.db.pool())
        .await
        .context("Failed to save block")?;

        Ok(())
    }

    /// Load all blocks for a session
    async fn load_blocks(&self, session_id: &Uuid) -> Result<Vec<Block>> {
        let rows = sqlx::query(
            r#"
            SELECT id, timestamp, command, output, exit_code, state, working_directory,
                   environment, started_at, completed_at, duration_ms, is_collapsed
            FROM blocks
            WHERE session_id = ?
            ORDER BY block_order ASC
            "#
        )
        .bind(session_id.to_string())
        .fetch_all(self.db.pool())
        .await?;

        let mut blocks = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let timestamp: String = row.get("timestamp");
            let working_directory: String = row.get("working_directory");
            let environment_json: Option<String> = row.get("environment");
            let state_str: String = row.get("state");

            let environment: HashMap<String, String> = environment_json
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();

            let started_at: Option<String> = row.get("started_at");
            let completed_at: Option<String> = row.get("completed_at");
            let duration_ms: Option<i64> = row.get("duration_ms");

            let state = match state_str.as_str() {
                "Editing" => BlockState::Editing,
                "Running" => BlockState::Running,
                "Completed" => BlockState::Completed,
                "Failed" => BlockState::Failed,
                "Cancelled" => BlockState::Cancelled,
                _ => BlockState::Completed,
            };

            blocks.push(Block {
                id: Uuid::parse_str(&id)?,
                timestamp: DateTime::parse_from_rfc3339(&timestamp)?.with_timezone(&Utc),
                command: row.get("command"),
                output: row.get("output"),
                exit_code: row.get("exit_code"),
                state,
                metadata: super::BlockMetadata {
                    duration: duration_ms.map(|ms| Duration::from_millis(ms as u64)),
                    working_directory: PathBuf::from(working_directory),
                    environment,
                    started_at: started_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                    completed_at: completed_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc))),
                },
                is_collapsed: row.get("is_collapsed"),
                is_selected: false,
                original_input: None, // Not stored in DB yet
            });
        }

        Ok(blocks)
    }

    /// Update session's updated_at timestamp
    pub async fn touch_session(&self, session_id: &Uuid) -> Result<()> {
        sqlx::query("UPDATE sessions SET updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(session_id.to_string())
            .execute(self.db.pool())
            .await?;
        Ok(())
    }

    /// Set a session as active (and deactivate others)
    pub async fn set_active_session(&self, session_id: &Uuid) -> Result<()> {
        // Deactivate all sessions
        sqlx::query("UPDATE sessions SET is_active = 0")
            .execute(self.db.pool())
            .await?;

        // Activate the specified session
        sqlx::query("UPDATE sessions SET is_active = 1 WHERE id = ?")
            .bind(session_id.to_string())
            .execute(self.db.pool())
            .await?;

        Ok(())
    }

    /// Get the currently active session
    pub async fn get_active_session(&self) -> Result<Option<Session>> {
        let row = sqlx::query("SELECT id FROM sessions WHERE is_active = 1 LIMIT 1")
            .fetch_optional(self.db.pool())
            .await?;

        if let Some(row) = row {
            let id: String = row.get("id");
            let session_id = Uuid::parse_str(&id)?;
            Ok(Some(self.load_session(&session_id).await?))
        } else {
            Ok(None)
        }
    }

    /// Delete a session and all its blocks
    pub async fn delete_session(&self, session_id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(session_id.to_string())
            .execute(self.db.pool())
            .await?;

        tracing::info!("Deleted session: {}", session_id);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}
