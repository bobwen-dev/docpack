struct StyleDef {
    style_id: &'static str,
    name: &'static str,
    next: Option<&'static str>,
    outline_lvl: Option<u8>,
    bold: bool,
    font_size_half_pts: u32,
    spacing_before_pt: u32,
    spacing_after_pt: u32,
    keep_next: bool,
    keep_lines: bool,
}

fn default_styles() -> Vec<StyleDef> {
    vec![
        StyleDef {
            style_id: "Normal",
            name: "Normal",
            next: None,
            outline_lvl: None,
            bold: false,
            font_size_half_pts: 22,
            spacing_before_pt: 0,
            spacing_after_pt: 0,
            keep_next: false,
            keep_lines: false,
        },
        StyleDef {
            style_id: "Heading1",
            name: "heading 1",
            next: Some("Normal"),
            outline_lvl: Some(0),
            bold: true,
            font_size_half_pts: 48,
            spacing_before_pt: 24,
            spacing_after_pt: 12,
            keep_next: true,
            keep_lines: true,
        },
        StyleDef {
            style_id: "Heading2",
            name: "heading 2",
            next: Some("Normal"),
            outline_lvl: Some(1),
            bold: true,
            font_size_half_pts: 36,
            spacing_before_pt: 12,
            spacing_after_pt: 6,
            keep_next: true,
            keep_lines: true,
        },
    ]
}

/// pt to twentieths of a point (1/20 pt): OOXML spacing unit
fn pt_to_twips(pt: u32) -> u32 {
    pt * 20
}

pub fn generate_styles_xml() -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
"#,
    );

    for style in &default_styles() {
        xml.push_str(&format!(
            "  <w:style w:type=\"paragraph\" w:styleId=\"{}\">\n",
            style.style_id
        ));
        xml.push_str(&format!("    <w:name w:val=\"{}\"/>\n", style.name));

        if let Some(next) = style.next {
            xml.push_str(&format!("    <w:next w:val=\"{}\"/>\n", next));
        }

        let has_p_pr = style.keep_next
            || style.keep_lines
            || style.spacing_before_pt > 0
            || style.spacing_after_pt > 0
            || style.outline_lvl.is_some();

        if has_p_pr {
            xml.push_str("    <w:pPr>\n");

            if style.keep_next {
                xml.push_str("      <w:keepNext/>\n");
            }
            if style.keep_lines {
                xml.push_str("      <w:keepLines/>\n");
            }
            if style.spacing_before_pt > 0 || style.spacing_after_pt > 0 {
                xml.push_str(&format!(
                    "      <w:spacing w:before=\"{}\" w:after=\"{}\"/>\n",
                    pt_to_twips(style.spacing_before_pt),
                    pt_to_twips(style.spacing_after_pt),
                ));
            }
            if let Some(lvl) = style.outline_lvl {
                xml.push_str(&format!("      <w:outlineLvl w:val=\"{}\"/>\n", lvl));
            }

            xml.push_str("    </w:pPr>\n");
        }

        let has_run_props = style.bold;
        if has_run_props {
            xml.push_str("    <w:rPr>\n");
            if style.bold {
                xml.push_str("      <w:b/>\n");
            }
            xml.push_str(&format!(
                "      <w:sz w:val=\"{}\"/>\n",
                style.font_size_half_pts
            ));
            xml.push_str(&format!(
                "      <w:szCs w:val=\"{}\"/>\n",
                style.font_size_half_pts
            ));
            xml.push_str("    </w:rPr>\n");
        }

        xml.push_str("  </w:style>\n");
    }

    xml.push_str("</w:styles>");
    xml
}
