use crate::docx::model::Document;
use std::io::Write;
use zip::write::FileOptions;
use zip::ZipWriter;

pub fn write_docx<W: Write + std::io::Seek>(writer: W, doc: &Document) -> Result<(), String> {
    let mut zip = ZipWriter::new(writer);
    let options: FileOptions<()> = FileOptions::default();

    // Create [Content_Types].xml
    let mut content_types = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/docpack/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/docpack/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>"#
    );
    if doc.header.is_some() {
        content_types.push_str(r#"
  <Override PartName="/docpack/header.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/>"#);
    }
    content_types.push_str("\n</Types>");
    zip.start_file("[Content_Types].xml", options)
        .map_err(|e| format!("Start [Content_Types].xml: {}", e))?;
    zip.write_all(content_types.as_bytes())
        .map_err(|e| format!("Write [Content_Types].xml: {}", e))?;

    // Create _rels/.rels
    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="docpack/document.xml"/>
</Relationships>"#;
    zip.start_file("_rels/.rels", options)
        .map_err(|e| format!("Start _rels/.rels: {}", e))?;
    zip.write_all(rels.as_bytes())
        .map_err(|e| format!("Write _rels/.rels: {}", e))?;

    // Create docpack/_rels/document.xml.rels (document-level relationships)
    let mut docx_rels = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#
    );
    if doc.header.is_some() {
        docx_rels.push_str(r#"
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header.xml"/>"#);
    }
    docx_rels.push_str("\n</Relationships>");
    zip.start_file("docpack/_rels/document.xml.rels", options)
        .map_err(|e| format!("Start docpack/_rels/document.xml.rels: {}", e))?;
    zip.write_all(docx_rels.as_bytes())
        .map_err(|e| format!("Write docpack/_rels/document.xml.rels: {}", e))?;

    // Create styles.xml
    let styles = crate::docx::style_gen::generate_styles_xml();
    zip.start_file("docpack/styles.xml", options)
        .map_err(|e| format!("Start docpack/styles.xml: {}", e))?;
    zip.write_all(styles.as_bytes())
        .map_err(|e| format!("Write docpack/styles.xml: {}", e))?;

    // Create header.xml if document has a header
    if let Some(ref header_text) = doc.header {
        let header_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:p><w:r><w:t>{}</w:t></w:r></w:p>
</w:hdr>"#,
            escape_xml(header_text)
        );
        zip.start_file("docpack/header.xml", options)
            .map_err(|e| format!("Start docpack/header.xml: {}", e))?;
        zip.write_all(header_xml.as_bytes())
            .map_err(|e| format!("Write docpack/header.xml: {}", e))?;
    }

    // Create document.xml
    let has_header = doc.header.is_some();
    let mut document_xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>"#,
    );

    for paragraph in &doc.paragraphs {
        if paragraph.is_heading() {
            let style = paragraph.style.as_deref().unwrap_or("Heading1");
            let text = paragraph
                .runs
                .first()
                .map(|r| escape_xml(&r.text))
                .unwrap_or_default();
            document_xml.push_str(&format!(
                "<w:p><w:pPr><w:pStyle w:val=\"{}\"/></w:pPr>{}</w:p>",
                style,
                write_runs_with_br(&text)
            ));
        } else {
            for run in &paragraph.runs {
                let text = escape_xml(&run.text);
                document_xml.push_str(&format!("<w:p>{}</w:p>", write_runs_with_br(&text)));
            }
        }
    }

    // Section properties with header reference
    document_xml.push_str("<w:sectPr>");
    if has_header {
        document_xml.push_str(r#"<w:headerReference w:type="default" r:id="rId2"/>"#);
    }
    document_xml.push_str("</w:sectPr>");

    document_xml.push_str("</w:body></w:document>");

    zip.start_file("docpack/document.xml", options)
        .map_err(|e| format!("Start docpack/document.xml: {}", e))?;
    zip.write_all(document_xml.as_bytes())
        .map_err(|e| format!("Write docpack/document.xml: {}", e))?;

    zip.finish().map_err(|e| format!("Finish zip: {}", e))?;

    Ok(())
}

fn write_runs_with_br(text: &str) -> String {
    let mut result = String::new();
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            result.push_str("<w:br/>");
        }
        result.push_str(&format!("<w:r><w:t>{}</w:t></w:r>", line));
    }
    result
}

fn escape_xml(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            '\r' => result.push_str("&#xD;"),
            // Strip invalid XML chars (allow \t, \n)
            c if (c as u32) < 0x20 && c != '\t' && c != '\n' => {}
            _ => result.push(c),
        }
    }
    result
}
