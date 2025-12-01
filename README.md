# Duplicate File Finder

Written in Rust 

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

*** Be careful when bulk deleting files that you do not delete system or other important files!***

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

## License

MIT License 

---

## Acknowledgements

- eframe / egui — immediate-mode GUI in Rust
- walkdir — recursive directory traversal
- sha2 — SHA-256 hashing
- hex — hex encoding of hashes
- rfd — native file dialogs
