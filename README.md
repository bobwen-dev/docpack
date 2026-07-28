<!--
docpack: v1.0.0
lang: Rust
edition: 2021
crate-type: library + binary
keywords: docx, pack, unpack, codebase, ai-context, chatgpt, claude
-->

# DocPack

**Pack an entire project directory into a single DOCX document — drag it into any AI chat — and unpack it back to files.**

[![Rust 2021](https://img.shields.io/badge/rust-2021_edition-blue)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/bobwen-dev/docpack/releases)
[![GitHub release](https://img.shields.io/github/v/release/bobwen-dev/docpack)](https://github.com/bobwen-dev/docpack/releases)
[![MIT license](https://img.shields.io/github/license/bobwen-dev/docpack)](LICENSE)

- [Features](#features)
- [Quick Start](#quick-start)
- [Why](#why)
- [Installation](#installation)
- [Usage](#usage)
- [Project Structure](#project-structure)
- [Output Example](#output-example)
- [Exclude Rules](#exclude-rules)
- [Settings](#settings)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- **One-shot AI context** — bundle your whole codebase into a single DOCX and drag it into ChatGPT, Claude, Gemini, DeepSeek, Kimi, or any chat UI that accepts DOCX files.
- **Text auto-detection** — distinguishes text from binary files using BOM detection, encoding validation, and meaningful-character heuristics. Binary files are skipped with a count shown.
- **Multi-encoding support** — reads files in UTF-8, GBK, SHIFT_JIS, EUC-KR, and more (configurable). Falls back to lossy UTF-8 for unreadable files.
- **DOCX roundtrip** — files packed by DocPack can be unpacked back to their original filenames and contents. File paths use `/` separators for cross-platform compatibility.
- **Auto-rename** — if the output file or unpack directory already exists, DocPack appends `_1`, `_2`, etc. instead of overwriting.
- **Cross-platform paths** — headings in the DOCX always use `/` separators, so a DOCX created on Windows unpacks correctly on macOS and Linux.
- **i18n** — UI available in English, 简体中文, and 繁體中文.
- **Single binary** — no runtime, no dependencies, no install-anything.

## Quick Start

```bash
# 1. Download a binary from the [Releases page](https://github.com/bobwen-dev/docpack/releases)
# 2. Pack the current directory into a DOCX
docpack pack . -o project.docx

# 3. Drag project.docx into your AI chat — done.
```

## Why

Web-based AI assistants accept DOCX file uploads and parse their content natively. But they can't read an entire project directory in one shot — you either paste files one by one or zip them, which most chat UIs don't preview.

DocPack converts any directory tree into a DOCX document. Every file becomes a heading (its relative path) followed by its full content, ready for the AI to read, search, and reason about your project as a whole.

## Installation

### From source

```bash
git clone https://github.com/bobwen-dev/docpack
cd docpack
cargo build --release
```

The binary is at `target/release/docpack` (or `docpack.exe` on Windows).

Cross-platform builds:
- **Linux / macOS**: `cargo build --release`
- **Windows**: `cargo build --release` (with MSVC or GNU toolchain)

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

## Project Structure
```
src/
├── lib.rs          # Public API — re-exports key types and functions
├── main.rs         # Binary entry point
├── cli.rs          # CLI argument parser (clap)
├── gui.rs          # GUI window (eframe/egui)
├── pack.rs         # Collect files from directory, detect text vs binary
├── unpack.rs       # Extract DOCX back to original file tree
├── docx/
│   ├── writer.rs   # Build DOCX: ZIP archive + Open XML parts
│   ├── reader.rs   # Parse DOCX back into Document model
│   ├── model.rs    # Document, Paragraph, Run data types
│   └── style_gen.rs# Heading and paragraph style generation
├── ignore.rs       # Gitignore-style pattern matching for excludes
├── settings.rs     # User settings load/save
├── lang.rs         # i18n: English, 简体中文, 繁體中文
├── style.rs        # DOCX style constants
├── icon_bytes.rs   # Embedded application icon
├── constants.rs    # Shared constants
└── platform/       # Platform-specific features (Windows context menu)
```

## Output Example

When you pack a project, the resulting DOCX is a flat sequence — each file becomes a Heading 1 (relative path) followed by its content as normal paragraphs:

```
Heading 1: "README.md"
  [full content of README.md]

Heading 1: "Cargo.toml"
  [full content of Cargo.toml]

Heading 1: "src/main.rs"
  [full content of src/main.rs]

Heading 1: "src/lib.rs"
  [full content of src/lib.rs]
```

There are no directory headings, no multi-level headings, and no nested tree — every file is at the same level.

## Exclude Rules

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

## Contributing

Issues and PRs welcome. Before submitting, please:

1. Run `cargo test` — all tests must pass
2. Run `cargo build --release` on your target platform

## Related

Prefer **PDF** instead of DOCX? Check out [pack2pdf](https://github.com/bobwen-dev/pack2pdf) — same idea, PDF format with embedded images and CJK support.

## License

MIT
