// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::State;
use todo_core::{Database, apply_op, Op, ObjectV1, Kind, get_data_dir, replay_logs};
use std::sync::Mutex;

struct AppState {
    db: Mutex<Option<Database>>,
}

#[tauri::command]
async fn init_database() -> Result<String, String> {
    let data_dir = get_data_dir();
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    
    let db_path = data_dir.join("graph.db");
    let db_url = format!("sqlite://{}", db_path.display());
    
    let db = Database::new(&db_url).await.map_err(|e| e.to_string())?;
    replay_logs(&db).await.map_err(|e| e.to_string())?;
    
    Ok(format!("Database initialized at: {}", db_path.display()))
}

#[tauri::command]
async fn add_task(title: String, description: Option<String>, priority: Option<String>) -> Result<String, String> {
    let data_dir = get_data_dir();
    let db_path = data_dir.join("graph.db");
    let db_url = format!("sqlite://{}", db_path.display());
    
    let db = Database::new(&db_url).await.map_err(|e| e.to_string())?;
    
    let priority_val = priority.unwrap_or_else(|| "medium".to_string());
    let task = ObjectV1::new_task(&title, description.as_deref(), &priority_val);
    let task_id = task.id.to_string();
    let op = Op::AppendObject(task);
    
    apply_op(&db, op).await.map_err(|e| e.to_string())?;
    
    Ok(task_id)
}

#[tauri::command]
async fn get_tasks() -> Result<Vec<serde_json::Value>, String> {
    let data_dir = get_data_dir();
    let db_path = data_dir.join("graph.db");
    let db_url = format!("sqlite://{}", db_path.display());
    
    let db = Database::new(&db_url).await.map_err(|e| e.to_string())?;
    let tasks = db.get_objects_by_kind(Kind::Task).await.map_err(|e| e.to_string())?;
    
    let task_data: Vec<serde_json::Value> = tasks.into_iter().map(|task| {
        serde_json::json!({
            "id": task.id.to_string(),
            "kind": "Task",
            "payload": task.payload,
            "created": task.created.to_rfc3339(),
            "updated": task.updated.map(|u| u.to_rfc3339())
        })
    }).collect();
    
    Ok(task_data)
}

#[tauri::command]
async fn complete_task(task_id: String) -> Result<String, String> {
    let data_dir = get_data_dir();
    let db_path = data_dir.join("graph.db");
    let db_url = format!("sqlite://{}", db_path.display());
    
    let db = Database::new(&db_url).await.map_err(|e| e.to_string())?;
    
    let id = ulid::Ulid::from_string(&task_id).map_err(|e| e.to_string())?;
    let patch = serde_json::json!({
        "completed": true
    });
    
    let op = Op::UpdateObject { id, patch };
    apply_op(&db, op).await.map_err(|e| e.to_string())?;
    
    Ok("Task completed".to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            db: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            init_database,
            add_task,
            get_tasks,
            complete_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
