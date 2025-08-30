use sqlx::{SqlitePool, Row, sqlite::SqliteConnectOptions};
use std::path::PathBuf;
use crate::widgets::Widget;

#[derive(Clone)]
pub struct InvestigationDB {
    pool: SqlitePool,
}

impl InvestigationDB {
    pub async fn create(file_path: &PathBuf, name: &str, description: &str, color: &[f32; 3]) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(file_path)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        let mut db = Self { pool };
        db.initialize(name, description, color).await?;
        Ok(db)
    }
    
    pub async fn open(file_path: &PathBuf) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(file_path)
            .create_if_missing(false);
        let pool = SqlitePool::connect_with(options).await?;
        let db = Self { pool };
        
        // Run migrations for existing databases
        sqlx::migrate!("./migrations/investigations").run(&db.pool).await?;
        
        Ok(db)
    }
    
    async fn initialize(&mut self, name: &str, description: &str, color: &[f32; 3]) -> Result<(), sqlx::Error> {
        // Run SQLx migrations for investigation database
        sqlx::migrate!("./migrations/investigations").run(&self.pool).await?;
        
        // Set initial metadata
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        let color_string = format!("{},{},{}", color[0], color[1], color[2]);
            
        sqlx::query(
            "INSERT INTO metadata (name, description, color_rgb, created_at, version) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(name)
        .bind(description)
        .bind(color_string)
        .bind(now)
        .bind("1.0")
        .execute(&self.pool).await?;
        
        Ok(())
    }
    
    
    pub async fn get_metadata(&self) -> Result<Option<(String, String, [f32; 3], i64, String)>, sqlx::Error> {
        let row = sqlx::query("SELECT name, description, color_rgb, created_at, version FROM metadata LIMIT 1")
            .fetch_optional(&self.pool).await?;
            
        match row {
            Some(row) => {
                let color_string = row.get::<String, _>("color_rgb");
                let color_parts: Vec<&str> = color_string.split(',').collect();
                let color = [
                    color_parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0.2),
                    color_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0.4),
                    color_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.85),
                ];
                
                Ok(Some((
                    row.get::<String, _>("name"),
                    row.get::<String, _>("description"),
                    color,
                    row.get::<i64, _>("created_at"),
                    row.get::<String, _>("version"),
                )))
            }
            None => Ok(None)
        }
    }
    
    pub async fn update_metadata(&self, name: &str, description: &str, color: &[f32; 3]) -> Result<(), sqlx::Error> {
        let color_string = format!("{},{},{}", color[0], color[1], color[2]);
        
        sqlx::query("UPDATE metadata SET name = ?, description = ?, color_rgb = ? WHERE rowid = 1")
            .bind(name)
            .bind(description)
            .bind(color_string)
            .execute(&self.pool).await?;
            
        Ok(())
    }
    
    pub async fn save_widget(&self, widget_id: i32, widget_version: i32, widget_type: &str, config_json: &str, 
                            pos_x: f32, pos_y: f32, size_x: f32, size_y: f32, collapsed: bool) -> Result<(), sqlx::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        sqlx::query(
            "INSERT OR REPLACE INTO widgets (id, version, widget_type, config_json, position_x, position_y, size_x, size_y, created_at, collapsed) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(widget_id)
        .bind(widget_version)
        .bind(widget_type)
        .bind(config_json)
        .bind(pos_x)
        .bind(pos_y)
        .bind(size_x)
        .bind(size_y)
        .bind(now)
        .bind(collapsed)
        .execute(&self.pool).await?;
        
        Ok(())
    }

    pub async fn record_raw_data(&self, widget_id: i32, widget_version: i32, line_content: &str, line_number: i32) -> Result<(), sqlx::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        sqlx::query("INSERT INTO raw_data (widget_id, widget_version, timestamp, line_content, line_number) VALUES (?, ?, ?, ?, ?)")
            .bind(widget_id)
            .bind(widget_version)
            .bind(now)
            .bind(line_content)
            .bind(line_number)
            .execute(&self.pool).await?;
            
        Ok(())
    }
    
    pub async fn load_widgets(&self) -> Result<Vec<(i32, i32, String, String, f32, f32, f32, f32, bool)>, sqlx::Error> {
        // Only load the latest version of each widget
        let rows = sqlx::query("
            SELECT id, version, widget_type, config_json, position_x, position_y, size_x, size_y, collapsed 
            FROM widgets w1
            WHERE archived_at IS NULL 
            AND version = (SELECT MAX(version) FROM widgets w2 WHERE w2.id = w1.id AND w2.archived_at IS NULL)
        ")
            .fetch_all(&self.pool).await?;
            
        let mut widgets = Vec::new();
        for row in rows {
            widgets.push((
                row.get::<i32, _>("id"),
                row.get::<i32, _>("version"),
                row.get::<String, _>("widget_type"),
                row.get::<String, _>("config_json"),
                row.get::<f32, _>("position_x"),
                row.get::<f32, _>("position_y"),
                row.get::<f32, _>("size_x"),
                row.get::<f32, _>("size_y"),
                row.get::<bool, _>("collapsed"),
            ));
        }
        
        Ok(widgets)
    }
    
    pub async fn load_widgets_at_time(&self, timestamp: i64) -> Result<Vec<(i32, i32, String, String, f32, f32, f32, f32, bool)>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, version, widget_type, config_json, position_x, position_y, size_x, size_y, collapsed FROM widgets WHERE created_at <= ? AND (archived_at IS NULL OR archived_at > ?)")
            .bind(timestamp)
            .bind(timestamp)
            .fetch_all(&self.pool).await?;
            
        let mut widgets = Vec::new();
        for row in rows {
            widgets.push((
                row.get::<i32, _>("id"),
                row.get::<i32, _>("version"),
                row.get::<String, _>("widget_type"),
                row.get::<String, _>("config_json"),
                row.get::<f32, _>("position_x"),
                row.get::<f32, _>("position_y"),
                row.get::<f32, _>("size_x"),
                row.get::<f32, _>("size_y"),
                row.get::<bool, _>("collapsed"),
            ));
        }
        
        Ok(widgets)
    }
    
    pub async fn save_widget_instance(&self, widget: &crate::widgets::WidgetType) -> Result<(), sqlx::Error> {
        let widget_id = widget.widget_id() as i32;
        let widget_version = widget.widget_version();
        let widget_type = widget.widget_type_name();
        let widget_json = serde_json::to_string(widget).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;
        
        self.save_widget(widget_id, widget_version, widget_type, &widget_json, 0.0, 0.0, 600.0, 400.0, false).await
    }
    
    pub async fn load_widget_instances(&self) -> Result<Vec<crate::widgets::WidgetType>, Box<dyn std::error::Error>> {
        use crate::widgets::WidgetType;
        
        let widgets_data = self.load_widgets().await?;
        let mut widgets = Vec::new();
        
        for (_widget_id, _widget_version, widget_type, widget_json, _pos_x, _pos_y, _size_x, _size_y, _collapsed) in widgets_data {
            match serde_json::from_str::<WidgetType>(&widget_json) {
                Ok(widget) => widgets.push(widget),
                Err(e) => {
                    eprintln!("Failed to load widget {}: {}", widget_type, e);
                    continue; // Skip invalid widgets
                }
            }
        }
        
        Ok(widgets)
    }
    
    pub async fn load_widget_instances_at_time(&self, timestamp: i64) -> Result<Vec<crate::widgets::WidgetType>, Box<dyn std::error::Error>> {
        use crate::widgets::WidgetType;
        
        let widgets_data = self.load_widgets_at_time(timestamp).await?;
        let mut widgets = Vec::new();
        
        for (_widget_id, _widget_version, widget_type, widget_json, _pos_x, _pos_y, _size_x, _size_y, _collapsed) in widgets_data {
            match serde_json::from_str::<WidgetType>(&widget_json) {
                Ok(widget) => widgets.push(widget),
                Err(e) => {
                    eprintln!("Failed to load widget {}: {}", widget_type, e);
                    continue; // Skip invalid widgets
                }
            }
        }
        
        Ok(widgets)
    }
    
    pub async fn archive_widget(&self, widget_id: i32) -> Result<(), sqlx::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        sqlx::query("UPDATE widgets SET archived_at = ? WHERE id = ?")
            .bind(now)
            .bind(widget_id)
            .execute(&self.pool).await?;
        Ok(())
    }
    
    pub async fn archive_widget_instance(&self, widget: &crate::widgets::WidgetType) -> Result<(), sqlx::Error> {
        let widget_id = widget.widget_id() as i32;
        self.archive_widget(widget_id).await
    }
    
    pub async fn remove_widget(&self, widget_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM widgets WHERE id = ?")
            .bind(widget_id)
            .execute(&self.pool).await?;
        Ok(())
    }
    
    pub async fn remove_widget_instance(&self, widget: &crate::widgets::WidgetType) -> Result<(), sqlx::Error> {
        let widget_id = widget.widget_id() as i32;
        self.remove_widget(widget_id).await
    }
    
    pub async fn get_widget_data(&self, widget_id: i32, widget_version: i32) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query("SELECT line_content FROM raw_data WHERE widget_id = ? AND widget_version = ? ORDER BY line_number ASC")
            .bind(widget_id)
            .bind(widget_version)
            .fetch_all(&self.pool).await?;
            
        let mut lines = Vec::new();
        for row in rows {
            lines.push(row.get::<String, _>("line_content"));
        }
        
        Ok(lines)
    }
}