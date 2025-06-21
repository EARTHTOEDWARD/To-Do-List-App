use crate::model::{Op, ObjectV1, EdgeV1, Kind};
use anyhow::Result;
use sqlx::{SqlitePool, Row};
use ulid::Ulid;
use chrono::{DateTime, Utc};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        use sqlx::sqlite::SqliteConnectOptions;
        use std::str::FromStr;
        
        // Parse the database URL and set create_if_missing
        let mut options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);
        
        let pool = SqlitePool::connect_with(options).await?;
        
        // Create tables if they don't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS objects (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                payload JSON NOT NULL,
                created INTEGER NOT NULL,
                updated INTEGER
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS edges (
                id TEXT PRIMARY KEY,
                from_id TEXT NOT NULL,
                to_id TEXT NOT NULL,
                typ TEXT NOT NULL,
                created INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn apply_op(&self, op: &Op) -> Result<()> {
        match op {
            Op::AppendObject(obj) => self.insert_object(obj).await?,
            Op::AppendEdge(edge) => self.insert_edge(edge).await?,
            Op::UpdateObject { id, patch } => self.update_object(*id, patch).await?,
        }
        Ok(())
    }

    async fn insert_object(&self, obj: &ObjectV1) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO objects (id, kind, payload, created, updated)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(obj.id.to_string())
        .bind(format!("{:?}", obj.kind))
        .bind(obj.payload.to_string())
        .bind(obj.created.timestamp())
        .bind(obj.updated.map(|u| u.timestamp()))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn insert_edge(&self, edge: &EdgeV1) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO edges (id, from_id, to_id, typ, created)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(edge.id.to_string())
        .bind(edge.from.to_string())
        .bind(edge.to.to_string())
        .bind(&edge.typ)
        .bind(edge.created.timestamp())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_object(&self, id: Ulid, patch: &serde_json::Value) -> Result<()> {
        // Get current object
        let row = sqlx::query("SELECT payload FROM objects WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let mut current_payload: serde_json::Value = 
                serde_json::from_str(row.get::<String, _>("payload").as_str())?;
            
            // Merge patch into current payload
            if let serde_json::Value::Object(patch_map) = patch {
                if let serde_json::Value::Object(ref mut current_map) = current_payload {
                    for (key, value) in patch_map {
                        current_map.insert(key.clone(), value.clone());
                    }
                }
            }

            // Update in database
            sqlx::query(
                "UPDATE objects SET payload = ?, updated = ? WHERE id = ?"
            )
            .bind(current_payload.to_string())
            .bind(Utc::now().timestamp())
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_objects_by_kind(&self, kind: Kind) -> Result<Vec<ObjectV1>> {
        let rows = sqlx::query(
            "SELECT id, kind, payload, created, updated FROM objects WHERE kind = ?"
        )
        .bind(format!("{:?}", kind))
        .fetch_all(&self.pool)
        .await?;

        let mut objects = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let payload: String = row.get("payload");
            let created: i64 = row.get("created");
            let updated: Option<i64> = row.get("updated");

            objects.push(ObjectV1 {
                id: Ulid::from_string(&id)?,
                kind,
                payload: serde_json::from_str(&payload)?,
                created: DateTime::from_timestamp(created, 0).unwrap(),
                updated: updated.map(|u| DateTime::from_timestamp(u, 0).unwrap()),
            });
        }

        Ok(objects)
    }

    pub async fn get_all_objects(&self) -> Result<Vec<ObjectV1>> {
        let rows = sqlx::query(
            "SELECT id, kind, payload, created, updated FROM objects ORDER BY created DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut objects = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let kind_str: String = row.get("kind");
            let payload: String = row.get("payload");
            let created: i64 = row.get("created");
            let updated: Option<i64> = row.get("updated");

            let kind = match kind_str.as_str() {
                "Task" => Kind::Task,
                "Chat" => Kind::Chat,
                "Doc" => Kind::Doc,
                "Commit" => Kind::Commit,
                _ => continue,
            };

            objects.push(ObjectV1 {
                id: Ulid::from_string(&id)?,
                kind,
                payload: serde_json::from_str(&payload)?,
                created: DateTime::from_timestamp(created, 0).unwrap(),
                updated: updated.map(|u| DateTime::from_timestamp(u, 0).unwrap()),
            });
        }

        Ok(objects)
    }

    pub async fn get_edges_for_object(&self, object_id: Ulid) -> Result<Vec<EdgeV1>> {
        let rows = sqlx::query(
            "SELECT id, from_id, to_id, typ, created FROM edges WHERE from_id = ? OR to_id = ?"
        )
        .bind(object_id.to_string())
        .bind(object_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut edges = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let from_id: String = row.get("from_id");
            let to_id: String = row.get("to_id");
            let typ: String = row.get("typ");
            let created: i64 = row.get("created");

            edges.push(EdgeV1 {
                id: Ulid::from_string(&id)?,
                from: Ulid::from_string(&from_id)?,
                to: Ulid::from_string(&to_id)?,
                typ,
                created: DateTime::from_timestamp(created, 0).unwrap(),
            });
        }

        Ok(edges)
    }
}
