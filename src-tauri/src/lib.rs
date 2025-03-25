use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct Setting {
    id: Option<i64>,
    key: String,
    value: String,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn check_and_create_directory(path: &str) -> Result<bool, String> {
    let path = Path::new(path);
    if path.exists() {
        if path.is_dir() {
            return Ok(true);
        } else {
            return Err("Path exists but is not a directory".into());
        }
    }

    match fs::create_dir_all(path) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Failed to create directory: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Define migrations
    let migrations = vec![
        tauri_plugin_sql::Migration {
            version: 1,
            description: "create_settings_table",
            sql: "CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY AUTOINCREMENT, key TEXT UNIQUE, value TEXT);",
            kind: tauri_plugin_sql::MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:settings.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, check_and_create_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
