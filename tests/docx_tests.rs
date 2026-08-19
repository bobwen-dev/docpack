use docpack::docx::model::Document;
use docpack::docx::reader::read_docx;
use docpack::docx::writer::write_docx;
use std::io::Cursor;

fn write_docx_to_bytes(doc: &Document) -> Vec<u8> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.docx");
    let file = std::fs::File::create(&path).unwrap();
    write_docx(file, doc).unwrap();
    std::fs::read(&path).unwrap()
}

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
    assert_eq!(zip.len(), 5);
}

#[test]
fn test_docx_writer_carriage_return_escape() {
    let mut doc = Document::new();
    doc.add_heading("test.txt");
    doc.add_text("line1\r\nline2\rline3");
    let data = write_docx_to_bytes(&doc);
    let read_doc = read_docx(Cursor::new(data)).unwrap();
    let body: String = read_doc
        .paragraphs
        .iter()
        .filter(|p| !p.is_heading())
        .flat_map(|p| p.runs.iter().map(|r| r.text.as_str()))
        .collect::<Vec<_>>()
        .join("");
    assert!(body.contains("line1"));
    assert!(body.contains("line2"));
    assert!(body.contains("line3"));
}
