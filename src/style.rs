use eframe::egui;
use std::sync::OnceLock;

pub const PIXELS_PER_POINT: f32 = 1.0;
pub const DRAG_HOVER_COLOR: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(204, 204, 255, 255);
pub const GRID_COLUMNS: usize = 3;
pub const SPACING: f32 = 8.0;
pub const SPACING_GAP: f32 = 4.0;
pub const COLOR_SUCCESS: egui::Color32 = egui::Color32::from_rgb(0, 204, 0);
pub const COLOR_ERROR: egui::Color32 = egui::Color32::from_rgb(204, 0, 0);
pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

static FONT_DEFINITIONS: OnceLock<egui::FontDefinitions> = OnceLock::new();

fn system_font_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    if cfg!(target_os = "windows") {
        if let Ok(windir) = std::env::var("WINDIR") {
            dirs.push(std::path::PathBuf::from(windir).join("Fonts"));
        } else if let Ok(sysroot) = std::env::var("SYSTEMROOT") {
            dirs.push(std::path::PathBuf::from(sysroot).join("Fonts"));
        }
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            dirs.push(std::path::PathBuf::from(localappdata).join(r"Microsoft\Windows\Fonts"));
        }
    } else if cfg!(target_os = "macos") {
        dirs.push(std::path::PathBuf::from("/System/Library/Fonts"));
        dirs.push(std::path::PathBuf::from("/Library/Fonts"));
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(std::path::PathBuf::from(home).join("Library/Fonts"));
        }
    } else {
        dirs.push(std::path::PathBuf::from("/usr/share/fonts"));
        if let Ok(home) = std::env::var("HOME") {
            let home_path = std::path::Path::new(&home);
            dirs.push(home_path.join(".fonts"));
            dirs.push(home_path.join(".local/share/fonts"));
        }
    }
    dirs
}

fn cjk_font_names() -> &'static [&'static str] {
    if cfg!(target_os = "windows") {
        &[
            "msyh.ttc",
            "msyh.ttf",
            "msyhbd.ttc",
            "simsun.ttc",
            "simsun.ttf",
            "simhei.ttf",
            "deng.ttf",
            "yahei.ttf",
            "yahei.ttc",
        ]
    } else if cfg!(target_os = "macos") {
        &[
            "PingFang.ttc",
            "STHeiti Light.ttc",
            "STHeiti Medium.ttc",
            "NotoSansCJK-Regular.ttc",
            "Arial Unicode.ttf",
        ]
    } else {
        &[
            "wqy-zenhei.ttc",
            "wqy-zenhei.ttf",
            "NotoSansCJK-Regular.ttc",
            "NotoSansCJK-Regular.ttf",
            "noto-sans-cjk.ttc",
        ]
    }
}

fn find_system_cjk_font() -> Option<std::path::PathBuf> {
    let dirs = system_font_dirs();
    let names = cjk_font_names();

    for dir in &dirs {
        for name in names {
            let path = dir.join(name);
            if path.exists() {
                return Some(path);
            }
        }
    }

    // Fallback: pick any .ttf or .ttc file from the first font directory
    let dir = dirs.into_iter().next()?;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "ttf" || ext == "ttc" {
                    return Some(path);
                }
            }
        }
    }

    None
}

pub fn get_font_definitions() -> &'static egui::FontDefinitions {
    FONT_DEFINITIONS.get_or_init(|| build_font_definitions())
}

fn build_font_definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    if let Some(path) = find_system_cjk_font() {
        if let Ok(bytes) = std::fs::read(&path) {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("CJK")
                .to_string();
            fonts
                .font_data
                .insert(name.clone(), egui::FontData::from_owned(bytes));
            if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                proportional.push(name);
            }
        }
    }

    fonts
}
