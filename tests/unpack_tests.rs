use docpack::unpack::unpack_docx;
use docpack::pack::pack_dir;
use docpack::docx::model::Document;
use docpack::docx::writer::write_docx;
use docpack::ignore::ExcludeRules;

fn write_docx_to_file(doc: &Document, path: &std::path::Path) {
    let file = std::fs::File::create(path).unwrap();
    write_docx(file, doc).unwrap();
}

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

    write_docx_to_file(&doc, &docx_path);

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

    write_docx_to_file(&doc, &docx_path);

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

    write_docx_to_file(&doc, &docx_path);

    let unpack_dir = dir.path().join("unpacked");
    let result = unpack_docx(&docx_path, &unpack_dir).unwrap();
    assert_eq!(result.file_count, 2);

    let file1 = unpack_dir.join("file1.txt");
    assert!(file1.exists());
    assert_eq!(std::fs::read_to_string(file1).unwrap(), "line1\nline2\nline3");

    let file2 = unpack_dir.join("file2.txt");
    assert!(file2.exists());
    assert_eq!(std::fs::read_to_string(file2).unwrap(), "content2");
}

#[test]
fn test_default_output_path() {
    let input = std::path::Path::new("C:\\Users\\test\\document.docx");
    let out = docpack::unpack::default_output_path(input);
    assert!(out.to_string_lossy().contains("document_unpacked"));
    assert!(!out.to_string_lossy().contains(".docx"));
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

    write_docx_to_file(&doc, &docx_path);

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

    write_docx_to_file(&doc, &docx_path);

    let unpack_dir = dir.path().join("unpacked");
    let result = unpack_docx(&docx_path, &unpack_dir).unwrap();
    assert_eq!(result.file_count, 1);

    let file1 = unpack_dir.join("file.txt");
    assert!(file1.exists());
    assert_eq!(std::fs::read_to_string(file1).unwrap(), "line1\nline2\nline3");
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

    write_docx_to_file(&doc, &docx_path);

    let unpack_dir = dir.path().join("unpacked");
    let result = unpack_docx(&docx_path, &unpack_dir).unwrap();
    assert_eq!(result.file_count, 3);

    let file2 = unpack_dir.join("file2.txt");
    assert!(file2.exists());
    assert_eq!(std::fs::read_to_string(file2).unwrap(), "");
}
