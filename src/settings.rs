use serde::{Deserialize, Serialize};

pub const DEFAULT_LOCAL_ENCODINGS: &[&str] = &["UTF-8", "GBK", "SHIFT_JIS", "EUC-KR"];

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    #[serde(default)]
    pub exclude_file: String,
    #[serde(default = "default_encodings")]
    pub local_encodings: Vec<String>,
    #[serde(default)]
    pub context_menu_prompted: bool,
}

fn default_language() -> String {
    "en".into()
}

fn default_encodings() -> Vec<String> {
    DEFAULT_LOCAL_ENCODINGS
        .iter()
        .map(|s| s.to_string())
        .collect()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en".into(),
            exclude_patterns: vec![],
            exclude_file: String::new(),
            local_encodings: default_encodings(),
            context_menu_prompted: false,
        }
    }
}

fn settings_path() -> std::path::PathBuf {
    let mut path = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
    } else {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                std::path::PathBuf::from(home).join(".config")
            })
    };
    path.push("docpack");
    std::fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

pub fn load_settings() -> Settings {
    let path = settings_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(s) = serde_json::from_str::<Settings>(&content) {
            return s;
        }
    }
    Settings::default()
}

pub fn save_settings(s: &Settings) -> Result<(), String> {
    let path = settings_path();
    let data = serde_json::to_string_pretty(s).map_err(|e| format!("Serialize settings: {}", e))?;
    std::fs::write(&path, data).map_err(|e| format!("Write settings: {}", e))?;
    Ok(())
}
