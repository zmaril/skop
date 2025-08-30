use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::database::{main_db::MainDB, investigation_db::InvestigationDB};

pub const COLORS: &[(&str, [f32; 3])] = &[
    ("Red", [0.85, 0.2, 0.2]), ("Blue", [0.2, 0.4, 0.85]), ("Green", [0.2, 0.7, 0.3]), 
    ("Yellow", [0.9, 0.8, 0.2]), ("Orange", [0.9, 0.5, 0.1]), ("Purple", [0.6, 0.3, 0.8]), 
    ("Pink", [0.9, 0.4, 0.7]), ("Violet", [0.5, 0.2, 0.8]), ("Crimson", [0.8, 0.1, 0.3]), 
    ("Azure", [0.0, 0.5, 1.0]), ("Coral", [1.0, 0.5, 0.3]), ("Gold", [1.0, 0.8, 0.0]), 
    ("Silver", [0.7, 0.7, 0.7]), ("Emerald", [0.3, 0.8, 0.5]), ("Ruby", [0.7, 0.1, 0.1]), 
    ("Amber", [1.0, 0.7, 0.0]), ("Jade", [0.0, 0.7, 0.4]), ("Cyan", [0.0, 0.8, 0.8]), 
    ("Lime", [0.6, 1.0, 0.2]), ("Indigo", [0.3, 0.0, 0.5])
];

const ANIMALS: &[&str] = &[
    "Tiger", "Eagle", "Wolf", "Bear", "Lion", "Shark", "Panther", "Falcon", "Fox", "Lynx",
    "Cobra", "Raven", "Hawk", "Leopard", "Jaguar", "Viper", "Phoenix", "Dragon", "Stallion", "Owl",
    "Cat", "Dog", "Rabbit", "Turtle", "Penguin", "Octopus", "Whale", "Elephant", "Giraffe", "Zebra"
];

fn generate_random_name_and_color() -> (String, [f32; 3]) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    
    let now = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    let hash = hasher.finish();
    
    let (color_name, color_rgb) = COLORS[hash as usize % COLORS.len()];
    let animal_name = ANIMALS[(hash >> 8) as usize % ANIMALS.len()];
    
    let name = format!("{} {}", color_name, animal_name);
    (name, color_rgb)
}

pub fn find_color_name(color: [f32; 3]) -> Option<&'static str> {
    COLORS.iter()
        .find(|(_, rgb)| {
            // Compare with small tolerance for floating point precision
            (rgb[0] - color[0]).abs() < 0.01 &&
            (rgb[1] - color[1]).abs() < 0.01 &&
            (rgb[2] - color[2]).abs() < 0.01
        })
        .map(|(name, _)| *name)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investigation {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub file_path: PathBuf,
    pub created_at: i64,
    pub last_accessed: i64,
    pub color: [f32; 3],
}

impl Investigation {
    pub fn new_with_random_name() -> Self {
        let (name, color) = generate_random_name_and_color();
        let description = format!("Investigation: {}", name);
        Self::new(name, description, color)
    }
    
    pub fn new(name: String, description: String, color: [f32; 3]) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        let filename = format!("{}.skop", name.replace(" ", "_").to_lowercase());
        let file_path = crate::database::get_skop_dir().join(filename);
        
        Self {
            id: None,
            name,
            description,
            file_path,
            created_at: now,
            last_accessed: now,
            color,
        }
    }
    
    pub async fn create(&mut self, main_db: &MainDB) -> Result<(), sqlx::Error> {
        // Create the investigation database file with name, description, and color
        let _investigation_db = InvestigationDB::create(&self.file_path, &self.name, &self.description, &self.color).await?;
        
        // Add to main database registry (only file path and timestamps)
        let id = main_db.add_investigation(&self.file_path.to_string_lossy()).await?;
        self.id = Some(id);
        
        Ok(())
    }
    
    pub async fn load_all(main_db: &MainDB) -> Result<Vec<Investigation>, sqlx::Error> {
        let rows = main_db.list_investigations().await?;
        let mut investigations = Vec::new();
        
        for (id, file_path, created_at, last_accessed) in rows {
            let path_buf = PathBuf::from(&file_path);
            
            // Try to load metadata from the investigation file
            if let Ok(db) = InvestigationDB::open(&path_buf).await {
                if let Ok(Some((name, description, color, _created_at, _version))) = db.get_metadata().await {
                    investigations.push(Investigation {
                        id: Some(id),
                        name,
                        description,
                        file_path: path_buf,
                        created_at,
                        last_accessed,
                        color,
                    });
                }
            }
        }
        
        Ok(investigations)
    }
    
    pub async fn open(&self) -> Result<InvestigationDB, sqlx::Error> {
        InvestigationDB::open(&self.file_path).await
    }
    
    pub async fn load_metadata(&mut self) -> Result<(), sqlx::Error> {
        let db = self.open().await?;
        if let Some((name, description, color, _created_at, _version)) = db.get_metadata().await? {
            self.name = name;
            self.description = description;
            self.color = color;
        }
        Ok(())
    }
    
    pub async fn update_last_accessed(&self, main_db: &MainDB) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            main_db.update_last_accessed(id).await?;
        }
        Ok(())
    }
    
    pub async fn update_metadata(&self) -> Result<(), sqlx::Error> {
        let investigation_db = self.open().await?;
        investigation_db.update_metadata(&self.name, &self.description, &self.color).await
    }
    
    pub fn format_timestamp(timestamp: i64) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;
            
        let elapsed_micros = now - timestamp;
        let elapsed_secs = elapsed_micros / 1_000_000;
        
        let days = elapsed_secs / 86400;
        let hours = (elapsed_secs % 86400) / 3600;
        let mins = (elapsed_secs % 3600) / 60;
        
        if days > 0 {
            format!("{} days ago", days)
        } else if hours > 0 {
            format!("{} hours ago", hours)
        } else if mins > 0 {
            format!("{} minutes ago", mins)
        } else {
            "Just now".to_string()
        }
    }
    
    pub async fn delete(self, main_db: &MainDB) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            // Delete investigation file
            if self.file_path.exists() {
                let _ = std::fs::remove_file(&self.file_path);
            }
            
            // Remove from main database
            sqlx::query("DELETE FROM investigations WHERE id = ?")
                .bind(id)
                .execute(&main_db.pool).await?;
        }
        Ok(())
    }
    
    pub async fn archive(&self, main_db: &MainDB) -> Result<(), sqlx::Error> {
        if let Some(id) = self.id {
            main_db.archive_investigation(id).await?;
        }
        Ok(())
    }
}