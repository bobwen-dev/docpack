/// Application constants
pub const DEFAULT_OUTPUT: &str = "output.docx";
pub const UNPACKED_SUFFIX: &str = "_unpacked";
pub const APP_NAME: &str = "DocPack";

/// Path separators
pub const BACKSLASH: &str = "\\";
pub const FORWARD_SLASH: &str = "/";

/// Windows reserved filenames
pub const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Forbidden path characters
pub const FORBIDDEN_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*', '\0'];

/// Default excluded patterns
pub const DEFAULT_EXCLUDES: &[&str] = &[
    ".*",
    "node_modules",
    "target",
    "Thumbs.db",
    "__pycache__",
    "dist",
    "build",
    "vendor",
    "package-lock.json",
    "yarn.lock",
    "Cargo.lock",
    "*.exe",
    "*.dll",
    "*.so",
    "*.dylib",
    "*.bin",
    "*.pdf",
    "*.zip",
    "*.docx",
    "*.tar",
    "*.gz",
    "*.7z",
    "*.rar",
    "*.png",
    "*.jpg",
    "*.jpeg",
    "*.gif",
    "*.ico",
    "*.svg",
    "*.webp",
    "*.bmp",
    "*.tiff",
    "*.ttf",
    "*.otf",
    "*.woff",
    "*.woff2",
    "*.eot",
    "*.mp3",
    "*.mp4",
    "*.avi",
    "*.mov",
    "*.wav",
    "*.flac",
    "*.aac",
    "*.ogg",
    "*.o",
    "*.obj",
    "*.lib",
    "*.a",
    "*.pyc",
    "*.tmp",
    "*.bak",
    "*.swp",
    "*.swo",
];
