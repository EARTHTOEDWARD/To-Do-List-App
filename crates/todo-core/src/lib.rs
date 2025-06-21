pub mod model;
pub mod log;
pub mod db;

pub use model::{Op, ObjectV1, EdgeV1, Kind};
pub use log::{append_op, read_all_ops, get_data_dir};
pub use db::Database;

use anyhow::{Result, bail};

const MAX_TITLE_LEN: usize = 200;

/// Validate an operation before it's committed to the log/db.
fn validate_op(op: &Op) -> Result<()> {
    match op {
        Op::AppendObject(obj) => {
            match obj.kind {
                Kind::Task => {
                    let title = obj.payload["title"].as_str()
                        .ok_or_else(|| anyhow::anyhow!("Task title missing or not a string"))?;
                    let trimmed = title.trim();
                    if trimmed.is_empty() {
                        bail!("Task title cannot be empty");
                    }
                    if trimmed.len() > MAX_TITLE_LEN {
                        bail!("Task title too long (>{} chars)", MAX_TITLE_LEN);
                    }
                    if obj.payload["archived"].as_bool().is_none() {
                        bail!("Task archived field must be a boolean");
                    }
                    if let Some(priority) = obj.payload["priority"].as_str() {
                        if !["high", "medium", "low"].contains(&priority) {
                            bail!("Invalid priority value");
                        }
                    } else {
                        bail!("Task priority missing or not a string");
                    }
                }
                _ => {}
            }
        }
        Op::UpdateObject { .. } => {
            // For now no special validation on patches.
        }
        Op::AppendEdge(_) => {}
    }
    Ok(())
}

/// Apply an operation: write to log and update database
pub async fn apply_op(db: &Database, op: Op) -> Result<()> {
    // Validate before applying
    validate_op(&op)?;

    // First write to append-only log
    log::append_op(&op)?;
    
    // Then apply to SQLite database
    db.apply_op(&op).await?;
    
    Ok(())
}

/// Replay all operations from logs into database
pub async fn replay_logs(db: &Database) -> Result<()> {
    let ops = read_all_ops()?;
    
    for op in ops {
        db.apply_op(&op).await?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper to create an in-memory DB and set a temporary data dir for the op-log
    async fn setup() -> Database {
        // Point HOME to a temp dir so log files don't pollute real user dir
        let tmp_dir = TempDir::new().expect("create temp dir");
        std::env::set_var("HOME", tmp_dir.path());

        Database::new("sqlite::memory:").await.expect("create db")
    }

    #[tokio::test]
    async fn add_task_and_retrieve() {
        let db = setup().await;

        // Create and apply a new task
        let task = ObjectV1::new_task("Unit-test task", Some("Ensures add works"), "medium");
        let op = Op::AppendObject(task.clone());
        apply_op(&db, op).await.expect("apply op");

        // Load from DB
        let tasks = db.get_objects_by_kind(Kind::Task).await.expect("get tasks");
        assert_eq!(tasks.len(), 1);
        let fetched = &tasks[0];
        assert_eq!(fetched.payload["title"], "Unit-test task");
        assert!(!fetched.payload["completed"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn complete_task() {
        let db = setup().await;

        // Add a task
        let task = ObjectV1::new_task("Complete me", None, "medium");
        let id = task.id;
        apply_op(&db, Op::AppendObject(task)).await.expect("append");

        // Patch to completed
        let patch = serde_json::json!({ "completed": true });
        apply_op(&db, Op::UpdateObject { id, patch }).await.expect("update");

        // Verify
        let tasks = db.get_objects_by_kind(Kind::Task).await.expect("get tasks");
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].payload["completed"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn invalid_task_title_should_fail() {
        let db = setup().await;

        // Create a task with an empty title
        let task = ObjectV1::new_task("   ", None, "medium");
        let res = apply_op(&db, Op::AppendObject(task)).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn archive_task() {
        let db = setup().await;
        let task = ObjectV1::new_task("Archive me", None, "low");
        let id = task.id;
        apply_op(&db, Op::AppendObject(task)).await.expect("append");

        let patch = serde_json::json!({"archived": true});
        apply_op(&db, Op::UpdateObject { id, patch }).await.expect("archive patch");

        let tasks = db.get_objects_by_kind(Kind::Task).await.expect("get tasks");
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].payload["archived"].as_bool().unwrap());
    }
}
