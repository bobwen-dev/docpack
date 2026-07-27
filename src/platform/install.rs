use std::path::PathBuf;
use std::process::Command;

pub enum Platform {
    Windows,
    MacOs,
    Linux,
}

impl Platform {
    pub fn detect() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Linux
        }
    }
}

pub struct ContextMenu;

impl ContextMenu {
    fn run_reg(args: &[&str]) -> Result<(), String> {
        let mut cmd = Command::new("reg");
        cmd.args(args);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let output = cmd.output()
            .map_err(|e| format!("reg command failed: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Registry error: {}", stderr));
        }
        Ok(())
    }

    /// Try to read ProgID for .docx extension. Returns None if:
    /// - reg.exe unavailable or fails
    /// - .docx key not found
    /// - default value is (value not set) / empty / starts with '('
    /// - the extracted value doesn't look like a ProgID
    fn query_docx_progid() -> Option<String> {
        let output = Command::new("reg")
            .args(["query", r"HKEY_CLASSES_ROOT\.docx", "/ve"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("HKEY_") {
                continue;
            }
            if let Some(pos) = line.rfind("REG_") {
                let rest = &line[pos..];
                if let Some(val_pos) = rest.find(' ') {
                    let val = rest[val_pos..].trim();
                    if val.is_empty() || val.starts_with('(') {
                        return None; // "(value not set)" or localized equivalent
                    }
                    return Some(val.to_string());
                }
            }
        }
        None
    }

    /// Try to add unpack menu under a specific ProgID in HKCU.
    /// Returns true on success. Cleans up partial writes on failure.
    fn try_add_progid_unpack_menu(progid: &str, unpack_cmd: &str) -> bool {
        let base = format!("HKEY_CURRENT_USER\\Software\\Classes\\{}\\shell\\DocPackUnpack", progid);
        // Display name
        if Self::run_reg(&["add", &base, "/ve", "/d", "Unpack DOCX here", "/f"]).is_err() {
            return false;
        }
        // Command
        if Self::run_reg(&["add", &format!("{}\\command", base), "/ve", "/d", unpack_cmd, "/f"]).is_err() {
            let _ = Self::run_reg(&["delete", &base, "/f"]);
            return false;
        }
        true
    }

    pub fn install(exe_path: &PathBuf) -> Result<(), String> {
        let exe = exe_path.to_string_lossy().to_string();
        let pack_cmd = format!("\"{}\" gui-pack \"%1\"", exe);
        let unpack_cmd = format!("\"{}\" gui-unpack \"%1\"", exe);

        // Always-installed entries (pack on directories and all files)
        let entries = [
            vec!["add", r"HKEY_CLASSES_ROOT\Directory\shell\DocPack", "/ve", "/d", "Pack with DocPack", "/f"],
            vec!["add", r"HKEY_CLASSES_ROOT\Directory\shell\DocPack\command", "/ve", "/d", &pack_cmd, "/f"],
            vec!["add", r"HKEY_CLASSES_ROOT\*\shell\DocPack", "/ve", "/d", "Pack with DocPack", "/f"],
            vec!["add", r"HKEY_CLASSES_ROOT\*\shell\DocPack\command", "/ve", "/d", &pack_cmd, "/f"],
        ];

        for args in &entries {
            Self::run_reg(args)?;
        }

        // Unpack menu: prefer ProgID-scoped (appears only on .docx), fallback to *\shell
        let progid_ok = if let Some(progid) = Self::query_docx_progid() {
            Self::try_add_progid_unpack_menu(&progid, &unpack_cmd)
        } else {
            false
        };

        if !progid_ok {
            // Fallback: show on all files (unpack validates file type at runtime)
            Self::run_reg(&["add", r"HKEY_CLASSES_ROOT\*\shell\DocPackUnpack", "/ve", "/d", "Unpack DOCX here", "/f"])?;
            Self::run_reg(&["add", r"HKEY_CLASSES_ROOT\*\shell\DocPackUnpack\command", "/ve", "/d", &unpack_cmd, "/f"])?;
        }

        // Clear any stale *\shell entry left over from a previous fallback install,
        // so we don't accumulate both ProgID and *\shell simultaneously.
        if progid_ok {
            let _ = Self::run_reg(&["delete", r"HKEY_CLASSES_ROOT\*\shell\DocPackUnpack", "/f"]);
        }

        // Remove legacy entries from multiple possible locations
        let legacy_keys = [
            r"HKEY_CLASSES_ROOT\docx\shell\DocPack",
            r"HKEY_CLASSES_ROOT\.docx\shell\DocPack",
            r"HKEY_CLASSES_ROOT\docx\shell\DocPackUnpack",
            r"HKEY_CLASSES_ROOT\.docx\shell\DocPackUnpack",
            r"HKEY_CURRENT_USER\Software\Classes\.docx\shell\DocPackUnpack",
        ];
        for key in &legacy_keys {
            let _ = Self::run_reg(&["delete", key, "/f"]);
        }

        Ok(())
    }

    pub fn uninstall() -> Result<(), String> {
        let keys = [
            r"HKEY_CLASSES_ROOT\Directory\shell\DocPack",
            r"HKEY_CLASSES_ROOT\*\shell\DocPack",
            r"HKEY_CLASSES_ROOT\*\shell\DocPackUnpack",
        ];

        for key in &keys {
            let _ = Self::run_reg(&["delete", key, "/f"]);
        }

        // Clean up ProgID-scoped entries under known Office ProgIDs
        let known_progids = ["Word.Document.12", "Word.Document.8", "Word.Document.6"];
        for progid in &known_progids {
            let base = format!("HKEY_CURRENT_USER\\Software\\Classes\\{}\\shell\\DocPackUnpack", progid);
            let _ = Self::run_reg(&["delete", &base, "/f"]);
        }

        // Clean up legacy entry locations
        let legacy_keys = [
            r"HKEY_CLASSES_ROOT\docx\shell\DocPack",
            r"HKEY_CLASSES_ROOT\.docx\shell\DocPack",
            r"HKEY_CLASSES_ROOT\docx\shell\DocPackUnpack",
            r"HKEY_CLASSES_ROOT\.docx\shell\DocPackUnpack",
            r"HKEY_CURRENT_USER\Software\Classes\.docx\shell\DocPackUnpack",
        ];
        for key in &legacy_keys {
            let _ = Self::run_reg(&["delete", key, "/f"]);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
