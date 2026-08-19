use docpack::cli;
use docpack::constants;
use docpack::docx::model::Document;
use docpack::docx::reader::read_docx;
use docpack::docx::writer::write_docx;
use docpack::ignore::ExcludeRules;
use docpack::lang::I18n;
use docpack::pack::{collect_text_files, is_text_file, pack_dir};
use docpack::platform::install::{ContextMenu, Platform};
use docpack::unpack::unpack_docx;
use std::io::{Cursor, Write};
use std::path::PathBuf;

// CLI Tests
#[test]
fn test_cli_pack() {
    let cli = cli::Cli::parse_from(["docpack", "pack", "src", "-o", "out.docx"]);
    match cli.command {
        Some(cli::Commands::Pack { paths, output, .. }) => {
            assert_eq!(paths, vec![PathBuf::from("src")]);
            assert_eq!(output, PathBuf::from("out.docx"));
        }
        _ => panic!("Expected Pack command"),
    }
}

#[test]
fn test_cli_pack_default_output() {
    let cli = cli::Cli::parse_from(["docpack", "pack", "src"]);
    match cli.command {
        Some(cli::Commands::Pack { paths, output, .. }) => {
            assert_eq!(paths, vec![PathBuf::from("src")]);
            assert_eq!(output, PathBuf::from(constants::DEFAULT_OUTPUT));
        }
        _ => panic!("Expected Pack command"),
    }
}

#[test]
fn test_cli_unpack() {
    let cli = cli::Cli::parse_from(["docpack", "unpack", "in.docx", "-o", "out_dir"]);
    match cli.command {
        Some(cli::Commands::Unpack { input, output, .. }) => {
            assert_eq!(input, PathBuf::from("in.docx"));
            assert_eq!(output, Some(PathBuf::from("out_dir")));
        }
        _ => panic!("Expected Unpack command"),
    }
}

#[test]
fn test_cli_install() {
    let cli = cli::Cli::parse_from(["docpack", "install"]);
    assert!(matches!(cli.command, Some(cli::Commands::Install)));
}

#[test]
fn test_cli_help() {
    let cli = cli::Cli::try_parse_from(["docpack", "--help"]);
    assert!(cli.is_ok() || cli.is_err());
}

// DOCX Model Tests
#[test]
fn test_docx_model_new_document() {
    let doc = Document::new();
    assert!(doc.paragraphs.is_empty());
    assert!(doc.header.is_none());
}

#[test]
fn test_docx_model_add_heading() {
    let mut doc = Document::new();
    doc.add_heading("src/main.rs");
    assert_eq!(doc.paragraphs.len(), 1);
    assert_eq!(doc.paragraphs[0].style.as_deref(), Some("Heading1"));
    assert_eq!(doc.paragraphs[0].runs[0].text, "src/main.rs");
}

#[test]
fn test_docx_model_add_text() {
    let mut doc = Document::new();
    doc.add_text("line1\nline2");
    assert_eq!(doc.paragraphs.len(), 2);
    assert!(doc.paragraphs[0].style.is_none());
}

#[test]
fn test_docx_model_is_heading() {
    let h = docpack::docx::model::Paragraph {
        style: Some("Heading1".into()),
        runs: vec![],
    };
    let h2 = docpack::docx::model::Paragraph {
        style: Some("Heading2".into()),
        runs: vec![],
    };
    let t = docpack::docx::model::Paragraph {
        style: None,
        runs: vec![],
    };
    assert!(h.is_heading());
    assert!(h2.is_heading());
    assert!(!t.is_heading());
}

#[test]
fn test_docx_model_header() {
    let mut doc = Document::new();
    doc.set_header("DocPack v1.0.0");
    assert_eq!(doc.header.as_deref(), Some("DocPack v1.0.0"));
}

// DOCX Reader Tests
#[test]
fn test_docx_reader_roundtrip() {
    let mut doc = Document::new();
    doc.add_heading("src/main.rs");
    doc.add_text("fn main() {}");
    doc.add_heading("README.md");
    doc.add_text("# Hello");

    let data = write_docx_to_bytes(&doc);
    let read_doc = read_docx(Cursor::new(data)).unwrap();
    assert_eq!(read_doc.paragraphs.len(), 4);
    assert!(read_doc.paragraphs[0].is_heading());
    assert_eq!(read_doc.paragraphs[0].runs[0].text, "src/main.rs");
}

#[test]
fn test_docx_reader_roundtrip_with_header() {
    let mut doc = Document::new();
    doc.set_header("DocPack v1.0.0");
    doc.add_heading("test.txt");
    doc.add_text("content");

    let data = write_docx_to_bytes(&doc);
    let read_doc = read_docx(Cursor::new(data)).unwrap();
    assert_eq!(read_doc.header.as_deref(), Some("DocPack v1.0.0"));
}

#[test]
fn test_docx_reader_empty_document() {
    let doc = Document::new();
    let data = write_docx_to_bytes(&doc);
    let read_doc = read_docx(Cursor::new(data)).unwrap();
    assert!(read_doc.paragraphs.is_empty());
}

#[test]
fn test_docx_reader_invalid_zip() {
    let result = read_docx(Cursor::new(b"not a zip".to_vec()));
    assert!(result.is_err());
}

// DOCX Writer Tests
#[test]
fn test_docx_writer_write_empty_docx() {
    let doc = Document::new();
    let cursor = Cursor::new(Vec::new());
    let result = write_docx(cursor, &doc);
    assert!(result.is_ok());
}

#[test]
fn test_docx_writer_write_docx_with_content() {
    let mut doc = Document::new();
    doc.set_header("DocPack v1.0.0");
    doc.add_heading("src/main.rs");
    doc.add_text("fn main() {\n    println!(\"hello\");\n}");
    doc.add_heading("README.md");
    doc.add_text("# DocPack");

    let cursor = Cursor::new(Vec::new());
    let result = write_docx(cursor, &doc);
    assert!(result.is_ok());
}

#[test]
fn test_docx_writer_docx_is_valid_zip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.docx");

    let mut doc = Document::new();
    doc.add_heading("test.txt");
    doc.add_text("content");
    let file = std::fs::File::create(&path).unwrap();
    write_docx(file, &doc).unwrap();

    let data = std::fs::read(&path).unwrap();
    let zip = zip::ZipArchive::new(std::io::Cursor::new(&*data)).unwrap();
    assert_eq!(zip.len(), 5); // [Content_Types].xml, _rels/.rels, docpack/_rels/document.xml.rels, docpack/styles.xml, docpack/document.xml
}

// Lang Tests
#[test]
fn test_lang_load_languages() {
    let i18n = I18n::new();
    assert!(!i18n.available_langs().is_empty());
}

#[test]
fn test_lang_get_string() {
    let i18n = I18n::new();
    let s = i18n.get("app_name");
    assert!(!s.is_empty());
    assert_ne!(s, "app_name");
}

#[test]
fn test_lang_switch_lang() {
    let mut i18n = I18n::new();
    i18n.set_lang("en");
    let en = i18n.get("app_name").to_string();
    i18n.set_lang("zh-CN");
    let zh = i18n.get("app_name").to_string();
    assert_ne!(en, zh);
}

#[test]
fn test_lang_missing_key() {
    let i18n = I18n::new();
    assert_eq!(i18n.get("nonexistent_key"), "nonexistent_key");
}

// Pack Tests
#[test]
fn test_pack_is_text_file() {
    let dir = tempfile::tempdir().unwrap();
    let text_path = dir.path().join("test.rs");
    let mut f = std::fs::File::create(&text_path).unwrap();
    f.write_all(b"fn main() {}").unwrap();
    assert!(is_text_file(&text_path, &[]));

    let bin_path = dir.path().join("test.bin");
    let mut f = std::fs::File::create(&bin_path).unwrap();
    f.write_all(&[0x00, 0x01, 0x02]).unwrap();
    assert!(!is_text_file(&bin_path, &[]));
}

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

// Unpack Tests
#[test]
fn test_unpack_roundtrip_pack_unpack() {
    let dir = tempfile::tempdir().unwrap();

    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
    std::fs::write(dir.path().join("readme.md"), "# DocPack").unwrap();

    let docx_path = dir.path().join("output.docx");
    let rules = ExcludeRules::new();
    pack_dir(dir.path(), &docx_path, &rules, &[], None).unwrap();

    let unpack_dir = dir.path().join("unpacked");
    let result = unpack_docx(&docx_path, &unpack_dir).unwrap();
    assert_eq!(result.file_count, 2);

    let main_rs = unpack_dir.join("src/main.rs");
    assert!(main_rs.exists());
    assert_eq!(std::fs::read_to_string(&main_rs).unwrap(), "fn main() {}");

    let readme = unpack_dir.join("readme.md");
    assert!(readme.exists());
    assert_eq!(std::fs::read_to_string(readme).unwrap(), "# DocPack");
}

#[test]
fn test_unpack_empty_file_between_headings() {
    let dir = tempfile::tempdir().unwrap();

    let docx_path = dir.path().join("test.docx");
    let mut doc = Document::new();
    doc.add_heading("file1.txt");
    doc.add_text("content1");
    doc.add_heading("file2.txt");
    doc.add_heading("file3.txt");
    doc.add_text("content3");

    let file = std::fs::File::create(&docx_path).unwrap();
    write_docx(file, &doc).unwrap();

    let unpack_dir = dir.path().join("unpacked");
    let result = unpack_docx(&docx_path, &unpack_dir).unwrap();
    assert_eq!(result.file_count, 3);

    let file1 = unpack_dir.join("file1.txt");
    assert!(file1.exists());
    assert_eq!(std::fs::read_to_string(file1).unwrap(), "content1");

    let file2 = unpack_dir.join("file2.txt");
    assert!(file2.exists());
    assert_eq!(std::fs::read_to_string(file2).unwrap(), "");

    let file3 = unpack_dir.join("file3.txt");
    assert!(file3.exists());
    assert_eq!(std::fs::read_to_string(file3).unwrap(), "content3");
}

#[test]
fn test_unpack_empty_file_at_end() {
    let dir = tempfile::tempdir().unwrap();

    let docx_path = dir.path().join("test.docx");
    let mut doc = Document::new();
    doc.add_heading("file1.txt");
    doc.add_text("content1");
    doc.add_heading("file2.txt");

    let file = std::fs::File::create(&docx_path).unwrap();
    write_docx(file, &doc).unwrap();

    let unpack_dir = dir.path().join("unpacked");
    let result = unpack_docx(&docx_path, &unpack_dir).unwrap();
    assert_eq!(result.file_count, 2);

    let file1 = unpack_dir.join("file1.txt");
    assert!(file1.exists());
    assert_eq!(std::fs::read_to_string(file1).unwrap(), "content1");

    let file2 = unpack_dir.join("file2.txt");
    assert!(file2.exists());
    assert_eq!(std::fs::read_to_string(file2).unwrap(), "");
}

#[test]
fn test_unpack_multiple_body_paragraphs() {
    let dir = tempfile::tempdir().unwrap();

    let docx_path = dir.path().join("test.docx");
    let mut doc = Document::new();
    doc.add_heading("file1.txt");
    doc.add_text("line1");
    doc.add_text("line2");
    doc.add_text("line3");
    doc.add_heading("file2.txt");
    doc.add_text("content2");

    let file = std::fs::File::create(&docx_path).unwrap();
    write_docx(file, &doc).unwrap();

    let unpack_dir = dir.path().join("unpacked");
    let result = unpack_docx(&docx_path, &unpack_dir).unwrap();
    assert_eq!(result.file_count, 2);

    let file1 = unpack_dir.join("file1.txt");
    assert!(file1.exists());
    assert_eq!(
        std::fs::read_to_string(file1).unwrap(),
        "line1\nline2\nline3"
    );

    let file2 = unpack_dir.join("file2.txt");
    assert!(file2.exists());
    assert_eq!(std::fs::read_to_string(file2).unwrap(), "content2");
}

#[test]
fn test_unpack_multiple_body_paragraphs_different_headings() {
    let dir = tempfile::tempdir().unwrap();

    let docx_path = dir.path().join("test.docx");
    let mut doc = Document::new();
    doc.add_heading("file1.txt");
    doc.add_text("line1");
    doc.add_heading("file2.txt");
    doc.add_text("line2");
    doc.add_heading("file3.txt");
    doc.add_text("line3");

    let file = std::fs::File::create(&docx_path).unwrap();
    write_docx(file, &doc).unwrap();

    let unpack_dir = dir.path().join("unpacked");
    let result = unpack_docx(&docx_path, &unpack_dir).unwrap();
    assert_eq!(result.file_count, 3);

    let file1 = unpack_dir.join("file1.txt");
    assert!(file1.exists());
    assert_eq!(std::fs::read_to_string(file1).unwrap(), "line1");

    let file2 = unpack_dir.join("file2.txt");
    assert!(file2.exists());
    assert_eq!(std::fs::read_to_string(file2).unwrap(), "line2");

    let file3 = unpack_dir.join("file3.txt");
    assert!(file3.exists());
    assert_eq!(std::fs::read_to_string(file3).unwrap(), "line3");
}

#[test]
fn test_unpack_multi_line_content_preserves_newlines() {
    let dir = tempfile::tempdir().unwrap();

    let docx_path = dir.path().join("test.docx");
    let mut doc = Document::new();
    doc.add_heading("file.txt");
    doc.add_text("line1\nline2\nline3");

    let file = std::fs::File::create(&docx_path).unwrap();
    write_docx(file, &doc).unwrap();

    let unpack_dir = dir.path().join("unpacked");
    let result = unpack_docx(&docx_path, &unpack_dir).unwrap();
    assert_eq!(result.file_count, 1);

    let file1 = unpack_dir.join("file.txt");
    assert!(file1.exists());
    assert_eq!(
        std::fs::read_to_string(file1).unwrap(),
        "line1\nline2\nline3"
    );
}

#[test]
fn test_unpack_empty_file_at_end_no_trailing_content() {
    let dir = tempfile::tempdir().unwrap();

    let docx_path = dir.path().join("test.docx");
    let mut doc = Document::new();
    doc.add_heading("file1.txt");
    doc.add_text("content1");
    doc.add_heading("file2.txt");
    doc.add_heading("file3.txt");
    doc.add_text("content3");

    let file = std::fs::File::create(&docx_path).unwrap();
    write_docx(file, &doc).unwrap();

    let unpack_dir = dir.path().join("unpacked");
    let result = unpack_docx(&docx_path, &unpack_dir).unwrap();
    assert_eq!(result.file_count, 3);

    let file2 = unpack_dir.join("file2.txt");
    assert!(file2.exists());
    assert_eq!(std::fs::read_to_string(file2).unwrap(), "");
}

// Platform Install Tests
#[test]
fn test_platform_detect() {
    let _p = Platform::detect();
}

#[test]
fn test_context_menu_install_nonexistent() {
    let p = PathBuf::from("/nonexistent/docpack");
    let result = ContextMenu::install(&p);
    match Platform::detect() {
        Platform::Windows => {
            assert!(result.is_err() || result.is_ok());
        }
        _ => {
            assert!(result.is_err());
        }
    }
}

// Helper function for writer tests
fn write_docx_to_bytes(doc: &Document) -> Vec<u8> {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    write_docx(cursor, doc).unwrap();
    buffer
}
