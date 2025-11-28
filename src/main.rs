mod scanner;

use eframe::egui;
use scanner::{scan_directory, FileInfo, ScanProgress};
use std::fs;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([700.0, 500.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "DupeFinder",
        options,
        Box::new(|_cc| Ok(Box::new(DupeFinderApp::default()))),
    )
}

struct DupeFinderApp {
    selected_dir: String,
    scanning: bool,
    scan_progress: Arc<Mutex<Option<ScanProgress>>>,
    duplicate_groups: Vec<DuplicateGroup>,
    total_size_savings: u64,
    result_receiver: Option<Receiver<Vec<Vec<FileInfo>>>>,
    status_message: String,
}

impl Default for DupeFinderApp {
    fn default() -> Self {
        Self {
            selected_dir: String::new(),
            scanning: false,
            scan_progress: Arc::new(Mutex::new(None)),
            duplicate_groups: Vec::new(),
            total_size_savings: 0,
            result_receiver: None,
            status_message: String::new(),
        }
    }
}

struct DuplicateGroup {
    files: Vec<FileInfo>,
    selected: Vec<bool>,
}

impl DupeFinderApp {
    fn start_scan(&mut self, ctx: &egui::Context) {
        if self.selected_dir.is_empty() || self.scanning {
            return;
        }
        
        self.scanning = true;
        self.duplicate_groups.clear();
        self.total_size_savings = 0;
        self.status_message.clear();
        
        let dir = self.selected_dir.clone();
        let progress = self.scan_progress.clone();
        let ctx_clone = ctx.clone();
        
        let (tx, rx) = channel();
        self.result_receiver = Some(rx);
        
        thread::spawn(move || {
            let progress_clone = progress.clone();
            let ctx_clone_2 = ctx_clone.clone();
            let groups = scan_directory(&dir, move |p| {
                *progress_clone.lock().unwrap() = Some(p);
                ctx_clone_2.request_repaint();
            });
            
            *progress.lock().unwrap() = None;
            let _ = tx.send(groups);
            ctx_clone.request_repaint();
        });
    }
    
    fn calculate_savings(&mut self) {
        self.total_size_savings = 0;
        for group in &self.duplicate_groups {
            let files_to_delete: Vec<_> = group.files.iter()
                .zip(&group.selected)
                .filter(|(_, &selected)| !selected)
                .collect();
            
            for (file, _) in files_to_delete {
                self.total_size_savings += file.size;
            }
        }
    }
    
    fn delete_unchecked(&mut self, group_idx: usize) {
        if group_idx >= self.duplicate_groups.len() {
            return;
        }
        
        let group = &self.duplicate_groups[group_idx];
        let mut deleted_count = 0;
        let mut errors = Vec::new();
        
        for (_idx, (file, &keep)) in group.files.iter().zip(&group.selected).enumerate() {
            if !keep {
                match fs::remove_file(&file.path) {
                    Ok(_) => deleted_count += 1,
                    Err(e) => errors.push(format!("Failed to delete {}: {}", file.path.display(), e)),
                }
            }
        }
        
        if errors.is_empty() {
            self.status_message = format!("✓ Deleted {} file(s) from group {}", deleted_count, group_idx + 1);
            self.duplicate_groups.remove(group_idx);
            self.calculate_savings();
        } else {
            self.status_message = format!("⚠ Errors: {}", errors.join("; "));
        }
    }
    
    fn select_newest(&mut self, group_idx: usize) {
        if let Some(group) = self.duplicate_groups.get_mut(group_idx) {
            if let Some((newest_idx, _)) = group.files.iter()
                .enumerate()
                .max_by_key(|(_, f)| fs::metadata(&f.path).ok().and_then(|m| m.modified().ok()))
            {
                for i in 0..group.selected.len() {
                    group.selected[i] = i == newest_idx;
                }
            }
        }
        self.calculate_savings();
    }
    
    fn select_oldest(&mut self, group_idx: usize) {
        if let Some(group) = self.duplicate_groups.get_mut(group_idx) {
            if let Some((oldest_idx, _)) = group.files.iter()
                .enumerate()
                .min_by_key(|(_, f)| fs::metadata(&f.path).ok().and_then(|m| m.modified().ok()))
            {
                for i in 0..group.selected.len() {
                    group.selected[i] = i == oldest_idx;
                }
            }
        }
        self.calculate_savings();
    }
    
    fn bulk_select_newest(&mut self) {
        for group in &mut self.duplicate_groups {
            if let Some((newest_idx, _)) = group.files.iter()
                .enumerate()
                .max_by_key(|(_, f)| fs::metadata(&f.path).ok().and_then(|m| m.modified().ok()))
            {
                for i in 0..group.selected.len() {
                    group.selected[i] = i == newest_idx;
                }
            }
        }
        self.calculate_savings();
    }
    
    fn bulk_select_oldest(&mut self) {
        for group in &mut self.duplicate_groups {
            if let Some((oldest_idx, _)) = group.files.iter()
                .enumerate()
                .min_by_key(|(_, f)| fs::metadata(&f.path).ok().and_then(|m| m.modified().ok()))
            {
                for i in 0..group.selected.len() {
                    group.selected[i] = i == oldest_idx;
                }
            }
        }
        self.calculate_savings();
    }

    fn bulk_delete_unchecked(&mut self) {
        let mut deleted_count = 0;
        let mut errors = Vec::new();
        let mut groups_to_remove = Vec::new();

        for (group_idx, group) in self.duplicate_groups.iter().enumerate() {
            let mut _group_deleted_count = 0;
            for (file, &keep) in group.files.iter().zip(&group.selected) {
                if !keep {
                    match fs::remove_file(&file.path) {
                        Ok(_) => {
                            deleted_count += 1;
                            _group_deleted_count += 1;
                        },
                        Err(e) => errors.push(format!("Failed to delete {}: {}", file.path.display(), e)),
                    }
                }
            }
             groups_to_remove.push(group_idx);
        }

        if errors.is_empty() {
            self.status_message = format!("✓ Bulk deleted {} file(s). All groups cleared.", deleted_count);
            self.duplicate_groups.clear();
            self.calculate_savings();
        } else {
            self.status_message = format!("⚠ Bulk delete finished with {} errors: {}", errors.len(), errors.iter().take(3).cloned().collect::<Vec<_>>().join("; "));
            self.duplicate_groups.clear();
             self.calculate_savings();
        }
    }
}

impl eframe::App for DupeFinderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for scan results
        if let Some(rx) = &self.result_receiver {
            if let Ok(groups) = rx.try_recv() {
                self.duplicate_groups = groups.into_iter()
                    .map(|files| {
                        let selected = vec![true; files.len()];
                        DuplicateGroup { files, selected }
                    })
                    .collect();
                self.scanning = false;
                self.result_receiver = None;
                self.calculate_savings();
                
                if self.duplicate_groups.is_empty() {
                    self.status_message = "No duplicates found.".to_string();
                } else {
                    self.status_message = format!("Found {} duplicate group(s)!", self.duplicate_groups.len());
                }
            }
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🔍 DupeFinder - Rust Duplicate File Finder");
            ui.add_space(10.0);
            
            // Directory selection
            ui.horizontal(|ui| {
                ui.label("Directory:");
                ui.add(egui::TextEdit::singleline(&mut self.selected_dir).desired_width(500.0));
                
                if ui.button("📁 Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.selected_dir = path.display().to_string();
                    }
                }
            });
            
            ui.add_space(10.0);
            
            // Scan button
            ui.horizontal(|ui| {
                if ui.add_enabled(!self.scanning, egui::Button::new("🔍 Scan Directory")).clicked() {
                    self.start_scan(ctx);
                }
                
                if self.scanning {
                    ui.spinner();
                    ui.label("Scanning...");
                }
            });
            
            ui.add_space(10.0);
            
            // Progress bar
            if let Some(progress) = self.scan_progress.lock().unwrap().as_ref() {
                let fraction = progress.current as f32 / progress.total.max(1) as f32;
                ui.add(egui::ProgressBar::new(fraction)
                    .text(format!("{} / {} files", progress.current, progress.total)));
                
                let current_file = &progress.current_file;
                let display_path = if current_file.len() > 80 {
                    format!("...{}", &current_file[current_file.len()-77..])
                } else {
                    current_file.clone()
                };
                ui.label(format!("📄 {}", display_path));
            }
            
            // Status message
            if !self.status_message.is_empty() {
                ui.add_space(5.0);
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), &self.status_message);
            }
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Results
            if !self.duplicate_groups.is_empty() {
                ui.horizontal(|ui| {
                    ui.heading(format!("📊 Found {} duplicate group(s)", self.duplicate_groups.len()));
                    ui.label("|");
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 200, 100),
                        format!("💾 Potential savings: {:.2} MB", self.total_size_savings as f64 / 1_048_576.0)
                    );
                });
                
                ui.add_space(5.0);
                
                // Bulk actions
                ui.horizontal(|ui| {
                    ui.label("Bulk Actions:");
                    if ui.button("📅 Keep Newest in All Groups").clicked() {
                        self.bulk_select_newest();
                    }
                    if ui.button("🕰 Keep Oldest in All Groups").clicked() {
                        self.bulk_select_oldest();
                    }
                    if ui.button("🗑 Delete Unchecked in All Groups").clicked() {
                        self.bulk_delete_unchecked();
                    }
                });
                
                ui.add_space(10.0);
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut group_to_delete = None;
                    let mut recalculate = false;
                    let mut select_newest_for = None;
                    let mut select_oldest_for = None;
                    
                    for (group_idx, group) in self.duplicate_groups.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.strong(format!("Group {} ", group_idx + 1));
                                ui.label(format!("({} files, {:.2} MB each)", 
                                    group.files.len(),
                                    group.files[0].size as f64 / 1_048_576.0
                                ));
                            });
                            
                            ui.add_space(5.0);
                            
                            for (idx, file) in group.files.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    let checkbox_response = ui.checkbox(&mut group.selected[idx], "Keep");
                                    if checkbox_response.changed() {
                                        recalculate = true;
                                    }
                                    ui.label(file.path.display().to_string());
                                });
                            }
                            
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                if ui.button("📅 Keep Newest").clicked() {
                                    select_newest_for = Some(group_idx);
                                }
                                if ui.button("🕰 Keep Oldest").clicked() {
                                    select_oldest_for = Some(group_idx);
                                }
                                if ui.button("🗑 Delete Unchecked").clicked() {
                                    group_to_delete = Some(group_idx);
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                    }
                    
                    if recalculate {
                        self.calculate_savings();
                    }
                    
                    if let Some(idx) = select_newest_for {
                        self.select_newest(idx);
                    }
                    
                    if let Some(idx) = select_oldest_for {
                        self.select_oldest(idx);
                    }
                    
                    if let Some(idx) = group_to_delete {
                        self.delete_unchecked(idx);
                    }
                });
            } else if !self.scanning {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label("Select a directory and click 'Scan Directory' to find duplicate files.");
                    ui.add_space(10.0);
                    ui.label("✓ Uses SHA-256 hashing for accurate detection");
                    ui.label("✓ Fast parallel processing");
                    ui.label("✓ Safe file operations");
                });
            }
        });
    }
}
