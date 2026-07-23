use std::path::{Path, PathBuf};
use super::types::{FileManagerState, FileItem, DeviceDrive};

#[derive(Debug, Clone, Copy)]
pub struct DiskSpaceInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

struct LsblkDevice {
    name: String,
    fstype: String,
    label: String,
    parttype: String,
    mountpoint: String,
    size: String,
    _rm: String,
}

fn parse_lsblk_line(line: &str) -> Option<LsblkDevice> {
    let mut name = String::new();
    let mut fstype = String::new();
    let mut label = String::new();
    let mut parttype = String::new();
    let mut mountpoint = String::new();
    let mut size = String::new();
    let mut _rm = String::new();

    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        
        let mut key = String::new();
        while i < chars.len() && chars[i] != '=' {
            key.push(chars[i]);
            i += 1;
        }
        if i >= chars.len() || chars[i] != '=' {
            break;
        }
        i += 1; // skip '='

        if i >= chars.len() || chars[i] != '"' {
            break;
        }
        i += 1; // skip '"'
        
        let mut val = String::new();
        while i < chars.len() && chars[i] != '"' {
            val.push(chars[i]);
            i += 1;
        }
        if i < chars.len() && chars[i] == '"' {
            i += 1; // skip '"'
        }
        
        match key.trim() {
            "NAME" => name = val,
            "FSTYPE" => fstype = val,
            "LABEL" => label = val,
            "PARTTYPE" => parttype = val,
            "MOUNTPOINT" => mountpoint = val,
            "SIZE" => size = val,
            "RM" => _rm = val,
            _ => {}
        }
    }

    if name.is_empty() {
        None
    } else {
        Some(LsblkDevice { name, fstype, label, parttype, mountpoint, size, _rm })
    }
}

impl FileManagerState {
    pub fn detect_drives(&mut self) -> Vec<DeviceDrive> {
        if self.last_drive_detect_time.elapsed().as_secs() >= 3 {
            self.trigger_drive_refresh();
            self.last_drive_detect_time = std::time::Instant::now();
        }
        
        self.detected_drives.lock().unwrap().clone()
    }

    pub fn trigger_drive_refresh(&self) {
        if self.is_detecting_drives.swap(true, std::sync::atomic::Ordering::SeqCst) == false {
            let drives_arc = self.detected_drives.clone();
            let is_detecting_arc = self.is_detecting_drives.clone();
            
            std::thread::spawn(move || {
                let mut drives = Vec::new();
                
                #[cfg(target_os = "windows")]
                {
                    for letter in b'A'..=b'Z' {
                        let drive_path = PathBuf::from(format!("{}:\\", letter as char));
                        if drive_path.exists() {
                            let name = if letter as char == 'C' {
                                "System Disk (C:)".to_string()
                            } else {
                                format!("Local Disk ({}:)", letter as char)
                            };
                            drives.push(DeviceDrive {
                                name,
                                path: drive_path,
                                is_mounted: true,
                                device_node: "".to_string(),
                            });
                        }
                    }
                }

                #[cfg(target_os = "macos")]
                {
                    drives.push(DeviceDrive {
                        name: "Macintosh HD".to_string(),
                        path: PathBuf::from("/"),
                        is_mounted: true,
                        device_node: "".to_string(),
                    });
                    if let Ok(entries) = std::fs::read_dir("/Volumes") {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "Volume".to_string());
                                drives.push(DeviceDrive {
                                    name,
                                    path,
                                    is_mounted: true,
                                    device_node: "".to_string(),
                                });
                            }
                        }
                    }
                }

                #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                {
                    drives.push(DeviceDrive {
                        name: "System Disk".to_string(),
                        path: PathBuf::from("/"),
                        is_mounted: true,
                        device_node: "".to_string(),
                    });
                    
                    let output = std::process::Command::new("lsblk")
                        .args(&["-P", "-o", "NAME,FSTYPE,LABEL,PARTTYPE,MOUNTPOINT,SIZE,RM"])
                        .output();
                    if let Ok(out) = output {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        for line in stdout.lines() {
                            if let Some(dev) = parse_lsblk_line(line) {
                                if dev.fstype.is_empty() || dev.fstype == "swap" || dev.fstype == "cryptswap" {
                                    continue;
                                }
                                if dev.parttype == "c12a7328-f81f-11d2-ba4b-00a0c93ec93b" {
                                    continue;
                                }
                                if dev.mountpoint == "/" || dev.mountpoint == "/boot/efi" || dev.mountpoint == "/recovery" || dev.mountpoint.starts_with("[SWAP]") {
                                    continue;
                                }

                                let dev_path = format!("/dev/{}", dev.name);
                                let is_mounted = !dev.mountpoint.is_empty();
                                let path = if is_mounted {
                                    PathBuf::from(dev.mountpoint)
                                } else {
                                    PathBuf::from(&dev_path)
                                };

                                let label = if !dev.label.is_empty() {
                                    dev.label
                                } else {
                                    format!("Local Disk ({})", dev.name)
                                };

                                let display_name = format!("{} ({})", label, dev.size);

                                drives.push(DeviceDrive {
                                    name: display_name,
                                    path,
                                    is_mounted,
                                    device_node: dev_path,
                                });
                            }
                        }
                    }

                    if drives.len() <= 1 {
                        if let Ok(entries) = std::fs::read_dir("/media") {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_dir() {
                                    if let Ok(mounts) = std::fs::read_dir(&path) {
                                        for mount in mounts.flatten() {
                                            let mount_path = mount.path();
                                            let name = mount_path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "External Drive".to_string());
                                            drives.push(DeviceDrive {
                                                name,
                                                path: mount_path,
                                                is_mounted: true,
                                                device_node: "".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        if let Ok(entries) = std::fs::read_dir("/run/media") {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if path.is_dir() {
                                    if let Ok(mounts) = std::fs::read_dir(&path) {
                                        for mount in mounts.flatten() {
                                            let mount_path = mount.path();
                                            let name = mount_path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "External Drive".to_string());
                                            drives.push(DeviceDrive {
                                                name,
                                                path: mount_path,
                                                is_mounted: true,
                                                device_node: "".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Ok(mut guard) = drives_arc.lock() {
                    *guard = drives;
                }
                is_detecting_arc.store(false, std::sync::atomic::Ordering::SeqCst);
            });
        }
    }

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

    pub fn select_item(&mut self, filtered_idx: usize) {
        let filtered = self.get_filtered_items();
        if filtered_idx < filtered.len() {
            let item_path = filtered[filtered_idx].path.clone();
            
            if let Some(global_idx) = self.items.iter().position(|it| it.path == item_path) {
                self.selected_idx = Some(global_idx);
                self.text_preview = None;

                let item = &self.items[global_idx];
                if !item.is_dir && item.size < 500_000 {
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

    pub fn change_dir(&mut self, new_path: PathBuf) {
        if new_path.exists() && new_path.is_dir() {
            self.history.truncate(self.history_idx + 1);
            self.history.push(new_path.clone());
            self.history_idx = self.history.len() - 1;
            
            self.current_dir = new_path;
            self.search_query.clear();
            self.clear_selection();
            self.scan_current_dir();
        }
    }

    pub fn mount_and_change_dir_by_dev(&mut self, dev_path: &Path) {
        let dev_str = dev_path.to_string_lossy();
        let output = std::process::Command::new("udisksctl")
            .args(&["mount", "-b", &dev_str])
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            
            let mut mount_point = None;
            if let Some(pos) = stdout.find(" at ") {
                mount_point = Some(stdout[pos + 4..].trim().to_string());
            } else if let Some(pos) = stderr.find(" mounted at ") {
                mount_point = Some(stderr[pos + 12..].trim().to_string());
            }
            
            if let Some(mp) = mount_point {
                let path = PathBuf::from(mp);
                if path.exists() && path.is_dir() {
                    self.change_dir(path);
                }
            }
        }
        self.trigger_drive_refresh();
    }

    pub fn unmount_dev(&mut self, dev_str: &str) {
        let output = std::process::Command::new("udisksctl")
            .args(&["unmount", "-b", dev_str])
            .output();
        if let Ok(_) = output {
            if !self.current_dir.exists() {
                let fallback = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
                self.change_dir(fallback);
            } else {
                self.scan_current_dir();
            }
        }
        self.trigger_drive_refresh();
    }

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

    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.change_dir(parent.to_path_buf());
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_paths.clear();
        self.selected_idx = None;
        self.select_anchor = None;
        self.text_preview = None;
    }

    pub fn select_single(&mut self, filtered_idx: usize) {
        self.clear_selection();
        let filtered = self.get_filtered_items();
        if filtered_idx < filtered.len() {
            let item_path = filtered[filtered_idx].path.clone();
            self.selected_paths.insert(item_path);
            self.select_anchor = Some(filtered_idx);
            
            if let Some(global_idx) = self.items.iter().position(|it| it.path == filtered[filtered_idx].path) {
                self.selected_idx = Some(global_idx);
                self.select_item(filtered_idx);
            }
        }
    }

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
                if let Some(global_idx) = self.items.iter().position(|it| it.path == item_path) {
                    self.selected_idx = Some(global_idx);
                    self.select_item(filtered_idx);
                }
            }
            self.select_anchor = Some(filtered_idx);
        }
    }

    pub fn select_range(&mut self, from_idx: usize, to_idx: usize) {
        let filtered = self.get_filtered_items();
        let start = from_idx.min(to_idx);
        let end = from_idx.max(to_idx);
        for idx in start..=end {
            if idx < filtered.len() {
                self.selected_paths.insert(filtered[idx].path.clone());
            }
        }
        if to_idx < filtered.len() {
            let item_path = filtered[to_idx].path.clone();
            if let Some(global_idx) = self.items.iter().position(|it| it.path == item_path) {
                self.selected_idx = Some(global_idx);
                self.select_item(to_idx);
            }
        }
    }

    pub fn open_terminal_in(&self, dir: &Path) {
        let path_str = dir.to_string_lossy().to_string();
        std::thread::spawn(move || {
            #[cfg(target_os = "windows")]
            {
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
                if std::process::Command::new("wl-copy")
                    .arg(&text)
                    .status()
                    .is_ok()
                {
                    return;
                }
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
}

pub fn query_disk_space(path: &Path) -> Option<DiskSpaceInfo> {
    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("df")
            .args(&["-B1", path.to_str()?])
            .output()
            .ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().nth(1)?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let total = parts[1].parse::<u64>().ok()?;
            let used = parts[2].parse::<u64>().ok()?;
            let free = parts[3].parse::<u64>().ok()?;
            Some(DiskSpaceInfo { total, used, free })
        } else {
            None
        }
    }
    #[cfg(target_os = "windows")]
    {
        None
    }
}

pub fn query_cpu_usage() -> f32 {
    if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = loadavg.split_whitespace().collect();
        if !parts.is_empty() {
            if let Ok(val) = parts[0].parse::<f32>() {
                let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4) as f32;
                return (val / cores * 100.0).clamp(0.0, 100.0);
            }
        }
    }
    25.0
}

pub fn query_ram_usage() -> Option<(u64, u64)> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0;
    let mut available = 0;
    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                total = parts[1].parse::<u64>().ok()? * 1024;
            }
        } else if line.starts_with("MemAvailable:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                available = parts[1].parse::<u64>().ok()? * 1024;
            }
        }
    }
    if total > 0 {
        Some((total - available, total))
    } else {
        None
    }
}
