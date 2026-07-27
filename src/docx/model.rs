pub struct Document {
    pub paragraphs: Vec<Paragraph>,
    pub header: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Paragraph {
    pub style: Option<String>,
    pub runs: Vec<Run>,
}

#[derive(Debug, Clone)]
pub struct Run {
    pub text: String,
}

impl Document {
    pub fn new() -> Self {
        Document {
            paragraphs: Vec::new(),
            header: None,
        }
    }

    pub fn add_heading(&mut self, text: &str) {
        self.paragraphs.push(Paragraph {
            style: Some("Heading1".into()),
            runs: vec![Run { text: text.into() }],
        });
    }

    pub fn add_text(&mut self, text: &str) {
        for line in text.lines() {
            self.paragraphs.push(Paragraph {
                style: None,
                runs: vec![Run { text: line.into() }],
            });
        }
    }

    pub fn set_header(&mut self, header: &str) {
        self.header = Some(header.into());
    }
}

impl Paragraph {
    pub fn is_heading(&self) -> bool {
        self.style.as_deref().map(|s| s.starts_with("Heading")).unwrap_or(false)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
