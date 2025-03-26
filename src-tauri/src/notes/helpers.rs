// src/notes/helpers.rs (updated version)
use regex::Regex;
use serde_json::{json, Value as JsonValue};
use serde_yaml::Value as YamlValue;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn find_backlinks(relative_path: &str, vault_directory: &str) -> Vec<(String, String)> {
    println!(
        "Finding backlinks for path: '{}' in vault: '{}'",
        relative_path, vault_directory
    );

    // Try using the ripgrep method first
    if let Some(links) = try_ripgrep_search(relative_path, vault_directory) {
        return links;
    }

    // If ripgrep failed, use the manual search fallback
    println!("Using fallback method to find backlinks");
    let backlinks = find_backlinks_fallback(relative_path, vault_directory);

    println!("Fallback method found {} backlinks", backlinks.len());
    backlinks
}

fn try_ripgrep_search(relative_path: &str, vault_directory: &str) -> Option<Vec<(String, String)>> {
    let mut all_matches: Vec<(String, String)> = Vec::new();

    // Get the title of the target note
    let title = get_title(None, Some(relative_path), Some(vault_directory));
    println!("Target note title: '{}'", title);

    // Create variations of the path to search for
    let path_variants = vec![
        relative_path.to_string(),
        if relative_path.ends_with(".md") {
            relative_path.trim_end_matches(".md").to_string()
        } else {
            String::new()
        },
        if relative_path.contains('/') {
            let parts: Vec<&str> = relative_path.split('/').collect();
            if let Some(last_part) = parts.last() {
                last_part.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        },
        if relative_path.contains('/') && relative_path.ends_with(".md") {
            let parts: Vec<&str> = relative_path.split('/').collect();
            if let Some(last_part) = parts.last() {
                last_part.trim_end_matches(".md").to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        },
    ];

    // Filter out empty variants
    let path_variants: Vec<String> = path_variants
        .into_iter()
        .filter(|v| !v.is_empty())
        .collect();

    println!("Will search for these path variants: {:?}", path_variants);

    for variant in path_variants {
        // Create pattern that handles both [[path]] and [[path | alias]] formats
        // with optional whitespace around the pipe
        let pattern = format!(r"\[\[{}(?:\s*\|[^\]]+)?\]\]", regex::escape(&variant));

        println!("Trying ripgrep with pattern: '{}'", pattern);

        let output = Command::new("rg")
            .arg("--files-with-matches")
            .arg("--no-heading")
            .arg("-U") // Multiline mode
            .arg("--glob")
            .arg("*.md")
            .arg(&pattern)
            .arg(vault_directory)
            .output();

        match output {
            Ok(o) => {
                // ripgrep returns exit code 1 when no matches found - that's normal
                if !o.status.success() && o.status.code() != Some(1) {
                    println!(
                        "ripgrep command failed with unexpected exit code: {:?}",
                        o.status.code()
                    );
                    println!("stderr: {}", String::from_utf8_lossy(&o.stderr));
                    continue;
                }

                let output_str = String::from_utf8_lossy(&o.stdout);

                // Process each line which represents a matching file
                for line in output_str.lines() {
                    let file_path = Path::new(line.trim());
                    if !file_path.exists() {
                        continue;
                    }

                    println!("Found matching file: {}", file_path.display());

                    if let Some(file_rel_path) = get_relative_path(file_path, vault_directory) {
                        // Skip the target file itself
                        if file_rel_path != relative_path {
                            let file_title = get_title(
                                Some(line.trim()),
                                Some(&file_rel_path),
                                Some(vault_directory),
                            );

                            println!(
                                "Adding backlink from ripgrep: {} - {}",
                                file_rel_path, file_title
                            );
                            all_matches.push((file_rel_path, file_title));
                        }
                    }
                }
            }
            Err(e) => {
                println!("ripgrep command failed to execute: {}", e);
                return None; // Abort ripgrep method completely
            }
        }
    }

    // Deduplicate results
    all_matches.sort_by(|a, b| a.0.cmp(&b.0));
    all_matches.dedup();

    if all_matches.is_empty() {
        None
    } else {
        Some(all_matches)
    }
}

// Helper to get relative path from absolute path
fn get_relative_path(file_path: &Path, vault_directory: &str) -> Option<String> {
    let vault_path = Path::new(vault_directory);
    match file_path.strip_prefix(vault_path) {
        Ok(rel_path) => Some(rel_path.to_string_lossy().to_string()),
        Err(_) => None,
    }
}

/// Fallback implementation without ripgrep - slower but more reliable
fn find_backlinks_fallback(relative_path: &str, vault_directory: &str) -> Vec<(String, String)> {
    let mut backlinks = Vec::new();

    // Get all notes in the vault
    let all_notes = get_all_notes(vault_directory);
    println!("Scanning {} notes for backlinks", all_notes.len());

    // Create multiple patterns to try
    let patterns = vec![
        // Standard pattern [[path]]
        format!(r"\[\[{}\]\]", regex::escape(relative_path)),
        // Aliased pattern [[path|alias]]
        format!(r"\[\[{}\|", regex::escape(relative_path)),
    ];

    // Also try without extension if it has one
    let no_ext_patterns = if relative_path.ends_with(".md") {
        let no_ext = relative_path.trim_end_matches(".md");
        vec![
            format!(r"\[\[{}\]\]", regex::escape(no_ext)),
            format!(r"\[\[{}\|", regex::escape(no_ext)),
        ]
    } else {
        Vec::new()
    };

    // Also try just the filename without path if it's in a subfolder
    let filename_patterns = if relative_path.contains('/') {
        let parts: Vec<&str> = relative_path.split('/').collect();
        if let Some(last_part) = parts.last() {
            vec![
                format!(r"\[\[{}\]\]", regex::escape(last_part)),
                format!(r"\[\[{}\|", regex::escape(last_part)),
            ]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Combine all patterns
    let all_patterns: Vec<Regex> = patterns
        .into_iter()
        .chain(no_ext_patterns.into_iter())
        .chain(filename_patterns.into_iter())
        .filter_map(|p| Regex::new(&p).ok())
        .collect();

    println!("Using {} regex patterns for matching", all_patterns.len());

    for (abs_path, rel_path) in all_notes {
        // Skip the target file itself
        if rel_path == relative_path {
            continue;
        }

        // Read file content
        if let Ok(content) = read_file_content(&Path::new(&abs_path)) {
            // Check if this file has any matches with any pattern
            let has_match = all_patterns.iter().any(|regex| regex.is_match(&content));

            if has_match {
                let title = get_title(Some(&abs_path), Some(&rel_path), Some(vault_directory));

                println!("Adding backlink from fallback: {} - {}", rel_path, title);
                backlinks.push((rel_path, title));
            }
        }
    }

    // Remove duplicates
    backlinks.sort_by(|a, b| a.0.cmp(&b.0));
    backlinks.dedup();

    backlinks
}

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
