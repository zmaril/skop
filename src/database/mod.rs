pub mod main_db;
pub mod investigation_db;

use std::path::PathBuf;

pub fn get_skop_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    if cfg!(debug_assertions) {
        PathBuf::from(home).join(".dev_skop")
    } else {
        PathBuf::from(home).join(".skop")
    }
}

pub fn ensure_skop_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let skop_dir = get_skop_dir();
    std::fs::create_dir_all(&skop_dir)?;
    Ok(skop_dir)
}