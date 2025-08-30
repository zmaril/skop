use sqlx::{SqlitePool, Row, sqlite::SqliteConnectOptions};
use super::ensure_skop_dir;

pub struct MainDB {
    pub pool: SqlitePool,
}

impl MainDB {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let skop_dir = ensure_skop_dir().map_err(|e| 
            sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, 
                format!("Failed to create skop directory: {}", e))))?;
        
        let db_path = skop_dir.join("main.db");
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        
        let mut db = Self { pool };
        db.initialize().await?;
        Ok(db)
    }
    
    async fn initialize(&mut self) -> Result<(), sqlx::Error> {
        // Run SQLx migrations for main database
        sqlx::migrate!("./migrations/main").run(&self.pool).await?;
        Ok(())
    }
    
    pub async fn add_investigation(&self, file_path: &str) -> Result<i64, sqlx::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        let result = sqlx::query(
            "INSERT INTO investigations (file_path, created_at, last_accessed, archived) VALUES (?, ?, ?, 0)"
        )
        .bind(file_path)
        .bind(now)
        .bind(now)
        .execute(&self.pool).await?;
        
        Ok(result.last_insert_rowid())
    }
    
    pub async fn list_investigations(&self) -> Result<Vec<(i64, String, i64, i64)>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, file_path, created_at, last_accessed 
             FROM investigations WHERE archived = 0 ORDER BY last_accessed DESC"
        ).fetch_all(&self.pool).await?;
        
        let mut investigations = Vec::new();
        for row in rows {
            investigations.push((
                row.get::<i64, _>("id"),
                row.get::<String, _>("file_path"),
                row.get::<i64, _>("created_at"),
                row.get::<i64, _>("last_accessed"),
            ));
        }
        
        Ok(investigations)
    }
    
    pub async fn archive_investigation(&self, investigation_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE investigations SET archived = 1 WHERE id = ?")
            .bind(investigation_id)
            .execute(&self.pool).await?;
        Ok(())
    }
    
    pub async fn update_last_accessed(&self, investigation_id: i64) -> Result<(), sqlx::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        let result = sqlx::query("UPDATE investigations SET last_accessed = ? WHERE id = ?")
            .bind(now)
            .bind(investigation_id)
            .execute(&self.pool).await?;
        
        if result.rows_affected() == 0 {
            eprintln!("WARNING: No investigation found with ID {} to update last_accessed time", investigation_id);
        } else {
            println!("Successfully updated last_accessed for investigation ID {}", investigation_id);
        }
        
        Ok(())
    }
    
}