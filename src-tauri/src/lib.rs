// src-tauri/src/lib.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
mod notes; // Add this to import our new module
use notes::helpers; // Import the helpers module

// Define a struct to return note data to the frontend
#[derive(Debug, Serialize, Deserialize)]
struct NoteInfo {
    title: String,
    absolute_path: String,
    relative_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BacklinkInfo {
    title: String,
    relative_path: String,
}

#[tauri::command]
fn get_backlinks(relative_path: &str, vault_directory: &str) -> Result<Vec<BacklinkInfo>, String> {
    println!(
        "Tauri command get_backlinks called with: path={}, vault={}",
        relative_path, vault_directory
    );

    // Validate inputs
    if relative_path.is_empty() {
        return Err("Relative path is empty".into());
    }

    if vault_directory.is_empty() {
        return Err("Vault directory is empty".into());
    }

    // Check if vault directory exists
    if !Path::new(vault_directory).exists() {
        return Err(format!(
            "Vault directory does not exist: {}",
            vault_directory
        ));
    }

    // Get backlinks
    let backlinks = helpers::find_backlinks(relative_path, vault_directory);
    println!("Found {} backlinks", backlinks.len());

    // Convert to BacklinkInfo structs
    let result = backlinks
        .into_iter()
        .map(|(rel_path, title)| BacklinkInfo {
            title,
            relative_path: rel_path,
        })
        .collect();

    Ok(result)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Add a new command to get all notes
#[tauri::command]
fn get_notes(vault_directory: &str) -> Vec<NoteInfo> {
    let notes = helpers::get_all_notes(vault_directory);
    let mut result = Vec::new();

    for (absolute_path, relative_path) in notes {
        let title = helpers::get_title(
            Some(&absolute_path),
            Some(&relative_path),
            Some(vault_directory),
        );

        result.push(NoteInfo {
            title,
            absolute_path,
            relative_path,
        });
    }

    result
}

#[tauri::command]
fn get_note_content(relative_path: &str, vault_directory: &str) -> String {
    println!(
        "Called get_note_content with: {}, {}",
        relative_path, vault_directory
    );
    helpers::get_content(None, Some(relative_path), Some(vault_directory))
}

#[tauri::command]
fn get_note_title(relative_path: &str, vault_directory: &str) -> String {
    helpers::get_title(None, Some(relative_path), Some(vault_directory))
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

#[tauri::command]
fn update_note_content(
    absolute_path: Option<&str>,
    relative_path: Option<&str>,
    vault_directory: &str,
    new_content: &str,
) -> Result<bool, String> {
    match helpers::update_note(
        absolute_path,
        relative_path,
        Some(vault_directory),
        new_content,
    ) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Failed to update note: {}", e)),
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
        .invoke_handler(tauri::generate_handler![
            greet,
            check_and_create_directory,
            get_notes,
            get_note_content, // Add our new function to get note content
            get_note_title,   // Add our new function to get note title
            update_note_content,
            get_backlinks
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
