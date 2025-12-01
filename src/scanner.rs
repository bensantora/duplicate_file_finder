use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
use sha2::{Sha256, Digest};

#[derive(Clone, Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub struct ScanProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub fn scan_directory<F>(dir: &str, progress_callback: F) -> Vec<Vec<FileInfo>>
where
    F: Fn(ScanProgress) + Send + 'static,
{
    let mut files_by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut total_files = 0;

    // Walk directory, skipping hidden files and directories
    let walker = WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e));

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                let size = metadata.len();
                if size > 0 {
                    files_by_size.entry(size).or_default().push(entry.path().to_path_buf());
                    total_files += 1;
                }
            }
        }
    }

    let potential_duplicates: Vec<_> = files_by_size
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect();

    let mut duplicates: Vec<Vec<FileInfo>> = Vec::new();
    let mut processed_count = 0;

    for (size, paths) in potential_duplicates {
        let mut files_by_hash: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for path in paths {
            processed_count += 1;
            progress_callback(ScanProgress {
                current: processed_count,
                total: total_files,
                current_file: path.display().to_string(),
            });

            if let Ok(hash) = hash_file(&path) {
                files_by_hash.entry(hash).or_default().push(path);
            }
        }

        for (_, paths) in files_by_hash {
            if paths.len() > 1 {
                let group: Vec<FileInfo> = paths
                    .into_iter()
                    .map(|path| FileInfo { path, size })
                    .collect();
                duplicates.push(group);
            }
        }
    }

    duplicates
}

fn hash_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(hex::encode(hasher.finalize()))
}
