use std::fs;
use std::path::{Path, PathBuf};

use crate::constants;
use crate::docx::reader::read_docx;

pub struct UnpackResult {
    pub file_count: usize,
    pub output_dir: PathBuf,
}

fn is_forbidden_char(c: char) -> bool {
    constants::FORBIDDEN_CHARS.contains(&c)
}

fn replace_forbidden_chars(text: &str) -> String {
    text.chars()
        .map(|c| if is_forbidden_char(c) { '_' } else { c })
        .collect()
}

const WINDOWS_RESERVED_NAMES: &[&str] = constants::WINDOWS_RESERVED;

fn is_windows_reserved_name(name: &str) -> bool {
    // Strip extension for reserved name check (CON.txt → CON)
    // Only strip the LAST extension to avoid false matches on dotted names
    let stem = if let Some(dot) = name.rfind('.') {
        &name[..dot]
    } else {
        name
    };
    let upper = stem.to_uppercase();
    WINDOWS_RESERVED_NAMES.contains(&upper.as_str())
}

fn is_absolute_path(normalized: &str) -> bool {
    // Unix absolute: /foo
    if normalized.starts_with('/') {
        return true;
    }
    // Windows UNC path: \\server\share
    if normalized.starts_with("//") {
        return true;
    }
    let chars: Vec<char> = normalized.chars().collect();
    // Windows drive letter: X:  or  X:\
    if chars.len() >= 2 && chars[0].is_ascii_alphabetic() && chars[1] == ':' {
        if chars.len() == 2 || chars[2] == '/' {
            return true;
        }
    }
    // Windows device path: \\.\  or \\?\
    if chars.len() >= 4
        && chars[0] == '\\'
        && chars[1] == '\\'
        && (chars[2] == '.' || chars[2] == '?')
        && chars[3] == '\\'
    {
        return true;
    }
    false
}

fn validate_heading_path(text: &str) -> Result<PathBuf, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Empty heading path".into());
    }

    let normalized = trimmed.replace(constants::BACKSLASH, constants::FORWARD_SLASH);

    if is_absolute_path(&normalized) {
        return Err(format!("Absolute path not allowed: {}", trimmed));
    }

    for component in normalized.split(constants::FORWARD_SLASH) {
        if component == ".." {
            return Err("Path traversal (..) is not allowed".into());
        }
        if is_windows_reserved_name(component) {
            return Err(format!("Windows reserved name: {}", component));
        }
    }

    let replaced = replace_forbidden_chars(&normalized);
    Ok(PathBuf::from(replaced))
}

pub fn default_output_path(input: &Path) -> PathBuf {
    let mut p = input.to_path_buf();
    p.set_extension("");
    PathBuf::from(format!(
        "{}{}",
        p.to_string_lossy(),
        constants::UNPACKED_SUFFIX
    ))
}

pub fn resolve_unpack_output(output: &Path) -> PathBuf {
    if !output.exists() {
        return output.to_path_buf();
    }
    let dir = output.parent().unwrap_or(Path::new("."));
    let stem = output
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    for i in 1..=9999u32 {
        let candidate = dir.join(format!("{}_{}", stem, i));
        if !candidate.exists() {
            return candidate;
        }
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    dir.join(format!("{}_{}", stem, ts))
}

pub fn unpack_docx(input: &Path, output: &Path) -> Result<UnpackResult, String> {
    let doc = read_docx(std::fs::File::open(input).map_err(|e| format!("Open input: {}", e))?)
        .map_err(|e| format!("Read DOCX: {}", e))?;

    let mut file_count = 0;

    // Create output directory
    fs::create_dir_all(output).map_err(|e| format!("Create output dir: {}", e))?;

    // For each paragraph, if it's a heading, create a file
    let mut current_heading: Option<String> = None;
    let mut current_content = String::new();

    for paragraph in &doc.paragraphs {
        if paragraph.is_heading() {
            // Save previous file if heading is non-empty
            if let Some(ref heading) = current_heading {
                if !heading.is_empty() {
                    let content = current_content.trim_end_matches('\n').to_string();
                    let heading_path = validate_heading_path(heading)?;
                    let full_path = output.join(&heading_path);
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| format!("Create parent dir: {}", e))?;
                    }
                    fs::write(&full_path, &content).map_err(|e| format!("Write file: {}", e))?;
                    file_count += 1;
                }
            }

            current_heading = Some(
                paragraph
                    .runs
                    .first()
                    .map(|r| r.text.clone())
                    .unwrap_or_default(),
            );
            current_content = String::new();
        } else {
            for run in &paragraph.runs {
                current_content.push_str(&run.text);
            }
            current_content.push('\n');
        }
    }

    // Save last file
    if let Some(ref heading) = current_heading {
        if !heading.is_empty() {
            let content = current_content.trim_end_matches('\n').to_string();
            let heading_path = validate_heading_path(heading)?;
            let full_path = output.join(&heading_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Create parent dir: {}", e))?;
            }
            fs::write(&full_path, &content).map_err(|e| format!("Write file: {}", e))?;
            file_count += 1;
        }
    }

    Ok(UnpackResult {
        file_count,
        output_dir: output.to_path_buf(),
    })
}
