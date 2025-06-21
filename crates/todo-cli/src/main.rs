use clap::{Parser, Subcommand};
use anyhow::Result;
use todo_core::{Database, apply_op, Op, ObjectV1, Kind, get_data_dir, replay_logs};

#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "Graph-OS To-Do CLI - Local-first task management")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add {
        /// Task title
        title: String,
        /// Task description
        #[arg(short, long)]
        description: Option<String>,
        /// Priority: high | medium | low
        #[arg(short, long, default_value = "medium")]
        priority: String,
    },
    /// List tasks
    Ls {
        /// Show completed tasks
        #[arg(short, long)]
        all: bool,
    },
    /// Mark task as complete
    Complete {
        /// Task ID (or partial match)
        id: String,
    },
    /// Archive a task
    Archive {
        /// Task ID (or partial match)
        id: String,
    },
    /// Initialize the todo database
    Init,
    /// Show database statistics
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let data_dir = get_data_dir();
    
    // Ensure data directory exists
    std::fs::create_dir_all(&data_dir)?;
    
    let db_path = data_dir.join("graph.db");
    let db_url = format!("sqlite://{}", db_path.display());
    
    let db = Database::new(&db_url).await?;
    
    match cli.command {
        Commands::Init => {
            println!("🚀 Initializing Graph-OS todo system...");
            println!("   Data directory: {}", data_dir.display());
            println!("   Database URL: {}", db_url);
            println!("✅ Database created successfully!");
            println!("   Try: todo add \"My first task\"");
        },
        _ => {
            // Replay logs on startup for other commands to ensure DB is up to date
            replay_logs(&db).await?;
        }
    }
    
    match cli.command {
        Commands::Init => {
            // Already handled above
        },
        Commands::Add { title, description, priority } => {
            replay_logs(&db).await?;
            let task = ObjectV1::new_task(&title, description.as_deref(), &priority);
            let op = Op::AppendObject(task.clone());
            
            apply_op(&db, op).await?;
            
            println!("✅ Added task: {}", title);
            println!("   ID: {}", task.id);
            if let Some(desc) = description {
                println!("   Description: {}", desc);
            }
        },
        
        Commands::Ls { all } => {
            replay_logs(&db).await?;
            let tasks = db.get_objects_by_kind(Kind::Task).await?;
            
            if tasks.is_empty() {
                println!("No tasks found. Add one with: todo add \"Your task\"");
                return Ok(());
            }
            
            println!("📋 Tasks:");
            for task in tasks {
                let payload = &task.payload;
                let title = payload["title"].as_str().unwrap_or("Untitled");
                let completed = payload["completed"].as_bool().unwrap_or(false);
                let archived = payload["archived"].as_bool().unwrap_or(false);
                let priority = payload["priority"].as_str().unwrap_or("medium");
                
                if !all && (completed || archived) {
                    continue;
                }
                
                let status = if completed { "✅" } else if archived { "📦" } else { "⏳" };
                let priority_emoji = match priority {
                    "high" => "🔴",
                    "medium" => "🟡",
                    "low" => "🟢",
                    _ => "⚪",
                };
                
                println!("  {} {} [{}] {}", status, priority_emoji, &task.id.to_string()[0..8], title);
                
                if let Some(description) = payload["description"].as_str() {
                    println!("    📝 {}", description);
                }
                
                println!("    🕐 Created: {}", task.created.format("%Y-%m-%d %H:%M"));
            }
        },
        
        Commands::Complete { id } => {
            replay_logs(&db).await?;
            let tasks = db.get_objects_by_kind(Kind::Task).await?;
            
            // Find task by ID prefix
            let matching_task = tasks.iter().find(|task| {
                task.id.to_string().starts_with(&id) || 
                task.payload["title"].as_str().unwrap_or("").contains(&id)
            });
            
            if let Some(task) = matching_task {
                let patch = serde_json::json!({
                    "completed": true
                });
                
                let op = Op::UpdateObject {
                    id: task.id,
                    patch,
                };
                
                apply_op(&db, op).await?;
                
                let title = task.payload["title"].as_str().unwrap_or("Untitled");
                println!("✅ Completed task: {}", title);
            } else {
                println!("❌ Task not found: {}", id);
                println!("   Use 'todo ls' to see available tasks");
            }
        },
        
        Commands::Archive { id } => {
            replay_logs(&db).await?;
            let tasks = db.get_objects_by_kind(Kind::Task).await?;
            
            // Find task by ID prefix
            let matching_task = tasks.iter().find(|task| {
                task.id.to_string().starts_with(&id) || 
                task.payload["title"].as_str().unwrap_or("").contains(&id)
            });
            
            if let Some(task) = matching_task {
                let patch = serde_json::json!({
                    "archived": true
                });
                
                let op = Op::UpdateObject {
                    id: task.id,
                    patch,
                };
                
                apply_op(&db, op).await?;
                
                let title = task.payload["title"].as_str().unwrap_or("Untitled");
                println!("✅ Archived task: {}", title);
            } else {
                println!("❌ Task not found: {}", id);
                println!("   Use 'todo ls' to see available tasks");
            }
        },
        
        Commands::Init => {
            println!("🚀 Initializing Graph-OS todo system...");
            println!("   Data directory: {}", data_dir.display());
            println!("   Database URL: {}", db_url);
            
            // Check if directory exists and is writable
            match std::fs::metadata(&data_dir) {
                Ok(metadata) => {
                    println!("   Directory exists: {}", metadata.is_dir());
                }
                Err(e) => {
                    println!("   Directory error: {}", e);
                }
            }
            
            // Try to create the database
            match Database::new(&db_url).await {
                Ok(_) => {
                    println!("✅ Database created successfully!");
                    println!("   Try: todo add \"My first task\"");
                }
                Err(e) => {
                    println!("❌ Database creation failed: {}", e);
                    return Err(e);
                }
            }
        },
        
        Commands::Stats => {
            replay_logs(&db).await?;
            let all_objects = db.get_all_objects().await?;
            let tasks = all_objects.iter().filter(|obj| matches!(obj.kind, Kind::Task)).count();
            let chats = all_objects.iter().filter(|obj| matches!(obj.kind, Kind::Chat)).count();
            let docs = all_objects.iter().filter(|obj| matches!(obj.kind, Kind::Doc)).count();
            let commits = all_objects.iter().filter(|obj| matches!(obj.kind, Kind::Commit)).count();
            
            let completed_tasks = all_objects.iter()
                .filter(|obj| matches!(obj.kind, Kind::Task))
                .filter(|obj| obj.payload["completed"].as_bool().unwrap_or(false))
                .count();
            
            println!("📊 Graph-OS Statistics:");
            println!("   📋 Tasks: {} ({} completed)", tasks, completed_tasks);
            println!("   💬 Chats: {}", chats);
            println!("   📄 Docs: {}", docs);
            println!("   🔗 Commits: {}", commits);
            println!("   📦 Total objects: {}", all_objects.len());
            println!("   📁 Data directory: {}", data_dir.display());
        },
    }
    
    Ok(())
}
