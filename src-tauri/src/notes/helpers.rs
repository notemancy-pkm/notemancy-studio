// src/notes/helpers.rs (updated version)
use serde_json::{json, Value as JsonValue};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Updates the content of a markdown file while preserving its frontmatter
pub fn update_note(
    absolute_path: Option<&str>,
    relative_path: Option<&str>,
    vault_directory: Option<&str>,
    new_content: &str,
) -> Result<(), io::Error> {
    // First, resolve the path to the note
    let path = resolve_note_path(absolute_path, relative_path, vault_directory)?;

    // Read the current content of the file
    let current_content = read_file_content(&path)?;

    // Extract frontmatter from the current content
    let (frontmatter, _) = extract_frontmatter_and_content(&current_content);

    // Create the new content with frontmatter preserved
    let updated_content = if let Some(frontmatter) = frontmatter {
        format!("---\n{}\n---\n\n{}", frontmatter, new_content)
    } else {
        // If there was no frontmatter, just use the new content
        new_content.to_string()
    };

    // Write the updated content to the file
    fs::write(path, updated_content)?;

    Ok(())
}

/// Recursively scans a directory for markdown files and returns them as (absolute_path, relative_path) pairs
pub fn get_all_notes(vault_directory: &str) -> Vec<(String, String)> {
    let vault_path = Path::new(vault_directory);
    if !vault_path.exists() || !vault_path.is_dir() {
        return Vec::new();
    }

    let mut result = Vec::new();
    if let Err(_) = collect_markdown_files(vault_path, vault_path, &mut result) {
        return Vec::new();
    }
    result
}

/// Helper function to recursively collect markdown files
fn collect_markdown_files(
    base_dir: &Path,
    current_dir: &Path,
    result: &mut Vec<(String, String)>,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_markdown_files(base_dir, &path, result)?;
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "md" {
                    // Get absolute path
                    let absolute_path = match fs::canonicalize(&path) {
                        Ok(p) => p.to_string_lossy().to_string(),
                        Err(_) => path.to_string_lossy().to_string(),
                    };

                    // Calculate relative path
                    let relative_path = match path.strip_prefix(base_dir) {
                        Ok(rel_path) => rel_path.to_string_lossy().to_string(),
                        Err(_) => {
                            // Fallback: use the filename if we can't get the relative path
                            path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        }
                    };

                    result.push((absolute_path, relative_path));
                }
            }
        }
    }

    Ok(())
}

/// Resolves a path to a note, given either an absolute path or a relative path and the vault directory
pub fn resolve_note_path(
    absolute_path: Option<&str>,
    relative_path: Option<&str>,
    vault_directory: Option<&str>,
) -> Result<PathBuf, io::Error> {
    // Try absolute path first
    if let Some(abs_path) = absolute_path {
        let path = PathBuf::from(abs_path);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try relative path with vault directory
    if let Some(rel_path) = relative_path {
        if let Some(vault_dir) = vault_directory {
            let full_path = Path::new(vault_dir).join(rel_path);
            if full_path.exists() {
                return Ok(full_path);
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Vault directory must be provided with relative path",
            ));
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Note file not found or invalid path provided",
    ))
}

/// Reads the content of a file
pub fn read_file_content(path: &Path) -> Result<String, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Extracts frontmatter and content from a markdown file
pub fn extract_frontmatter_and_content(content: &str) -> (Option<String>, String) {
    // Check if content starts with frontmatter delimiter
    if content.starts_with("---") {
        // Find the end of the frontmatter (second occurrence of ---)
        if let Some(end_index) = content[3..].find("---") {
            let frontmatter = &content[3..(end_index + 3)].trim();
            let content_start = end_index + 6; // Skip past the second "---"

            if content_start < content.len() {
                let rest_content = &content[content_start..].trim_start();
                return (Some(frontmatter.to_string()), rest_content.to_string());
            }
        }
    }

    // No frontmatter found or invalid format
    (None, content.to_string())
}

/// Strips frontmatter from a markdown file's content
pub fn strip_frontmatter(
    absolute_path: Option<&str>,
    relative_path: Option<&str>,
    vault_directory: Option<&str>,
) -> String {
    match resolve_note_path(absolute_path, relative_path, vault_directory) {
        Ok(path) => match read_file_content(&path) {
            Ok(content) => {
                let (_, content_without_frontmatter) = extract_frontmatter_and_content(&content);
                content_without_frontmatter
            }
            Err(_) => String::new(),
        },
        Err(_) => String::new(),
    }
}

/// Extracts and parses the YAML frontmatter from a markdown file into a JSON object
pub fn get_frontmatter(
    absolute_path: Option<&str>,
    relative_path: Option<&str>,
    vault_directory: Option<&str>,
) -> Option<JsonValue> {
    match resolve_note_path(absolute_path, relative_path, vault_directory) {
        Ok(path) => {
            match read_file_content(&path) {
                Ok(content) => {
                    let (frontmatter, _) = extract_frontmatter_and_content(&content);

                    if let Some(frontmatter_content) = frontmatter {
                        match serde_yaml::from_str::<YamlValue>(&frontmatter_content) {
                            Ok(yaml_value) => {
                                // Convert YAML value to JSON value
                                Some(yaml_to_json(yaml_value))
                            }
                            Err(_) => {
                                // If we can't parse as YAML, return an empty object
                                Some(json!({}))
                            }
                        }
                    } else {
                        // No frontmatter found
                        None
                    }
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

/// Helper function to convert YAML Value to JSON Value
fn yaml_to_json(yaml: YamlValue) -> JsonValue {
    match yaml {
        YamlValue::Null => JsonValue::Null,
        YamlValue::Bool(b) => JsonValue::Bool(b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                JsonValue::Number(i.into())
            } else if let Some(f) = n.as_f64() {
                if let Some(n) = serde_json::Number::from_f64(f) {
                    JsonValue::Number(n)
                } else {
                    JsonValue::Null
                }
            } else {
                JsonValue::Null
            }
        }
        YamlValue::String(s) => JsonValue::String(s),
        YamlValue::Sequence(seq) => JsonValue::Array(seq.into_iter().map(yaml_to_json).collect()),
        YamlValue::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                if let Some(key) = k.as_str() {
                    obj.insert(key.to_string(), yaml_to_json(v));
                }
            }
            JsonValue::Object(obj)
        }
        YamlValue::Tagged(tagged) => {
            // For tagged values, just convert the value part and ignore the tag
            yaml_to_json(tagged.value)
        }
    }
}

/// Gets the title of a markdown file from its frontmatter or filename
pub fn get_title(
    absolute_path: Option<&str>,
    relative_path: Option<&str>,
    vault_directory: Option<&str>,
) -> String {
    // Try to get title from frontmatter
    if let Some(frontmatter) = get_frontmatter(absolute_path, relative_path, vault_directory) {
        if let Some(title) = frontmatter.get("title") {
            if let Some(title_str) = title.as_str() {
                return title_str.to_string();
            }
        }
    }

    // If no title in frontmatter, use filename
    if let Ok(path) = resolve_note_path(absolute_path, relative_path, vault_directory) {
        if let Some(filename) = path.file_stem() {
            if let Some(filename_str) = filename.to_str() {
                return filename_str.to_string();
            }
        }
    }

    // Fallback to empty string if we couldn't extract a title
    String::new()
}

/// Gets the content of a markdown file with frontmatter stripped
pub fn get_content(
    absolute_path: Option<&str>,
    relative_path: Option<&str>,
    vault_directory: Option<&str>,
) -> String {
    strip_frontmatter(absolute_path, relative_path, vault_directory)
}
