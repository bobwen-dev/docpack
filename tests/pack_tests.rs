use docpack::pack::{is_text_file, pack_dir, collect_text_files};
use docpack::ignore::ExcludeRules;
use std::io::Write;

fn create_file(dir: &tempfile::TempDir, name: &str, data: &[u8]) {
    let mut f = std::fs::File::create(dir.path().join(name)).unwrap();
    f.write_all(data).unwrap();
}

// ── is_text_file tests ──

#[test]
fn test_is_text_file_empty() {
    let dir = tempfile::tempdir().unwrap();
    create_file(&dir, "empty", b"");
    assert!(is_text_file(&dir.path().join("empty"), &[]));
}

#[test]
fn test_is_text_file_ascii() {
    let dir = tempfile::tempdir().unwrap();
    create_file(&dir, "f.rs", b"fn main() {}");
    assert!(is_text_file(&dir.path().join("f.rs"), &[]));
}

#[test]
fn test_is_text_file_cjk_utf8() {
    let dir = tempfile::tempdir().unwrap();
    create_file(&dir, "cjk.txt", "你好世界".as_bytes());
    assert!(is_text_file(&dir.path().join("cjk.txt"), &[]));
}

#[test]
fn test_is_text_file_utf8_bom() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = vec![0xEF, 0xBB, 0xBF];
    data.extend_from_slice(b"hello");
    create_file(&dir, "bom.txt", &data);
    assert!(is_text_file(&dir.path().join("bom.txt"), &[]));
}

#[test]
fn test_is_text_file_utf16le_bom() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = vec![0xFF, 0xFE];
    for &b in b"hello" {
        data.push(b);
        data.push(0x00);
    }
    create_file(&dir, "u16le.txt", &data);
    assert!(is_text_file(&dir.path().join("u16le.txt"), &[]));
}

#[test]
fn test_is_text_file_utf16be_bom() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = vec![0xFE, 0xFF];
    for &b in b"hello" {
        data.push(0x00);
        data.push(b);
    }
    create_file(&dir, "u16be.txt", &data);
    assert!(is_text_file(&dir.path().join("u16be.txt"), &[]));
}

#[test]
fn test_is_text_file_utf16le_no_bom() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = Vec::new();
    for &b in b"hello world" {
        data.push(b);
        data.push(0x00);
    }
    create_file(&dir, "u16le.txt", &data);
    assert!(is_text_file(&dir.path().join("u16le.txt"), &[]));
}

#[test]
fn test_is_text_file_utf16be_no_bom() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = Vec::new();
    for &b in b"hello world" {
        data.push(0x00);
        data.push(b);
    }
    create_file(&dir, "u16be.txt", &data);
    assert!(is_text_file(&dir.path().join("u16be.txt"), &[]));
}

#[test]
fn test_is_text_file_utf32le_bom() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = vec![0xFF, 0xFE, 0x00, 0x00];
    for &b in b"hi" {
        data.push(b);
        data.push(0x00);
        data.push(0x00);
        data.push(0x00);
    }
    create_file(&dir, "u32le.txt", &data);
    assert!(is_text_file(&dir.path().join("u32le.txt"), &[]));
}

#[test]
fn test_is_text_file_utf32be_bom() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = vec![0x00, 0x00, 0xFE, 0xFF];
    for &b in b"hi" {
        data.push(0x00);
        data.push(0x00);
        data.push(0x00);
        data.push(b);
    }
    create_file(&dir, "u32be.txt", &data);
    assert!(is_text_file(&dir.path().join("u32be.txt"), &[]));
}

#[test]
fn test_is_text_file_gbk() {
    let dir = tempfile::tempdir().unwrap();
    // GBK encoding of "你好"
    let gbk = &[0xC4, 0xE3, 0xBA, 0xC3];
    create_file(&dir, "gbk.txt", gbk);
    // Without GBK in local_encodings → should be detected via meaningful check
    // Since these bytes are NOT valid UTF-8 and there's no GBK configured, the result depends on whether
    // the decoded content (if treated as some encoding) passes the meaningful check.
    // With empty encodings, UTF-8 fails, and there are no configured encodings to try. So → false (binary).
    assert!(!is_text_file(&dir.path().join("gbk.txt"), &[]));
    // With GBK configured → should decode correctly and pass meaningful check
    assert!(is_text_file(&dir.path().join("gbk.txt"), &["GBK".into()]));
}

#[test]
fn test_is_text_file_shift_jis() {
    let dir = tempfile::tempdir().unwrap();
    // Shift_JIS encoding of "こんにちは"
    let sjis = &[0x82, 0xB1, 0x82, 0xF1, 0x82, 0xC9, 0x82, 0xBF, 0x82, 0xCD];
    create_file(&dir, "sjis.txt", sjis);
    assert!(!is_text_file(&dir.path().join("sjis.txt"), &[]));
    assert!(is_text_file(&dir.path().join("sjis.txt"), &["Shift_JIS".into()]));
}

#[test]
fn test_is_text_file_binary_with_nul() {
    let dir = tempfile::tempdir().unwrap();
    create_file(&dir, "bin.dat", &[0x00, 0x01, 0x02]);
    assert!(!is_text_file(&dir.path().join("bin.dat"), &[]));
}

#[test]
fn test_is_text_file_binary_no_nul_but_meaningless() {
    let dir = tempfile::tempdir().unwrap();
    // Data that is valid UTF-8 but >10% control characters → not meaningful
    // 20 chars: 17 printable + 3 control (each 0x01) — 3*10 = 30 > 20 → fails
    let mut data = b"abcdefghijklmnopq".to_vec();
    data.push(0x01);
    data.push(0x01);
    data.push(0x01);
    create_file(&dir, "bad.txt", &data);
    assert!(!is_text_file(&dir.path().join("bad.txt"), &[]));
}

#[test]
fn test_is_text_file_binary_no_nul_meaningless_gbk() {
    let dir = tempfile::tempdir().unwrap();
    // GBK bytes that decode without errors but produce garbage characters
    // Many single-byte GBK sequences map to C1 control chars (0x80-0x9F) → not meaningful
    let garbage_gbk: Vec<u8> = (0x80..0x9A).collect(); // mostly C1 controls when decoded via GBK
    create_file(&dir, "garbage.txt", &garbage_gbk);
    // Without GBK: binary (not valid UTF-8)
    assert!(!is_text_file(&dir.path().join("garbage.txt"), &[]));
    // With GBK configured: decodes without errors but produces C1 controls → not meaningful
    assert!(!is_text_file(&dir.path().join("garbage.txt"), &["GBK".into()]));
}

#[test]
fn test_is_text_file_nonexistent() {
    assert!(!is_text_file(&std::path::Path::new("/nonexistent/foo.txt"), &[]));
}

#[test]
fn test_is_text_file_png_header() {
    let dir = tempfile::tempdir().unwrap();
    // PNG magic bytes (valid but binary)
    let png_header = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    create_file(&dir, "img.png", png_header);
    // Has 0x0A (LF) and 0x0D (CR) which are allowable controls, and 0x1A (SUB) = control
    // 8 bytes: 7 valid ASCII + 1 SUB(0x1A). 1*10=10 < 8? No, so fails meaningful check.
    // But actually 0x89 is not valid UTF-8 start byte... it would fail UTF-8 decode.
    // Without configured encoding → binary
    assert!(!is_text_file(&dir.path().join("img.png"), &[]));
}

#[test]
fn test_is_text_file_long_text() {
    let dir = tempfile::tempdir().unwrap();
    // Text longer than 8192 bytes to test read_first_bytes truncation
    let long_text = "Hello World!\n".repeat(1000);
    assert!(long_text.len() > 8192);
    create_file(&dir, "long.txt", long_text.as_bytes());
    assert!(is_text_file(&dir.path().join("long.txt"), &[]));
}

// ── collect_text_files tests ──

#[test]
fn test_pack_collect_text_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();
    let mut f = std::fs::File::create(dir.path().join("src/main.rs")).unwrap();
    f.write_all(b"fn main() {}").unwrap();
    let mut f = std::fs::File::create(dir.path().join("readme.md")).unwrap();
    f.write_all(b"# Hello").unwrap();
    let mut f = std::fs::File::create(dir.path().join("image.png")).unwrap();
    f.write_all(&[0x89, 0x50, 0x4E, 0x47]).unwrap();

    let rules = ExcludeRules::default();
    let result = collect_text_files(dir.path(), &rules, &[], None).unwrap();
    assert_eq!(result.text_files.len(), 2);
    assert_eq!(result.binary_skipped, 0);
}

// ── pack_dir tests ──

#[test]
fn test_pack_single_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();
    let mut f = std::fs::File::create(dir.path().join("src/main.rs")).unwrap();
    f.write_all(b"fn main() { println!(\"hello\"); }").unwrap();

    let output = dir.path().join("output.docx");
    let rules = ExcludeRules::new();
    let result = pack_dir(dir.path(), &output, &rules, &[], None).unwrap();
    assert_eq!(result.file_count, 1);
}

#[test]
fn test_collect_text_files_rejects_file() {
    use docpack::pack::collect_text_files;
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("test.txt");
    std::fs::write(&file, b"hello").unwrap();
    let rules = ExcludeRules::new();
    let result = collect_text_files(&file, &rules, &[], None);
    assert!(result.is_err());
}
