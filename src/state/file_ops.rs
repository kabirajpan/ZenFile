use std::path::{Path, PathBuf};
use super::types::{FileManagerState, FileItem, TransferEntry, TransferStatus, SortBy, SortOrder};

impl FileManagerState {
    pub fn scan_current_dir(&mut self) {
        self.items.clear();
        self.selected_idx = None;
        self.text_preview = None;

        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            let mut folders = Vec::new();
            let mut files = Vec::new();

            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown File")
                    .to_string();

                let metadata = entry.metadata().ok();
                let is_dir = path.is_dir();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = metadata.as_ref().and_then(|m| m.modified().ok());

                let (display_type, category) = get_file_info(&path);
                let mut extension = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let name_lower = name.to_lowercase();
                match name_lower.as_str() {
                    "cargo.toml" | "cargo.lock" | "pom.xml" | "build.gradle" | "settings.gradle" | 
                    "package.json" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" | "pnpm-lock.yml" | 
                    "bun.lock" | "bun.lockb" | "makefile" => {
                        extension = name_lower.clone();
                    }
                    _ => {
                        if name_lower.ends_with(".log") || name_lower == "log" || name_lower == "logs" {
                            extension = "log".to_string();
                        } else if name_lower.starts_with(".env") {
                            extension = "env".to_string();
                        } else if extension.is_empty() && name.starts_with('.') {
                            extension = name[1..].to_string();
                        }
                    }
                }

                let item = FileItem {
                    name,
                    path,
                    is_dir,
                    size,
                    modified,
                    category,
                    display_type,
                    extension,
                };

                if is_dir {
                    folders.push(item);
                } else {
                    files.push(item);
                }
            }

            let sort_by = self.sort_by;
            let sort_order = self.sort_order;
            let sort_fn = |a: &FileItem, b: &FileItem| {
                let cmp = match sort_by {
                    SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    SortBy::Size => a.size.cmp(&b.size),
                    SortBy::Type => a.display_type.to_lowercase().cmp(&b.display_type.to_lowercase()),
                    SortBy::DateModified => a.modified.cmp(&b.modified),
                };
                match sort_order {
                    SortOrder::Ascending => cmp,
                    SortOrder::Descending => cmp.reverse(),
                }
            };
            folders.sort_by(sort_fn);
            files.sort_by(sort_fn);

            self.items = folders;
            self.items.extend(files);
        }
    }

    pub fn load_tags(&mut self) {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("zenthra");
        path.push("tags.txt");
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if let Some((p_str, t_str)) = line.split_once('=') {
                    self.file_tags.insert(PathBuf::from(p_str), t_str.to_string());
                }
            }
        }
    }

    pub fn save_tags(&self) {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("zenthra");
        let _ = std::fs::create_dir_all(&path);
        path.push("tags.txt");
        let mut content = String::new();
        for (p, t) in &self.file_tags {
            content.push_str(&format!("{}={}\n", p.display(), t));
        }
        let _ = std::fs::write(path, content);
    }

    pub fn move_item(&mut self, src: &Path, dest_parent: &Path) {
        if !src.exists() || !dest_parent.exists() || !dest_parent.is_dir() {
            return;
        }
        if let Some(filename) = src.file_name() {
            let dest = dest_parent.join(filename);
            if dest != src {
                let _ = std::fs::rename(src, &dest);
                self.scan_current_dir();
            }
        }
    }

    pub fn copy_selected(&mut self) {
        if !self.selected_paths.is_empty() {
            let paths: Vec<PathBuf> = self.selected_paths.iter().cloned().collect();
            self.clipboard = Some((paths, false));
        }
    }

    pub fn cut_selected(&mut self) {
        if !self.selected_paths.is_empty() {
            let paths: Vec<PathBuf> = self.selected_paths.iter().cloned().collect();
            self.clipboard = Some((paths, true));
        }
    }

    pub fn paste_clipboard(&mut self) {
        if let Some((src_paths, is_cut)) = self.clipboard.clone() {
            let mut total_bytes = 0_u64;
            let mut copy_pairs = Vec::new();
            let current_dir = self.current_dir.clone();

            for src_path in &src_paths {
                if !src_path.exists() {
                    continue;
                }
                if let Some(filename) = src_path.file_name() {
                    let mut dest_path = current_dir.join(filename);
                    if dest_path == *src_path {
                        if is_cut {
                            continue;
                        } else {
                            let stem = dest_path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                            let ext = dest_path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
                            let mut count = 2;
                            loop {
                                let candidate = current_dir.join(format!("{} (copy {}){}", stem, count, ext));
                                if !candidate.exists() {
                                    dest_path = candidate;
                                    break;
                                }
                                count += 1;
                            }
                        }
                    }

                    let pairs = expand_copy_paths(src_path, &dest_path);
                    for (s, d) in pairs {
                        total_bytes += std::fs::metadata(&s).map(|m| m.len()).unwrap_or(0);
                        copy_pairs.push((s, d));
                    }
                }
            }

            if copy_pairs.is_empty() {
                return;
            }

            let filename = if src_paths.len() == 1 {
                let label = if is_cut { "Move" } else { "Copy" };
                let name = src_paths[0].file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "item".to_string());
                format!("{}: {}", label, name)
            } else {
                let label = if is_cut { "Moving" } else { "Copying" };
                format!("{} {} items", label, src_paths.len())
            };

            let entry = TransferEntry {
                filename,
                total_bytes,
                sent_bytes: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
                status: std::sync::Arc::new(std::sync::Mutex::new(TransferStatus::Sending)),
                speed_mbps: std::sync::Arc::new(std::sync::Mutex::new(0.0)),
                started_at: std::time::Instant::now(),
                cancel_flag: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            };

            let sent_arc = entry.sent_bytes.clone();
            let status_arc = entry.status.clone();
            let speed_arc = entry.speed_mbps.clone();
            let cancel_arc = entry.cancel_flag.clone();

            self.zendrop_notif_open = true;
            self.zendrop_transfers.insert(0, entry);
            if self.zendrop_transfers.len() > 20 {
                self.zendrop_transfers.truncate(20);
            }

            if is_cut {
                self.clipboard = None;
            }

            let pending_refresh_clone = self.pending_refresh.clone();
            let src_paths_clone = src_paths.clone();

            std::thread::spawn(move || {
                use std::io::{Read, Write};
                let start_time = std::time::Instant::now();
                let mut total_copied = 0_u64;

                for (src, dest) in copy_pairs {
                    if cancel_arc.load(std::sync::atomic::Ordering::SeqCst) {
                        *status_arc.lock().unwrap() = TransferStatus::Failed;
                        return;
                    }

                    if let Some(parent) = dest.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }

                    let rename_ok = if is_cut {
                        std::fs::rename(&src, &dest).is_ok()
                    } else {
                        false
                    };

                    if rename_ok {
                        let size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
                        total_copied += size;
                        sent_arc.store(total_copied, std::sync::atomic::Ordering::SeqCst);
                    } else {
                        let mut f_src = match std::fs::File::open(&src) {
                            Ok(f) => f,
                            Err(_) => {
                                *status_arc.lock().unwrap() = TransferStatus::Failed;
                                return;
                            }
                        };
                        let mut f_dest = match std::fs::File::create(&dest) {
                            Ok(f) => f,
                            Err(_) => {
                                *status_arc.lock().unwrap() = TransferStatus::Failed;
                                return;
                            }
                        };

                        let mut buffer = vec![0u8; 1024 * 1024];
                        let mut copy_ok = false;
                        loop {
                            if cancel_arc.load(std::sync::atomic::Ordering::SeqCst) {
                                drop(f_dest);
                                let _ = std::fs::remove_file(&dest);
                                *status_arc.lock().unwrap() = TransferStatus::Failed;
                                return;
                            }
                            let n = match f_src.read(&mut buffer) {
                                Ok(0) => {
                                    copy_ok = true;
                                    break;
                                }
                                Ok(n) => n,
                                Err(_) => break,
                            };
                            if f_dest.write_all(&buffer[..n]).is_err() {
                                break;
                            }
                            total_copied += n as u64;
                            sent_arc.store(total_copied, std::sync::atomic::Ordering::SeqCst);
                        }

                        if !copy_ok {
                            *status_arc.lock().unwrap() = TransferStatus::Failed;
                            return;
                        }

                        if is_cut {
                            let _ = std::fs::remove_file(&src);
                        }
                    }
                }

                if is_cut {
                    for path in &src_paths_clone {
                        if path.is_dir() {
                            let _ = std::fs::remove_dir_all(path);
                        }
                    }
                }

                let elapsed = start_time.elapsed().as_secs_f32().max(0.001);
                let mb = total_copied as f32 / 1_048_576.0;
                *speed_arc.lock().unwrap() = mb / elapsed;
                *status_arc.lock().unwrap() = TransferStatus::Done;
                sent_arc.store(total_bytes, std::sync::atomic::Ordering::SeqCst);

                pending_refresh_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            });
        }
    }
}

fn expand_copy_paths(src_root: &Path, dest_root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut result = Vec::new();
    let mut stack = vec![(src_root.to_path_buf(), dest_root.to_path_buf())];
    while let Some((src, dest)) = stack.pop() {
        if src.is_file() {
            result.push((src, dest));
        } else if src.is_dir() {
            let _ = std::fs::create_dir_all(&dest);
            if let Ok(entries) = std::fs::read_dir(&src) {
                for entry in entries.flatten() {
                    let child_src = entry.path();
                    let child_dest = dest.join(entry.file_name());
                    stack.push((child_src, child_dest));
                }
            }
        }
    }
    result
}

pub fn get_file_info(path: &Path) -> (String, String) {
    if path.is_dir() {
        return ("Folder".to_string(), "folder".to_string());
    }

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    match file_name.as_str() {
        "cargo.toml" => return ("Rust Cargo Manifest".to_string(), "text".to_string()),
        "cargo.lock" => return ("Cargo Lockfile".to_string(), "text".to_string()),
        "pom.xml" => return ("Maven Project Descriptor".to_string(), "text".to_string()),
        "build.gradle" => return ("Gradle Build Script".to_string(), "text".to_string()),
        "settings.gradle" => return ("Gradle Settings Script".to_string(), "text".to_string()),
        "gradlew" | "gradlew.bat" => return ("Gradle Wrapper".to_string(), "executable".to_string()),
        "package.json" => return ("Node.js Package Manifest".to_string(), "text".to_string()),
        "package-lock.json" => return ("npm Lockfile".to_string(), "text".to_string()),
        "yarn.lock" => return ("Yarn Lockfile".to_string(), "text".to_string()),
        "pnpm-lock.yaml" | "pnpm-lock.yml" => return ("pnpm Lockfile".to_string(), "text".to_string()),
        "bun.lock" | "bun.lockb" => return ("Bun Lockfile".to_string(), "text".to_string()),
        "makefile" => return ("Build Makefile".to_string(), "text".to_string()),
        "license" | "license.txt" | "license.md" | "copying" => return ("License Terms".to_string(), "text".to_string()),
        _ => {}
    }

    if file_name.ends_with(".log") || file_name == "log" || file_name == "logs" {
        return ("Log File".to_string(), "text".to_string());
    }

    if file_name.starts_with(".env") {
        return ("Environment Configuration".to_string(), "text".to_string());
    }

    let mut ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext.is_empty() && file_name.starts_with('.') {
        ext = file_name[1..].to_string();
    }

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" => ("Image File".to_string(), "image".to_string()),
        "txt" | "md" | "csv" => ("Text Document".to_string(), "text".to_string()),
        
        "rs" | "toml" | "json" | "js" | "ts" | "py" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "go" | "html" | "css" | 
        "yml" | "yaml" | "java" | "kt" | "kts" | "cs" | "swift" | "dart" | "php" | "rb" | "scala" | "lua" | "sql" | 
        "r" | "pl" | "pm" | "sh" | "bash" | "zsh" | "tsx" | "jsx" | "asm" | "s" | "assembly" | "diff" | "patch" | "properties" | "gradle" => {
            ("Source Code".to_string(), "text".to_string())
        }
        
        "gitignore" | "dockerignore" | "env" | "editorconfig" | "babelrc" | "eslintrc" | "prettierrc" => {
            ("Configuration File".to_string(), "text".to_string())
        }
        
        "zip" | "tar" | "gz" | "bz2" | "xz" | "rar" | "7z" => ("Archive File".to_string(), "archive".to_string()),
        
        "pdf" | "doc" | "docx" | "ppt" | "pptx" => ("Document File".to_string(), "document".to_string()),
        "xls" | "xlsx" | "ods" => ("Spreadsheet".to_string(), "document".to_string()),
        
        "bat" | "cmd" | "ps1" => ("Script File".to_string(), "executable".to_string()),
        "exe" | "bin" | "msi" | "app" => ("Executable Program".to_string(), "executable".to_string()),
        
        "class" => ("Java Class File".to_string(), "other".to_string()),
        "jar" | "war" => ("Java Archive".to_string(), "archive".to_string()),
        "o" | "obj" | "a" | "lib" | "so" | "dll" | "dylib" => ("Compiled Object / Library".to_string(), "other".to_string()),
        
        _ => ("Generic File".to_string(), "other".to_string()),
    }
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
