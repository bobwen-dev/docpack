# DocPack

Pack an entire directory tree into a single DOCX document — and unpack it back.

[![Rust](https://img.shields.io/badge/rust-2021-edition-blue)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)]()

## Why

Web-based AI assistants (ChatGPT, Claude, Gemini, DeepSeek, Kimi, Wenxin Yiyan, Tongyi Qianwen, Doubao, and others) accept DOCX file uploads and parse their content natively. But they can't read an entire project directory in one shot — you either paste files one by one or zip them, which most chat UIs don't preview.

DocPack bundles an entire codebase into a single DOCX that you can drag into any chat window. Every file becomes a section heading followed by its full content, ready for the AI to read, search, and reason about your project as a whole.

## Installation

### From source

```bash
git clone https://github.com/anomalyco/docpack
cd docpack
cargo build --release
```

The binary is at `target/release/docpack` (or `docpack.exe` on Windows).

### Windows context menu (optional)

```bash
docpack install      # adds right-click menus
docpack uninstall    # removes them
```

After install, right-click any folder or file to "Pack with DocPack", or right-click any `.docx` to "Unpack DOCX here".

## Usage

### GUI

Double-click `docpack.exe` (or run `docpack` with no arguments) to launch the graphical interface. Drag files or folders onto the window, optionally configure exclude lists and output path, then click **Pack**.

### CLI

```bash
# Pack a directory
docpack pack ./src

# Pack with explicit output
docpack pack ./src -o archive.docx

# Pack specific files
docpack pack main.rs lib.rs -o code.docx

# Pack with a custom exclude file
docpack pack . --exclude .docpackignore

# Unpack a DOCX
docpack unpack archive.docx

# Unpack to a specific directory
docpack unpack archive.docx -o ./output
```

### Exclude Rules

Create a `.docpackignore` file (gitignore syntax) in the project root:

```gitignore
# Built artifacts
node_modules/
target/
dist/

# Binary files
*.png
*.pdf
!important.png
```

## Features

- **Text auto-detection** — distinguishes text from binary files using BOM detection, encoding validation, and meaningful-character heuristics. Binary files are skipped with a count shown.
- **Multi-encoding support** — reads files in UTF-8, GBK, SHIFT_JIS, EUC-KR, and more (configurable). Falls back to lossy UTF-8 for unreadable files.
- **DOCX roundtrip** — files packed by DocPack can be unpacked back to their original filenames and contents. File paths use `/` separators for cross-platform compatibility.
- **Auto-rename** — if the output file or unpack directory already exists, DocPack appends `_1`, `_2`, etc. instead of overwriting.
- **Cross-platform paths** — headings in the DOCX always use `/` separators, so a DOCX created on Windows unpacks correctly on macOS and Linux.
- **i18n** — UI available in English, 简体中文, and 繁體中文.
- **No dependencies** — single binary, no runtime, no install-anything.

## Settings

Settings are stored at:

| Platform | Path |
|----------|------|
| Windows  | `%APPDATA%/docpack/settings.json` |
| Linux    | `~/.config/docpack/settings.json` |
| macOS    | `~/.config/docpack/settings.json` |

Configurable via the GUI Settings panel:

- **Language** — en / zh-CN / zh-TW
- **Exclude list** — gitignore-style patterns, one per line
- **Local encodings** — fallback encodings when UTF-8 fails
- **Context menu** — install / uninstall Windows right-click entries

## Build

```bash
cargo build --release
```

## License

MIT

## Contributing

Issues and PRs welcome. Before submitting, please:
1. Run `cargo test` — 127+ tests must pass
2. Run `cargo build --release` on your target platform
