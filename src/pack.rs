use crate::ignore::ExcludeRules;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Progress {
    pub current: usize,
    pub total: usize,
    pub file: String,
    pub phase: ProgressPhase,
}

#[derive(Clone, PartialEq)]
pub enum ProgressPhase {
    Collecting,
    Reading,
    Writing,
    Done,
}

pub type ProgressFn = Box<dyn Fn(Progress) + Send>;

pub struct TextFiles {
    pub text_files: Vec<PathBuf>,
    pub binary_skipped: usize,
}

pub struct PackResult {
    pub file_count: usize,
    pub binary_skipped: usize,
    pub output_path: PathBuf,
}

fn read_text_file(path: &Path, local_encodings: &[String]) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Read {}: {}", path.display(), e))?;

    // Try UTF-8 first (check without clone)
    if let Ok(s) = std::str::from_utf8(&bytes) {
        return Ok(s.to_owned());
    }

    // Try each configured encoding
    for enc in local_encodings {
        if enc.eq_ignore_ascii_case("UTF-8") || enc.eq_ignore_ascii_case("utf8") {
            continue; // already tried
        }
        if let Some(encoding) = encoding_rs::Encoding::for_label(enc.as_bytes()) {
            let (coded, _, had_errors) = encoding.decode(&bytes);
            if !had_errors {
                return Ok(coded.into_owned());
            }
        }
    }

    // Fallback: lossy UTF-8 decode (always succeeds)
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_first_bytes(path: &Path, max_len: usize) -> Option<Vec<u8>> {
    let mut file = fs::File::open(path).ok()?;
    let mut buf = vec![0u8; max_len];
    let n = file.read(&mut buf).ok()?;
    buf.truncate(n);
    Some(buf)
}

fn is_utf16_no_bom(bytes: &[u8]) -> bool {
    if bytes.len() < 4 || bytes.len() % 2 != 0 {
        return false;
    }
    let pairs = bytes.len() / 2;
    if pairs < 2 {
        return false;
    }
    let mut le_score = 0usize;
    let mut be_score = 0usize;
    for i in 0..pairs.min(4096) {
        let lo = bytes[2 * i];
        let hi = bytes[2 * i + 1];
        if lo != 0 && hi == 0 {
            le_score += 1;
        } else if lo == 0 && hi != 0 {
            be_score += 1;
        }
    }
    let total = le_score + be_score;
    if total < pairs.min(4096) / 4 {
        return false;
    }

    let (enc, is_le) = if le_score > be_score * 5 {
        (encoding_rs::UTF_16LE, true)
    } else if be_score > le_score * 5 {
        (encoding_rs::UTF_16BE, false)
    } else {
        return false;
    };

    let (_, _, had_errors) = enc.decode(bytes);
    if had_errors {
        return false;
    }

    // Verify non-NULL bytes in data positions are mostly printable
    let check_len = bytes.len().min(4096);
    let printable = if is_le {
        (0..check_len / 2)
            .filter(|&i| {
                let c = bytes[2 * i];
                c != 0 && (c.is_ascii_graphic() || c.is_ascii_whitespace())
            })
            .count()
    } else {
        (0..check_len / 2)
            .filter(|&i| {
                let c = bytes[2 * i + 1];
                c != 0 && (c.is_ascii_graphic() || c.is_ascii_whitespace())
            })
            .count()
    };
    printable > check_len / 4
}

fn is_utf32_no_bom(bytes: &[u8]) -> bool {
    if bytes.len() < 8 || bytes.len() % 4 != 0 {
        return false;
    }
    let quads = bytes.len() / 4;
    if quads < 2 {
        return false;
    }

    let check = quads.min(2048);
    let mut le_score = 0usize;
    let mut be_score = 0usize;
    for i in 0..check {
        let b0 = bytes[4 * i];
        let b1 = bytes[4 * i + 1];
        let b2 = bytes[4 * i + 2];
        let b3 = bytes[4 * i + 3];
        // UTF-32LE: char, 0, 0, 0
        if b0 != 0 && b1 == 0 && b2 == 0 && b3 == 0 {
            le_score += 1;
        }
        // UTF-32BE: 0, 0, 0, char
        if b0 == 0 && b1 == 0 && b2 == 0 && b3 != 0 {
            be_score += 1;
        }
    }

    let total = le_score + be_score;
    total > check / 4 && (le_score > be_score * 3 || be_score > le_score * 3)
}

fn is_meaningful_text(s: &str) -> bool {
    let total = s.chars().count();
    if total == 0 {
        return true;
    }
    let bad = s
        .chars()
        .filter(|&c| {
            let u = c as u32;
            u == 0
                || u == 0xFFFD
                || (u <= 0x1F && !matches!(u, 0x09 | 0x0A | 0x0C | 0x0D))
                || u == 0x7F
                || (0x80..=0x9F).contains(&u)
                || (0xD800..=0xDFFF).contains(&u)
                || (0xFDD0..=0xFDEF).contains(&u)
                || (u & 0xFFFE) == 0xFFFE
        })
        .count();
    bad * 10 <= total
}

pub fn is_text_file(path: &Path, local_encodings: &[String]) -> bool {
    let bytes = match read_first_bytes(path, 8192) {
        Some(b) => b,
        None => return false,
    };
    if bytes.is_empty() {
        return true;
    }

    // 1. BOM detection: UTF-8/UTF-16/UTF-32 BOM means text
    if let Some((_, _)) = encoding_rs::Encoding::for_bom(&bytes) {
        return true;
    }

    // 2. Try BOM-less Unicode detection (before NULL byte rejection)
    if is_utf16_no_bom(&bytes) || is_utf32_no_bom(&bytes) {
        return true;
    }

    // 3. NULL byte check: any NUL byte means binary (unless BOM/Unicode)
    if bytes.contains(&0) {
        return false;
    }

    // 4. Try UTF-8 decode and check for meaningful text
    if let Ok(s) = std::str::from_utf8(&bytes) {
        if is_meaningful_text(s) {
            return true;
        }
    }

    // 5. Try each configured local encoding
    for enc in local_encodings {
        if enc.eq_ignore_ascii_case("UTF-8") || enc.eq_ignore_ascii_case("utf8") {
            continue;
        }
        if let Some(encoding) = encoding_rs::Encoding::for_label(enc.as_bytes()) {
            let (decoded, _, had_errors) = encoding.decode(&bytes);
            if !had_errors && is_meaningful_text(&decoded) {
                return true;
            }
        }
    }

    // 6. Final fallback: lossy UTF-8 decode (consistent with read_text_file)
    let lossy = String::from_utf8_lossy(&bytes);
    is_meaningful_text(&lossy)
}

pub fn collect_text_files(
    source: &Path,
    exclude_rules: &ExcludeRules,
    local_encodings: &[String],
    on_progress: Option<&ProgressFn>,
) -> Result<TextFiles, String> {
    let mut text_files = Vec::new();
    let mut binary_skipped = 0;

    if source.is_dir() {
        let entries: Vec<_> = walkdir::WalkDir::new(source)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        let total = entries.len();
        for (current, entry) in entries.into_iter().enumerate() {
            let rel_path = match entry.path().strip_prefix(source) {
                Ok(p) => p.to_path_buf(),
                Err(_) => continue,
            };
            if exclude_rules.is_excluded(&rel_path.to_string_lossy()) {
                continue;
            }
            if let Some(ref cb) = on_progress {
                cb(Progress {
                    current: current + 1,
                    total,
                    file: rel_path.to_string_lossy().to_string(),
                    phase: ProgressPhase::Collecting,
                });
            }
            if is_text_file(entry.path(), local_encodings) {
                text_files.push(rel_path);
            } else {
                binary_skipped += 1;
            }
        }
    } else {
        return Err(format!("Expect directory, got file: {}", source.display()));
    }

    Ok(TextFiles {
        text_files,
        binary_skipped,
    })
}

fn build_doc_from_files(files: &[(PathBuf, String)]) -> crate::docx::model::Document {
    let mut doc = crate::docx::model::Document::new();
    for (rel_path, content) in files {
        doc.add_heading(&rel_path.to_string_lossy().replace('\\', "/"));
        doc.add_text(content);
    }
    doc
}

fn write_docx_file(doc: &crate::docx::model::Document, output: &Path) -> Result<(), String> {
    let parent = output.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|e| format!("Create output dir {}: {}", parent.display(), e))?;
    crate::docx::writer::write_docx(
        std::io::BufWriter::new(
            fs::File::create(output).map_err(|e| format!("Create output: {}", e))?,
        ),
        doc,
    )
    .map_err(|e| format!("Write DOCX: {}", e))
}

pub fn pack_dir(
    source: &Path,
    output: &Path,
    exclude_rules: &ExcludeRules,
    local_encodings: &[String],
    on_progress: Option<&ProgressFn>,
) -> Result<PackResult, String> {
    let files = collect_text_files(source, exclude_rules, local_encodings, on_progress)?;

    let mut doc_files: Vec<(PathBuf, String)> = Vec::new();
    for (i, rel_path) in files.text_files.iter().enumerate() {
        let full_path = source.join(rel_path);
        let content = read_text_file(&full_path, local_encodings)?;
        if let Some(ref cb) = on_progress {
            cb(Progress {
                current: i + 1,
                total: files.text_files.len(),
                file: rel_path.to_string_lossy().to_string(),
                phase: ProgressPhase::Reading,
            });
        }
        doc_files.push((rel_path.clone(), content));
    }

    if let Some(ref cb) = on_progress {
        cb(Progress {
            current: 0,
            total: 0,
            file: String::new(),
            phase: ProgressPhase::Writing,
        });
    }
    let doc = build_doc_from_files(&doc_files);
    write_docx_file(&doc, output)?;

    if let Some(ref cb) = on_progress {
        cb(Progress {
            current: 0,
            total: 0,
            file: String::new(),
            phase: ProgressPhase::Done,
        });
    }

    Ok(PackResult {
        file_count: doc_files.len(),
        binary_skipped: files.binary_skipped,
        output_path: output.to_path_buf(),
    })
}

pub fn pack_files(
    sources: &[PathBuf],
    output: &Path,
    exclude_rules: &ExcludeRules,
    local_encodings: &[String],
    on_progress: Option<&ProgressFn>,
) -> Result<PackResult, String> {
    if sources.is_empty() {
        return Err("No sources".into());
    }

    if sources.len() == 1 && sources[0].is_dir() {
        return pack_dir(
            &sources[0],
            output,
            exclude_rules,
            local_encodings,
            on_progress,
        );
    }

    let mut all_files: Vec<(PathBuf, String)> = Vec::new();
    let mut binary_skipped = 0;

    for source in sources {
        if source.is_dir() {
            let files = collect_text_files(source, exclude_rules, local_encodings, on_progress)?;
            for rel_path in &files.text_files {
                let full_path = source.join(rel_path);
                let content = read_text_file(&full_path, local_encodings)?;
                all_files.push((rel_path.clone(), content));
            }
            binary_skipped += files.binary_skipped;
        } else {
            if let Some(ref cb) = on_progress {
                cb(Progress {
                    current: all_files.len() + 1,
                    total: sources.len(),
                    file: source.to_string_lossy().to_string(),
                    phase: ProgressPhase::Reading,
                });
            }
            if !is_text_file(source, local_encodings) {
                binary_skipped += 1;
                continue;
            }
            let content = read_text_file(source, local_encodings)?;
            let name = source
                .file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| source.to_path_buf());
            all_files.push((name, content));
        }
    }

    if let Some(ref cb) = on_progress {
        cb(Progress {
            current: 0,
            total: 0,
            file: String::new(),
            phase: ProgressPhase::Writing,
        });
    }

    let mut seen = std::collections::HashSet::new();
    for (rel, _) in &all_files {
        if !seen.insert(rel.clone()) {
            return Err(format!(
                "duplicate file path in sources: {} (unpack would overwrite it)",
                rel.display()
            ));
        }
    }

    let doc = build_doc_from_files(&all_files);
    write_docx_file(&doc, output)?;

    if let Some(ref cb) = on_progress {
        cb(Progress {
            current: 0,
            total: 0,
            file: String::new(),
            phase: ProgressPhase::Done,
        });
    }

    Ok(PackResult {
        file_count: all_files.len(),
        binary_skipped,
        output_path: output.to_path_buf(),
    })
}

pub fn resolve_output_name(paths: &[PathBuf], explicit: &Path) -> PathBuf {
    if explicit != Path::new(crate::constants::DEFAULT_OUTPUT) {
        return explicit.to_path_buf();
    }

    let dir = if paths.len() == 1 {
        paths[0].parent().map(|p| p.to_path_buf())
    } else if let Some(common) = find_common_parent(paths) {
        Some(common.to_path_buf())
    } else {
        paths
            .first()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
    };

    let dir = dir.unwrap_or_else(|| PathBuf::from("."));

    let name = if paths.len() == 1 {
        let single = &paths[0];
        if single.is_file() {
            single
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "output".to_string())
        } else {
            single
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "output".to_string())
        }
    } else if let Some(common) = find_common_parent(paths) {
        common
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string())
    } else {
        "output".to_string()
    };

    auto_rename(dir.join(format!("{}.docx", name)))
}

fn find_common_parent(paths: &[PathBuf]) -> Option<PathBuf> {
    let parents: Vec<&Path> = paths.iter().filter_map(|p| p.parent()).collect();
    if parents.is_empty() {
        return None;
    }
    let mut common = parents[0];
    for p in &parents[1..] {
        while !p.starts_with(common) {
            common = common.parent()?;
        }
    }
    if common.as_os_str().is_empty() || common.parent().is_none() {
        return None;
    }
    Some(common.to_path_buf())
}

pub fn auto_rename(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();
    let dir = path.parent().unwrap_or(Path::new("."));
    for i in 1..=9999u32 {
        let new_path = dir.join(format!("{}_{}{}", stem, i, ext));
        if !new_path.exists() {
            return new_path;
        }
    }
    // Fallback with timestamp (unlikely to collide)
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    dir.join(format!("{}_{}{}", stem, ts, ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_meaningful_text_empty() {
        assert!(is_meaningful_text(""));
    }

    #[test]
    fn test_is_meaningful_text_ascii() {
        assert!(is_meaningful_text("fn main() {}"));
    }

    #[test]
    fn test_is_meaningful_text_cjk() {
        assert!(is_meaningful_text("你好世界"));
    }

    #[test]
    fn test_is_meaningful_text_mixed() {
        assert!(is_meaningful_text("Hello 世界 123!"));
    }

    #[test]
    fn test_is_meaningful_text_whitespace() {
        assert!(is_meaningful_text("line1\n\tline2\r\nline3"));
    }

    #[test]
    fn test_is_meaningful_text_replacement() {
        assert!(!is_meaningful_text("\u{FFFD}"));
        assert!(!is_meaningful_text("good\u{FFFD}bad"));
    }

    #[test]
    fn test_is_meaningful_text_nul() {
        assert!(!is_meaningful_text("\0"));
    }

    #[test]
    fn test_is_meaningful_text_controls() {
        assert!(!is_meaningful_text("xxxx\x01xxxx\x02xxxx\x03"));
    }

    #[test]
    fn test_is_meaningful_text_c1() {
        assert!(!is_meaningful_text("\u{80}\u{9F}"));
    }

    #[test]
    fn test_is_meaningful_text_nonchar() {
        assert!(!is_meaningful_text("\u{FDD0}"));
    }

    #[test]
    fn test_is_meaningful_text_threshold() {
        assert!(is_meaningful_text("1234567890")); // 10 good, 0 bad → 0 ≤ 10 ✓
        assert!(is_meaningful_text("123456789\u{01}")); // 9 good + 1 bad = 10, 10 ≤ 10 ✓
        assert!(is_meaningful_text("1234567890\u{01}")); // 10 good + 1 bad = 11, 10 ≤ 11 ✓
        assert!(!is_meaningful_text("12345678\u{01}\u{01}")); // 8 good + 2 bad = 10, 20 > 10 ✗
    }

    #[test]
    fn test_is_meaningful_text_del() {
        assert!(!is_meaningful_text("\u{7F}"));
    }

    #[test]
    fn test_is_meaningful_text_allowable_controls() {
        assert!(is_meaningful_text("\t\n\r\u{0C}"));
    }

    #[test]
    fn duplicate_rel_paths_rejected() {
        use tempfile::TempDir;
        let d = TempDir::new().unwrap();
        let a = d.path().join("a");
        let b = d.path().join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(a.join("x.txt"), b"one").unwrap();
        std::fs::write(b.join("x.txt"), b"two").unwrap();
        let out = d.path().join("out.docx");
        let rules = crate::ignore::ExcludeRules::new();
        let enc: Vec<String> = Vec::new();
        let result = pack_files(&[a, b], &out, &rules, &enc, None);
        assert!(result.is_err());
        assert!(!out.exists());
    }
}
