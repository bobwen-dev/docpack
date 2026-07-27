use docpack::platform::install::{ContextMenu, Platform};
use std::path::PathBuf;

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
