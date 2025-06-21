use crate::model::Op;
use anyhow::Result;
use std::path::PathBuf;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use chrono::Utc;

pub fn get_data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".todo-graph")
}

pub fn get_ops_dir() -> PathBuf {
    get_data_dir().join("ops")
}

pub fn append_op(op: &Op) -> Result<()> {
    let ops_dir = get_ops_dir();
    create_dir_all(&ops_dir)?;
    
    let date = Utc::now().format("%Y-%m").to_string();
    let path = ops_dir.join(format!("{}.jsonl", date));
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    
    serde_json::to_writer(&file, op)?;
    writeln!(file)?;
    
    Ok(())
}

pub fn read_ops_for_month(year_month: &str) -> Result<Vec<Op>> {
    let path = get_ops_dir().join(format!("{}.jsonl", year_month));
    
    if !path.exists() {
        return Ok(Vec::new());
    }
    
    let content = std::fs::read_to_string(path)?;
    let mut ops = Vec::new();
    
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let op: Op = serde_json::from_str(line)?;
        ops.push(op);
    }
    
    Ok(ops)
}

pub fn read_all_ops() -> Result<Vec<Op>> {
    let ops_dir = get_ops_dir();
    if !ops_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut all_ops = Vec::new();
    
    for entry in std::fs::read_dir(ops_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                let mut ops = read_ops_for_month(filename)?;
                all_ops.append(&mut ops);
            }
        }
    }
    
    // Sort by creation time
    all_ops.sort_by(|a, b| {
        let time_a = match a {
            Op::AppendObject(obj) => obj.created,
            Op::AppendEdge(edge) => edge.created,
            Op::UpdateObject { .. } => Utc::now(), // Updates don't have timestamps in this version
        };
        let time_b = match b {
            Op::AppendObject(obj) => obj.created,
            Op::AppendEdge(edge) => edge.created,
            Op::UpdateObject { .. } => Utc::now(),
        };
        time_a.cmp(&time_b)
    });
    
    Ok(all_ops)
}
