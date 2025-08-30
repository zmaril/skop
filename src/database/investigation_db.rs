use sqlx::{SqlitePool, Row, sqlite::SqliteConnectOptions};
use std::path::PathBuf;

pub struct InvestigationDB {
    pool: SqlitePool,
}

impl InvestigationDB {
    pub async fn create(file_path: &PathBuf, name: &str, description: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(file_path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        let mut db = Self { pool };
        db.initialize(name, description).await?;
        Ok(db)
    }
    
    pub async fn open(file_path: &PathBuf) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(file_path)
            .create_if_missing(false);
        let pool = SqlitePool::connect_with(options).await?;
        Ok(Self { pool })
    }
    
    async fn initialize(&mut self, name: &str, description: &str) -> Result<(), sqlx::Error> {
        // Investigation metadata
        sqlx::query(
            "CREATE TABLE investigation_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )"
        ).execute(&self.pool).await?;
        
        // Widget configurations and state
        sqlx::query(
            "CREATE TABLE widgets (
                id INTEGER PRIMARY KEY,
                widget_type TEXT NOT NULL,
                config_json TEXT NOT NULL,
                position_x REAL NOT NULL,
                position_y REAL NOT NULL,
                size_x REAL NOT NULL,
                size_y REAL NOT NULL,
                created_at INTEGER NOT NULL,
                active BOOLEAN DEFAULT 1
            )"
        ).execute(&self.pool).await?;
        
        // Raw data capture
        sqlx::query(
            "CREATE TABLE raw_data (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                widget_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                line_content TEXT NOT NULL,
                line_number INTEGER NOT NULL,
                FOREIGN KEY(widget_id) REFERENCES widgets(id)
            )"
        ).execute(&self.pool).await?;
        
        // Set initial metadata
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        self.set_metadata("name", name).await?;
        self.set_metadata("description", description).await?;
        self.set_metadata("created_at", &now.to_string()).await?;
        self.set_metadata("version", "1.0").await?;
        
        Ok(())
    }
    
    pub async fn set_metadata(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO investigation_meta (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(&self.pool).await?;
        Ok(())
    }
    
    pub async fn get_metadata(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query("SELECT value FROM investigation_meta WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool).await?;
            
        Ok(row.map(|r| r.get::<String, _>("value")))
    }
    
    pub async fn save_widget(&self, widget_id: i32, widget_type: &str, config_json: &str, 
                            pos_x: f32, pos_y: f32, size_x: f32, size_y: f32) -> Result<(), sqlx::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        sqlx::query(
            "INSERT OR REPLACE INTO widgets (id, widget_type, config_json, position_x, position_y, size_x, size_y, created_at, active) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1)"
        )
        .bind(widget_id)
        .bind(widget_type)
        .bind(config_json)
        .bind(pos_x)
        .bind(pos_y)
        .bind(size_x)
        .bind(size_y)
        .bind(now)
        .execute(&self.pool).await?;
        
        Ok(())
    }
    
    pub async fn log_data(&self, widget_id: i32, line_content: &str, line_number: i32) -> Result<(), sqlx::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        sqlx::query("INSERT INTO raw_data (widget_id, timestamp, line_content, line_number) VALUES (?, ?, ?, ?)")
            .bind(widget_id)
            .bind(now)
            .bind(line_content)
            .bind(line_number)
            .execute(&self.pool).await?;
            
        Ok(())
    }
}