use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::theme::{ThemeMode, ThemeColors};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewMode {
    List,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortBy {
    Name,
    Size,
    Type,
    DateModified,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Clone, Debug)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub category: String,  // e.g. "image", "text", "archive", "document", "executable", "other", "folder"
    pub display_type: String, // e.g. "Folder", "Image File", "Text Document"
    pub extension: String, // lowercase extension, e.g. "rs"
}

pub struct FileManagerState {
    pub current_dir: PathBuf,
    pub items: Vec<FileItem>,
    pub selected_idx: Option<usize>,
    pub history: Vec<PathBuf>,
    pub history_idx: usize,
    pub search_query: String,
    pub theme: ThemeMode,
    pub accent_color: String,
    pub highlight_color: String,
    pub show_about: bool,
    pub text_preview: Option<String>,
    pub about_x: f32,
    pub about_y: f32,
    pub last_click_time: Option<std::time::Instant>,
    pub last_clicked_idx: Option<usize>,
    pub renaming_item: Option<PathBuf>,
    pub rename_buffer: String,
    pub delete_confirm: Option<PathBuf>,
    pub sidebar_visible: bool,
    pub clipboard: Option<(Vec<PathBuf>, bool)>, // (Paths, is_cut)
    pub view_mode: ViewMode,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub details_visible: bool,
    pub icon_theme: String,
    pub context_menu_pos: Option<(f32, f32)>,
    pub context_menu_target: Option<usize>,
    pub info_window_target: Option<usize>,
    pub info_window_pos: [f32; 2],
    pub info_window_open: bool,
    pub file_tags: std::collections::HashMap<PathBuf, String>,
    pub folder_color: String,
    pub flat_folders: bool,
    pub sidebar_width: f32,
    pub details_width: f32,
    pub active_resize_sidebar: bool,
    pub active_resize_details: bool,
    pub selected_paths: std::collections::HashSet<PathBuf>,
    pub select_anchor: Option<usize>,
    pub ctrl_pressed: bool,
    pub shift_pressed: bool,
    pub dragging_item: Option<PathBuf>,
    pub drag_pressed_item: Option<PathBuf>,
    pub drag_start_pos: Option<(f32, f32)>,
    pub drag_item_offset: Option<(f32, f32)>,
    pub drag_select_start: Option<(f32, f32)>,
    pub drag_select_current: Option<(f32, f32)>,
    pub item_rects: Vec<(PathBuf, f32, f32, f32, f32)>, // (path, x, y, w, h) screen rects
    pub deferred_click_idx: Option<usize>,
}

impl FileManagerState {
    pub fn colors(&self) -> ThemeColors {
        ThemeColors::resolve(self.theme, &self.accent_color, &self.highlight_color)
    }

    pub fn new() -> Self {
        // Start in user home directory (or current directory fallback)
        let initial_dir = dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));
        
        let mut state = Self {
            current_dir: initial_dir.clone(),
            items: Vec::new(),
            selected_idx: None,
            history: vec![initial_dir],
            history_idx: 0,
            search_query: String::new(),
            theme: ThemeMode::Dark,
            accent_color: "yellow".to_string(),
            highlight_color: "gray".to_string(),
            show_about: true,
            text_preview: None,
            about_x: 390.0,
            about_y: 150.0,
            last_click_time: None,
            last_clicked_idx: None,
            renaming_item: None,
            rename_buffer: String::new(),
            delete_confirm: None,
            sidebar_visible: true,
            clipboard: None,
            view_mode: ViewMode::List,
            sort_by: SortBy::Name,
            sort_order: SortOrder::Ascending,
            details_visible: true,
            icon_theme: "gruvbox".to_string(),
            context_menu_pos: None,
            context_menu_target: None,
            info_window_target: None,
            info_window_pos: [300.0, 200.0],
            info_window_open: false,
            file_tags: std::collections::HashMap::new(),
            folder_color: "gray".to_string(),
            flat_folders: false,
            sidebar_width: 220.0,
            details_width: 300.0,
            active_resize_sidebar: false,
            active_resize_details: false,
            selected_paths: std::collections::HashSet::new(),
            select_anchor: None,
            ctrl_pressed: false,
            shift_pressed: false,
            dragging_item: None,
            drag_pressed_item: None,
            drag_start_pos: None,
            drag_item_offset: None,
            drag_select_start: None,
            drag_select_current: None,
            item_rects: Vec::new(),
            deferred_click_idx: None,
        };

        state.load_tags();
        state.scan_current_dir();
        state
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

    /// Reload the files in the current folder.
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

            // Sort folders and files using chosen sort criteria
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

    /// Retrieve the filtered list of items matching the active search query.
    pub fn get_filtered_items(&self) -> Vec<FileItem> {
        let query = self.search_query.trim().to_lowercase();
        self.items.iter()
            .filter(|item| {
                if query.is_empty() {
                    true
                } else {
                    item.name.to_lowercase().contains(&query)
                }
            })
            .cloned()
            .collect()
    }

    /// Selects an item in the current filtered view, loading text previews if applicable.
    pub fn select_item(&mut self, filtered_idx: usize) {
        let filtered = self.get_filtered_items();
        if filtered_idx < filtered.len() {
            let item_path = filtered[filtered_idx].path.clone();
            
            // Map back to global index in self.items
            if let Some(global_idx) = self.items.iter().position(|it| it.path == item_path) {
                self.selected_idx = Some(global_idx);
                self.text_preview = None;

                let item = &self.items[global_idx];
                if !item.is_dir && item.size < 500_000 {
                    // Try to load a text preview snippet if it's UTF-8 and contains no null bytes
                    if let Ok(content) = std::fs::read_to_string(&item.path) {
                        if !content.contains('\0') {
                            let snippet: String = content.chars().take(300).collect();
                            let display_snippet = if content.chars().count() > 300 {
                                format!("{}...", snippet)
                            } else {
                                snippet
                            };
                            self.text_preview = Some(display_snippet);
                        }
                    }
                }
            }
        }
    }

    /// Navigate to a new directory path.
    pub fn change_dir(&mut self, new_path: PathBuf) {
        if new_path.exists() && new_path.is_dir() {
            // Truncate any forward history
            self.history.truncate(self.history_idx + 1);
            self.history.push(new_path.clone());
            self.history_idx = self.history.len() - 1;
            
            self.current_dir = new_path;
            self.search_query.clear();
            self.clear_selection();
            self.scan_current_dir();
        }
    }

    /// Go back in history.
    pub fn go_back(&mut self) -> bool {
        if self.history_idx > 0 {
            self.history_idx -= 1;
            self.current_dir = self.history[self.history_idx].clone();
            self.search_query.clear();
            self.clear_selection();
            self.scan_current_dir();
            true
        } else {
            false
        }
    }

    /// Go forward in history.
    pub fn go_forward(&mut self) -> bool {
        if self.history_idx + 1 < self.history.len() {
            self.history_idx += 1;
            self.current_dir = self.history[self.history_idx].clone();
            self.search_query.clear();
            self.clear_selection();
            self.scan_current_dir();
            true
        } else {
            false
        }
    }

    /// Go up to parent folder.
    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.change_dir(parent.to_path_buf());
        }
    }

    /// Clear all selected items.
    pub fn clear_selection(&mut self) {
        self.selected_paths.clear();
        self.selected_idx = None;
        self.select_anchor = None;
        self.text_preview = None;
    }

    /// Select a single item (clearing any existing selection).
    pub fn select_single(&mut self, filtered_idx: usize) {
        self.clear_selection();
        let filtered = self.get_filtered_items();
        if filtered_idx < filtered.len() {
            let item_path = filtered[filtered_idx].path.clone();
            self.selected_paths.insert(item_path);
            self.select_anchor = Some(filtered_idx);
            
            // Map back to global index in self.items
            if let Some(global_idx) = self.items.iter().position(|it| it.path == filtered[filtered_idx].path) {
                self.selected_idx = Some(global_idx);
                // Load preview
                self.select_item(global_idx);
            }
        }
    }

    /// Toggle selection state of a single item.
    pub fn toggle_select(&mut self, filtered_idx: usize) {
        let filtered = self.get_filtered_items();
        if filtered_idx < filtered.len() {
            let item_path = filtered[filtered_idx].path.clone();
            if self.selected_paths.contains(&item_path) {
                self.selected_paths.remove(&item_path);
                if self.selected_idx.map(|idx| &self.items[idx].path) == Some(&item_path) {
                    self.selected_idx = None;
                    self.text_preview = None;
                }
            } else {
                self.selected_paths.insert(item_path.clone());
                // Map back to global index in self.items
                if let Some(global_idx) = self.items.iter().position(|it| it.path == item_path) {
                    self.selected_idx = Some(global_idx);
                    // Load preview
                    self.select_item(global_idx);
                }
            }
            self.select_anchor = Some(filtered_idx);
        }
    }

    /// Select a range of items from from_idx to to_idx.
    pub fn select_range(&mut self, from_idx: usize, to_idx: usize) {
        let filtered = self.get_filtered_items();
        let start = from_idx.min(to_idx);
        let end = from_idx.max(to_idx);
        for idx in start..=end {
            if idx < filtered.len() {
                self.selected_paths.insert(filtered[idx].path.clone());
            }
        }
        // Set last clicked/focused
        if to_idx < filtered.len() {
            let item_path = filtered[to_idx].path.clone();
            if let Some(global_idx) = self.items.iter().position(|it| it.path == item_path) {
                self.selected_idx = Some(global_idx);
                self.select_item(global_idx);
            }
        }
    }

    /// Spawn terminal in selected or current folder (cross-platform).
    pub fn open_terminal_in(&self, dir: &std::path::Path) {
        let path_str = dir.to_string_lossy().to_string();
        std::thread::spawn(move || {
            #[cfg(target_os = "windows")]
            {
                // Try Windows Terminal first, fall back to cmd.exe
                if std::process::Command::new("wt")
                    .args(&["-d", &path_str])
                    .spawn()
                    .is_err()
                {
                    let _ = std::process::Command::new("cmd")
                        .args(&["/C", "start", "cmd.exe"])
                        .current_dir(&path_str)
                        .spawn();
                }
            }
            #[cfg(target_os = "macos")]
            {
                let script = format!("tell app \"Terminal\" to do script \"cd '{}'\" activate", path_str);
                let _ = std::process::Command::new("osascript")
                    .args(&["-e", &script])
                    .spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let terminals = [
                    ("kitty", vec!["--directory".to_string(), path_str.clone()]),
                    ("alacritty", vec!["--working-directory".to_string(), path_str.clone()]),
                    ("gnome-terminal", vec!["--working-directory".to_string(), path_str.clone()]),
                    ("konsole", vec!["--workdir".to_string(), path_str.clone()]),
                    ("xterm", vec!["-e".to_string(), "bash".to_string(), "-c".to_string(), format!("cd '{}' && exec bash", path_str)]),
                ];
                for (term, args) in terminals {
                    if std::process::Command::new(term)
                        .args(&args)
                        .spawn()
                        .is_ok()
                    {
                        return;
                    }
                }
            }
        });
    }

    /// Copies a given path string to the system clipboard (cross-platform).
    pub fn copy_path_to_clipboard(&self, path_str: &str) {
        let text = path_str.to_string();
        std::thread::spawn(move || {
            #[cfg(target_os = "windows")]
            {
                use std::io::Write;
                if let Ok(mut child) = std::process::Command::new("clip")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    let _ = child.wait();
                }
            }
            #[cfg(target_os = "macos")]
            {
                use std::io::Write;
                if let Ok(mut child) = std::process::Command::new("pbcopy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    let _ = child.wait();
                }
            }
            #[cfg(target_os = "linux")]
            {
                use std::io::Write;
                // 1. Try wl-copy (Wayland)
                if std::process::Command::new("wl-copy")
                    .arg(&text)
                    .status()
                    .is_ok()
                {
                    return;
                }
                // 2. Try xclip (X11)
                if let Ok(mut child) = std::process::Command::new("xclip")
                    .arg("-selection")
                    .arg("clipboard")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    let _ = child.wait();
                    return;
                }
                // 3. Try xsel (X11 fallback)
                if let Ok(mut child) = std::process::Command::new("xsel")
                    .arg("-ib")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    let _ = child.wait();
                }
            }
        });
    }

    /// Moves a file or folder from src to dest_parent folder.
    pub fn move_item(&mut self, src: &std::path::Path, dest_parent: &std::path::Path) {
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

    /// Copies selected files to state.clipboard
    pub fn copy_selected(&mut self) {
        if !self.selected_paths.is_empty() {
            let paths: Vec<PathBuf> = self.selected_paths.iter().cloned().collect();
            self.clipboard = Some((paths, false));
        }
    }

    /// Cuts selected files to state.clipboard
    pub fn cut_selected(&mut self) {
        if !self.selected_paths.is_empty() {
            let paths: Vec<PathBuf> = self.selected_paths.iter().cloned().collect();
            self.clipboard = Some((paths, true));
        }
    }

    /// Pastes files currently in state.clipboard to self.current_dir
    pub fn paste_clipboard(&mut self) {
        if let Some((src_paths, is_cut)) = self.clipboard.clone() {
            for src_path in src_paths {
                if !src_path.exists() {
                    continue;
                }
                if let Some(filename) = src_path.file_name() {
                    let mut dest_path = self.current_dir.join(filename);
                    if dest_path == src_path {
                        if is_cut {
                            // Moving to same location is a no-op
                            continue;
                        } else {
                            // Copying to same location should generate a unique name
                            let stem = dest_path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                            let ext = dest_path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
                            let mut count = 2;
                            loop {
                                let candidate = self.current_dir.join(format!("{} (copy {}){}", stem, count, ext));
                                if !candidate.exists() {
                                    dest_path = candidate;
                                    break;
                                }
                                count += 1;
                            }
                        }
                    }
                    if is_cut {
                        let _ = std::fs::rename(&src_path, &dest_path);
                    } else {
                        // Cross-platform recursive copy using std::fs
                        copy_recursive(&src_path, &dest_path);
                    }
                }
            }
            if is_cut {
                self.clipboard = None;
            }
            self.scan_current_dir();
        }
    }
}

/// Cross-platform recursive copy of src to dest using only std::fs.
fn copy_recursive(src: &std::path::Path, dest: &std::path::Path) {
    if src.is_dir() {
        let _ = std::fs::create_dir_all(dest);
        if let Ok(entries) = std::fs::read_dir(src) {
            for entry in entries.flatten() {
                let child_src = entry.path();
                let child_dest = dest.join(entry.file_name());
                copy_recursive(&child_src, &child_dest);
            }
        }
    } else {
        let _ = std::fs::copy(src, dest);
    }
}

/// Utility for categorizing files and picking descriptive types
pub fn get_file_info(path: &Path) -> (String, String) {
    if path.is_dir() {
        return ("Folder".to_string(), "folder".to_string());
    }

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 1. Exact match on filename (case-insensitive)
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
        
        // Developer Source Code
        "rs" | "toml" | "json" | "js" | "ts" | "py" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "go" | "html" | "css" | 
        "yml" | "yaml" | "java" | "kt" | "kts" | "cs" | "swift" | "dart" | "php" | "rb" | "scala" | "lua" | "sql" | 
        "r" | "pl" | "pm" | "sh" | "bash" | "zsh" | "tsx" | "jsx" | "asm" | "s" | "assembly" | "diff" | "patch" | "properties" | "gradle" => {
            ("Source Code".to_string(), "text".to_string())
        }
        
        // Configuration / Dotfiles
        "gitignore" | "dockerignore" | "env" | "editorconfig" | "babelrc" | "eslintrc" | "prettierrc" => {
            ("Configuration File".to_string(), "text".to_string())
        }
        
        // Archives
        "zip" | "tar" | "gz" | "bz2" | "xz" | "rar" | "7z" => ("Archive File".to_string(), "archive".to_string()),
        
        // Documents
        "pdf" | "doc" | "docx" | "ppt" | "pptx" => ("Document File".to_string(), "document".to_string()),
        "xls" | "xlsx" | "ods" => ("Spreadsheet".to_string(), "document".to_string()),
        
        // Executables / Scripts
        "bat" | "cmd" | "ps1" => ("Script File".to_string(), "executable".to_string()),
        "exe" | "bin" | "msi" | "app" => ("Executable Program".to_string(), "executable".to_string()),
        
        // Compiled code and Bytecode
        "class" => ("Java Class File".to_string(), "other".to_string()),
        "jar" | "war" => ("Java Archive".to_string(), "archive".to_string()),
        "o" | "obj" | "a" | "lib" | "so" | "dll" | "dylib" => ("Compiled Object / Library".to_string(), "other".to_string()),
        
        _ => ("Generic File".to_string(), "other".to_string()),
    }
}

/// Helper to format byte counts into user-friendly units
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
