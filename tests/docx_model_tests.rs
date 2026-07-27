use docpack::docx::model::Document;

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
    let h = docpack::docx::model::Paragraph { style: Some("Heading1".into()), runs: vec![] };
    let h2 = docpack::docx::model::Paragraph { style: Some("Heading2".into()), runs: vec![] };
    let t = docpack::docx::model::Paragraph { style: None, runs: vec![] };
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
