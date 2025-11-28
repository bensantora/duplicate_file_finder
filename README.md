# Duplicate File Finder

A simple cross-platform GUI application written in Rust that finds duplicate files by size and SHA-256 content hash. Built with eframe/egui for the user interface, WalkDir for filesystem traversal, and sha2 for hashing. DupeFinder helps you identify duplicate files and safely remove unwanted copies.

---

## Table of contents

- [Features](#features)
- [Screenshot](#screenshot)
- [Why use this tool](#why-use-this-tool)
- [How it works](#how-it-works)
- [Requirements](#requirements)
- [Build & run](#build--run)
- [Usage](#usage)
- [Safety notes](#safety-notes)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

---

## Features

- GUI built with eframe / egui for a native desktop experience
- Traverses directories recursively (walkdir)
- Groups files by size to avoid unnecessary hashing
- Uses SHA-256 to detect identical file contents
- Displays scan progress including current file and progress bar
- Per-group and bulk actions:
  - Keep newest / oldest in a group
  - Toggle individual files as "Keep"
  - Delete unchecked files for a group or all groups
- Shows estimated potential disk space savings
- Safe deletion behavior with UI feedback and error reporting

---

## Why use this tool

- Fast: avoids hashing files of unique sizes and hashes only plausible duplicates
- Accurate: uses a strong SHA-256 hash for content equality
- User friendly: visual UI makes it simple to inspect duplicates before deleting
- Cross-platform: runs anywhere Rust and the used crates support (Windows, macOS, Linux)

---

## How it works (high level)

1. Walk the selected directory and collect files grouped by size.
2. Ignore sizes with only one file (cannot be duplicates).
3. For each size with multiple files, compute SHA-256 hashes and group by hash.
4. Present groups with more than one file to the user.
5. User selects which files to keep; unchecked files are candidates for deletion.
6. Deletions are performed via standard filesystem calls and errors are reported in the UI.

---

## Requirements

- Rust toolchain (1.70+ recommended)
- Platform-specific GUI dependencies (handled by the used crates; examples below)
- Crates used (Cargo.toml should include):
  - eframe
  - egui
  - walkdir
  - sha2
  - hex
  - rfd

---

## Build & run

Clone the repository and use cargo to build or run.

1. Clone:
   git clone https://github.com/your-username/dupefinder.git
   cd dupefinder

2. Run in debug:
   cargo run

3. Build a release binary:
   cargo build --release

4. Run the release binary:
   ./target/release/dupefinder

Notes:
- For the best performance, use `--release`.
- On some platforms (notably macOS), you may need extra build tools. The Rust toolchain and a system C linker are typically sufficient.

---

## Usage

1. Start the app.
2. Click "Browse" and select the directory you want to scan.
3. Click "Scan Directory".
4. While scanning, a progress bar and current file path are shown.
5. After scanning, duplicate groups are displayed.
6. For each group:
   - Use the "Keep" checkboxes to mark files you want to retain.
   - Use "Keep Newest" or "Keep Oldest" to auto-select one copy.
   - Click "Delete Unchecked" to remove the unchecked files in that group.
7. Use bulk actions to apply "Keep Newest / Oldest" or "Delete Unchecked" across all groups.

The status area shows success messages or errors from failed filesystem operations.

---

## Safety notes

- Deletion is irreversible via the app. Please make sure:
  - You have backups of important data, or
  - You review duplicates carefully before deleting.
- Consider running a scan and inspecting groups without deleting first.
- The app attempts file deletions using standard filesystem APIs; permission errors or locked files will be reported.
- The algorithm groups by size first and then uses SHA-256; this reduces false positives and avoids unnecessary hashing.

---

## Troubleshooting

- Long scan times:
  - Scanning and hashing large directories with many files can take time — use Release mode for better speed.
  - Exclude large read-only directories if not needed.
- Permissions errors when deleting:
  - Run the app with appropriate permissions or change file permissions.
  - Locked files (e.g., open by another program) may fail to delete.
- Missing GUI on headless systems:
  - eframe/egui apps require a desktop environment. Use a machine with a graphical session.
- High memory or CPU usage:
  - Hashing is CPU-bound. Run in release mode to minimize overhead. Consider scanning smaller batches.


## Contributing

Contributions are welcome! Suggested ways to contribute:

- Improve UI/UX (sorting, grouping, previews)
- Add a dry-run / "move to trash" option instead of permanent deletion
- Add an exclusion list for file patterns or paths
- Improve progress reporting (accurate total hashed vs. estimated)
- Add unit and integration tests

## License

MIT License 

---

## Acknowledgements

- eframe / egui — immediate-mode GUI in Rust
- walkdir — recursive directory traversal
- sha2 — SHA-256 hashing
- hex — hex encoding of hashes
- rfd — native file dialogs
