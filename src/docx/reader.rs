use crate::docx::model::{Document, Paragraph, Run};
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use std::io::{Read, Seek};
use zip::ZipArchive;

pub fn read_docx<R: Read + Seek>(reader: R) -> Result<Document, String> {
    let mut archive = ZipArchive::new(reader).map_err(|e| format!("Zip error: {}", e))?;

    let mut doc = Document::new();

    let mut xml = String::new();
    archive
        .by_name("docpack/document.xml")
        .map_err(|e| format!("Open document.xml: {}", e))?
        .read_to_string(&mut xml)
        .map_err(|e| format!("Read document.xml: {}", e))?;

    doc.paragraphs = parse_body(&xml);

    if let Ok(mut h) = archive.by_name("docpack/header.xml") {
        let mut header_xml = String::new();
        h.read_to_string(&mut header_xml).ok();
        if !header_xml.is_empty() {
            doc.header = parse_header(&header_xml);
        }
    }

    Ok(doc)
}

fn parse_header(xml: &str) -> Option<String> {
    let mut reader = XmlReader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_t = false;
    let mut text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                in_t = name.as_ref() == b"w:t";
            }
            Ok(Event::Text(ref e)) if in_t => {
                if let Ok(s) = e.unescape() {
                    text.push_str(&s);
                }
            }
            Ok(Event::End(_)) => in_t = false,
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if text.is_empty() { None } else { Some(text) }
}

fn parse_body(xml: &str) -> Vec<Paragraph> {
    let mut reader = XmlReader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut paragraphs: Vec<Paragraph> = Vec::new();

    let mut depth = 0usize;
    let mut in_p = false;
    let mut in_t = false;
    let mut current_style: Option<String> = None;
    let mut current_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                depth += 1;
                let name = e.name();
                let name_bytes = name.as_ref();
                if name_bytes == b"w:p" {
                    in_p = true;
                    current_style = None;
                    current_text.clear();
                } else if in_p && name_bytes == b"w:pPr" {
                } else if in_p && name_bytes == b"w:r" {
                } else if in_p && name_bytes == b"w:pStyle" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"w:val" {
                            if let Ok(val) = attr.decode_and_unescape_value(reader.decoder()) {
                                current_style = Some(val.to_string());
                            }
                        }
                    }
                } else if in_p && name_bytes == b"w:t" {
                    in_t = true;
                }
            }
            Ok(Event::Empty(ref e)) if in_p => {
                let name = e.name();
                let name_bytes = name.as_ref();
                if name_bytes == b"w:pStyle" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"w:val" {
                            if let Ok(val) = attr.decode_and_unescape_value(reader.decoder()) {
                                current_style = Some(val.to_string());
                            }
                        }
                    }
                } else if name_bytes == b"w:br" {
                    current_text.push('\n');
                } else if name_bytes == b"w:tab" {
                    current_text.push('\t');
                }
            }
            Ok(Event::Text(ref e)) if in_t => {
                if let Ok(s) = e.unescape() {
                    current_text.push_str(&s);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let name_bytes = name.as_ref();
                if name_bytes == b"w:p" && in_p {
                    paragraphs.push(Paragraph {
                        style: current_style.take(),
                        runs: if current_text.is_empty() {
                            vec![]
                        } else {
                            vec![Run { text: current_text.clone() }]
                        },
                    });
                    current_text.clear();
                    in_p = false;
                } else if name_bytes == b"w:t" {
                    in_t = false;
                }

                if depth > 0 {
                    depth -= 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    paragraphs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docx::writer::write_docx;
    use std::io::Cursor;

    fn write_docx_to_bytes(doc: &Document) -> Vec<u8> {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.docx");
        let file = std::fs::File::create(&path).unwrap();
        write_docx(file, doc).unwrap();
        std::fs::read(&path).unwrap()
    }

    #[test]
    fn test_roundtrip() {
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
    fn test_roundtrip_with_header() {
        let mut doc = Document::new();
        doc.set_header("DocPack v1.0.0");
        doc.add_heading("test.txt");
        doc.add_text("content");

        let data = write_docx_to_bytes(&doc);
        let read_doc = read_docx(Cursor::new(data)).unwrap();
        assert_eq!(read_doc.header.as_deref(), Some("DocPack v1.0.0"));
    }

    #[test]
    fn test_empty_document() {
        let doc = Document::new();
        let data = write_docx_to_bytes(&doc);
        let read_doc = read_docx(Cursor::new(data)).unwrap();
        assert!(read_doc.paragraphs.is_empty());
    }

    #[test]
    fn test_invalid_zip() {
        let result = read_docx(Cursor::new(b"not a zip".to_vec()));
        assert!(result.is_err());
    }
}
