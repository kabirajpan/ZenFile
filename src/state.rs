use std::path::{Path, PathBuf};
use std::time::SystemTime;
use crate::theme::{ThemeMode, ThemeColors};

#[derive(Debug, Clone)]
pub struct DeviceDrive {
    pub name: String,
    pub path: PathBuf,
    pub is_mounted: bool,
    pub device_node: String,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveScreen {
    Browser,
    Dashboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZendropSendMode {
    Files,
    Clipboard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DashboardSelection {
    None,
    Drive(std::path::PathBuf),
    Cpu,
    Ram,
    Directory(std::path::PathBuf),
}

pub struct FileManagerState {
    pub active_screen: ActiveScreen,
    pub current_dir: PathBuf,
    pub items: Vec<FileItem>,
    pub selected_idx: Option<usize>,
    pub history: Vec<PathBuf>,
    pub history_idx: usize,
    pub search_query: String,
    pub theme: ThemeMode,
    pub accent_color: String,
    pub highlight_color: String,
    pub glassmorphism_glow1_color: String,
    pub glassmorphism_glow2_color: String,
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
    pub alt_pressed: bool,
    pub super_pressed: bool,
    pub dragging_item: Option<PathBuf>,
    pub drag_pressed_item: Option<PathBuf>,
    pub drag_start_pos: Option<(f32, f32)>,
    pub drag_item_offset: Option<(f32, f32)>,
    pub drag_select_start: Option<(f32, f32)>,
    pub drag_select_current: Option<(f32, f32)>,
    pub item_rects: Vec<(PathBuf, f32, f32, f32, f32)>, // (path, x, y, w, h) screen rects
    pub deferred_click_idx: Option<usize>,
    pub detected_drives: std::sync::Arc<std::sync::Mutex<Vec<DeviceDrive>>>,
    pub is_detecting_drives: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub last_drive_detect_time: std::time::Instant,
    pub dashboard_selection: DashboardSelection,
    // ZenDrop
    pub zendrop_open: bool,
    pub zendrop_devices: Vec<ZendropDevice>,
    pub last_zendrop_scan: Option<std::time::Instant>,
    pub zendrop_btn_pos: Option<(f32, f32)>,
    pub zendrop_network_name: String,
    pub zendrop_networks: Vec<WifiNetwork>,
    // Dialog / Interaction states
    pub wifi_connect_ssid: Option<String>,
    pub wifi_connect_password: String,
    pub wifi_connect_error: Option<String>,
    pub wifi_connecting: bool,
    pub zendrop_send_target: Option<ZendropDevice>,
    pub zendrop_send_progress: f32,
    pub zendrop_send_status: String,
    pub wifi_connection_result: std::sync::Arc<std::sync::Mutex<Option<Result<String, String>>>>,
    pub zendrop_progress: std::sync::Arc<std::sync::atomic::AtomicU32>,
    pub zendrop_error: Option<String>,
    pub zendrop_show_devices: bool,
    pub zendrop_status_msg: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    pub zendrop_send_mode: ZendropSendMode,
}

#[derive(Debug, Clone)]
pub struct ZendropDevice {
    pub ip: String,
    pub mac: String,
    pub hostname: String,
    pub iface: String,
}

#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub active: bool,
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
    pub fn colors(&self) -> ThemeColors {
        ThemeColors::resolve(self.theme, &self.accent_color, &self.highlight_color)
    }

    pub fn refresh_zendrop(&mut self) {
        self.zendrop_devices = scan_zendrop_devices();
        self.zendrop_networks = scan_wifi_networks();
        self.zendrop_network_name = query_network_name();
        self.last_zendrop_scan = Some(std::time::Instant::now());
    }
    pub fn disconnect_from_wifi(&mut self, ssid: String) {
        self.wifi_connecting = true;
        self.wifi_connect_error = None;
        
        let ssid_clone = ssid.clone();
        let result_arc = self.wifi_connection_result.clone();
        
        std::thread::spawn(move || {
            let mut cmd = std::process::Command::new("nmcli");
            cmd.args(&["connection", "down", "id", &ssid_clone]);
            
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        *result_arc.lock().unwrap() = Some(Ok(String::new()));
                    } else {
                        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
                        let err = if err.is_empty() {
                            String::from_utf8_lossy(&output.stdout).trim().to_string()
                        } else {
                            err
                        };
                        *result_arc.lock().unwrap() = Some(Err(if err.is_empty() { "Disconnect failed".to_string() } else { err }));
                    }
                }
                Err(e) => {
                    *result_arc.lock().unwrap() = Some(Err(e.to_string()));
                }
            }
        });
    }
    pub fn connect_to_wifi(&mut self, ssid: String, password: Option<String>) {
        self.wifi_connecting = true;
        self.wifi_connect_error = None;
        
        let ssid_clone = ssid.clone();
        let pass_clone = password;
        let result_arc = self.wifi_connection_result.clone();
        
        std::thread::spawn(move || {
            let mut cmd = std::process::Command::new("nmcli");
            cmd.args(&["dev", "wifi", "connect", &ssid_clone]);
            if let Some(ref pass) = pass_clone {
                if !pass.is_empty() {
                    cmd.args(&["password", pass]);
                }
            }
            
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        *result_arc.lock().unwrap() = Some(Ok(ssid_clone));
                    } else {
                        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
                        let err = if err.is_empty() {
                            String::from_utf8_lossy(&output.stdout).trim().to_string()
                        } else {
                            err
                        };
                        *result_arc.lock().unwrap() = Some(Err(if err.is_empty() { "Connection failed".to_string() } else { err }));
                    }
                }
                Err(e) => {
                    *result_arc.lock().unwrap() = Some(Err(e.to_string()));
                }
            }
        });
    }

    pub fn start_zendrop_send(&mut self, device: ZendropDevice) {
        self.zendrop_send_target = Some(device.clone());
        self.zendrop_send_progress = 0.0;
        self.zendrop_send_status = "Connecting to device...".to_string();

        let progress_arc = self.zendrop_progress.clone();
        progress_arc.store(0, std::sync::atomic::Ordering::SeqCst);

        // Get files to send
        let mut files_to_send = Vec::new();
        let is_clipboard = self.zendrop_send_mode == ZendropSendMode::Clipboard;
        if is_clipboard {
            if let Some(clip_text) = read_from_clipboard() {
                let temp_file = std::env::temp_dir().join("Clipboard_Content.txt");
                if std::fs::write(&temp_file, clip_text).is_ok() {
                    files_to_send.push(temp_file);
                }
            }
            if files_to_send.is_empty() {
                self.zendrop_send_status = "Clipboard is empty or could not be read".to_string();
                return;
            }
        } else {
            if self.selected_paths.is_empty() {
                files_to_send.push(self.current_dir.clone());
            } else {
                files_to_send = self.selected_paths.iter().cloned().collect();
            }
        }

        let ip = device.ip.clone();
        let status_msg_clone = self.zendrop_status_msg.clone();

        std::thread::spawn(move || {
            use std::io::{Write, Read};
            use std::net::TcpStream;

            // Collect files recursively
            let mut files = Vec::new();
            for path in files_to_send {
                let parent = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| path.clone());
                let _ = collect_files_recursive(path, &parent, &mut files);
            }

            if files.is_empty() {
                *status_msg_clone.lock().unwrap() = Some("No files found to send".to_string());
                progress_arc.store(100, std::sync::atomic::Ordering::SeqCst);
                return;
            }

            // Calculate total size
            let mut total_bytes = 0_u64;
            for (_, path) in &files {
                if let Ok(meta) = std::fs::metadata(path) {
                    total_bytes += meta.len();
                }
            }

            let mut sent_bytes = 0_u64;

            // Send files (each file opens a new connection to port 8888)
            for (idx, (rel_name, path)) in files.iter().enumerate() {
                *status_msg_clone.lock().unwrap() = Some(format!("Sending ({}/{}): {}", idx + 1, files.len(), rel_name));

                let mut stream = match TcpStream::connect((ip.as_str(), 8888)) {
                    Ok(s) => s,
                    Err(e) => {
                        *status_msg_clone.lock().unwrap() = Some(format!("Connection failed: {}", e));
                        progress_arc.store(100, std::sync::atomic::Ordering::SeqCst);
                        return;
                    }
                };

                let mut file = match std::fs::File::open(path) {
                    Ok(f) => f,
                    Err(_) => continue,
                };

                let file_len = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

                // Write Java DataOutputStream Header format:
                // 1. writeUTF: 2 bytes string length (u16 big-endian) + UTF-8 string bytes
                let name_bytes = rel_name.as_bytes();
                if name_bytes.len() > 65535 {
                    continue; // skip files exceeding name length limit
                }
                let name_len = name_bytes.len() as u16;
                if stream.write_all(&name_len.to_be_bytes()).is_err() { break; }
                if stream.write_all(name_bytes).is_err() { break; }

                // 2. writeLong: 8 bytes file length (u64 big-endian)
                if stream.write_all(&file_len.to_be_bytes()).is_err() { break; }

                // 3. writeBoolean: 1 byte (0x00 for false / no compression)
                if stream.write_all(&[0x00]).is_err() { break; }

                // Write content
                let mut buffer = vec![0; 64 * 1024];
                let mut file_sent = 0_u64;
                while file_sent < file_len {
                    let read_len = match file.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(_) => break,
                    };
                    if stream.write_all(&buffer[..read_len]).is_err() { break; }
                    file_sent += read_len as u64;
                    sent_bytes += read_len as u64;

                    // Update progress
                    let pct = if total_bytes > 0 {
                        ((sent_bytes as f64 / total_bytes as f64) * 100.0) as u32
                    } else {
                        100
                    };
                    progress_arc.store(pct.min(99), std::sync::atomic::Ordering::SeqCst);
                }
            }

            progress_arc.store(100, std::sync::atomic::Ordering::SeqCst);
            *status_msg_clone.lock().unwrap() = Some("Transfer Complete!".to_string());
        });
    }

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

    pub fn new() -> Self {
        // Start in user home directory (or current directory fallback)
        let initial_dir = dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));
        
        let mut state = Self {
            active_screen: ActiveScreen::Browser,
            current_dir: initial_dir.clone(),
            items: Vec::new(),
            selected_idx: None,
            history: vec![initial_dir],
            history_idx: 0,
            search_query: String::new(),
            theme: ThemeMode::Dark,
            accent_color: "yellow".to_string(),
            highlight_color: "gray".to_string(),
            glassmorphism_glow1_color: "purple".to_string(),
            glassmorphism_glow2_color: "blue".to_string(),
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
            alt_pressed: false,
            super_pressed: false,
            dragging_item: None,
            drag_pressed_item: None,
            drag_start_pos: None,
            drag_item_offset: None,
            drag_select_start: None,
            drag_select_current: None,
            item_rects: Vec::new(),
            deferred_click_idx: None,
            detected_drives: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            is_detecting_drives: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            last_drive_detect_time: std::time::Instant::now(),
            dashboard_selection: DashboardSelection::None,
            zendrop_open: false,
            zendrop_devices: Vec::new(),
            last_zendrop_scan: None,
            zendrop_btn_pos: None,
            zendrop_network_name: "Local Network".to_string(),
            zendrop_networks: Vec::new(),
            wifi_connect_ssid: None,
            wifi_connect_password: String::new(),
            wifi_connect_error: None,
            wifi_connecting: false,
            zendrop_send_target: None,
            zendrop_send_progress: 0.0,
            zendrop_send_status: String::new(),
            wifi_connection_result: std::sync::Arc::new(std::sync::Mutex::new(None)),
            zendrop_progress: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            zendrop_error: None,
            zendrop_show_devices: false,
            zendrop_status_msg: std::sync::Arc::new(std::sync::Mutex::new(None)),
            zendrop_send_mode: ZendropSendMode::Files,
        };

        // Spawn background TCP listener on port 8888 to receive incoming ZenDrop/file_transfer_app transfers
        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default()))
            .join("ZenDrop");

        std::thread::spawn(move || {
            use std::net::TcpListener;
            use std::io::{Read, Write};

            let listener = match TcpListener::bind("0.0.0.0:8888") {
                Ok(l) => l,
                Err(_) => return,
            };

            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let _ = std::fs::create_dir_all(&download_dir);

                // Read filename length: 2 bytes (u16 big-endian)
                let mut name_len_buf = [0; 2];
                if stream.read_exact(&mut name_len_buf).is_err() { continue; }
                let name_len = u16::from_be_bytes(name_len_buf) as usize;

                // Read filename string
                let mut name_buf = vec![0; name_len];
                if stream.read_exact(&mut name_buf).is_err() { continue; }
                let rel_name = String::from_utf8_lossy(&name_buf).into_owned();

                // Read content size: 8 bytes (u64 big-endian)
                let mut content_len_buf = [0; 8];
                if stream.read_exact(&mut content_len_buf).is_err() { continue; }
                let content_len = u64::from_be_bytes(content_len_buf);

                // Read compression flag: 1 byte boolean
                let mut compressed_buf = [0; 1];
                if stream.read_exact(&mut compressed_buf).is_err() { continue; }
                let _compressed = compressed_buf[0] != 0;

                let target_path = download_dir.join(rel_name);
                if let Some(parent) = target_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                let mut file = match std::fs::File::create(&target_path) {
                    Ok(f) => f,
                    Err(_) => continue,
                };

                let mut buffer = vec![0; 64 * 1024];
                let mut read_bytes = 0_u64;
                while read_bytes < content_len {
                    let to_read = ((content_len - read_bytes) as usize).min(buffer.len());
                    let n = match stream.read(&mut buffer[..to_read]) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(_) => break,
                    };
                    if file.write_all(&buffer[..n]).is_err() { break; }
                    read_bytes += n as u64;
                }
            }
        });

        state.trigger_drive_refresh();
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

#[derive(Debug, Clone, Copy)]
pub struct DiskSpaceInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
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

fn get_default_gateway() -> Option<String> {
    let content = std::fs::read_to_string("/proc/net/route").ok()?;
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let dest = parts[1];
            let gw_hex = parts[2];
            if dest == "00000000" && gw_hex != "00000000" {
                if let Ok(val) = u32::from_str_radix(gw_hex, 16) {
                    let bytes = val.to_ne_bytes();
                    return Some(format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3]));
                }
            }
        }
    }
    None
}

/// Scan /proc/net/arp for recently seen LAN devices.
pub fn scan_zendrop_devices() -> Vec<crate::state::ZendropDevice> {
    let mut devices = Vec::new();

    // Auto-inject default gateway (e.g. mobile hotspot host)
    if let Some(gw_ip) = get_default_gateway() {
        devices.push(crate::state::ZendropDevice {
            ip: gw_ip,
            mac: "Hotspot".to_string(),
            hostname: "Mobile App Gateway".to_string(),
            iface: "wlp2s0".to_string(),
        });
    }

    if let Ok(content) = std::fs::read_to_string("/proc/net/arp") {
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Format: IP, HW type, Flags, HW address, Mask, Device
            if parts.len() < 6 { continue; }
            let ip    = parts[0].to_string();
            let flags = parts[2];
            let mac   = parts[3].to_string();
            let iface = parts[5].to_string();

            // Flags 0x0 = incomplete (no reply), skip those
            if flags == "0x0" || mac == "00:00:00:00:00:00" { continue; }

            // Avoid duplication of gateway IP
            if devices.iter().any(|d| d.ip == ip) { continue; }

            // Try a simple hostname lookup via /etc/hosts first, then fall back to ip
            let hostname = lookup_hostname(&ip);

            devices.push(crate::state::ZendropDevice { ip, mac, hostname, iface });
        }
    }

    devices
}

fn lookup_hostname(ip: &str) -> String {
    // Check /etc/hosts
    if let Ok(hosts) = std::fs::read_to_string("/etc/hosts") {
        for line in hosts.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.first() == Some(&ip) {
                if let Some(name) = parts.get(1) {
                    return name.to_string();
                }
            }
        }
    }
    // Fall back to IP itself
    ip.to_string()
}

pub fn query_network_name() -> String {
    // Try iwgetid -r
    if let Ok(output) = std::process::Command::new("iwgetid")
        .arg("-r")
        .output()
    {
        if output.status.success() {
            let ssid = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !ssid.is_empty() {
                return ssid;
            }
        }
    }

    // Try nmcli as fallback
    if let Ok(output) = std::process::Command::new("nmcli")
        .args(&["-t", "-f", "ACTIVE,SSID", "dev", "wifi"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("yes:") {
                    let ssid = line.trim_start_matches("yes:").trim().to_string();
                    if !ssid.is_empty() {
                        return ssid;
                    }
                }
            }
        }
    }

    "Wired / Local Network".to_string()
}

pub fn scan_wifi_networks() -> Vec<crate::state::WifiNetwork> {
    let output = match std::process::Command::new("nmcli")
        .args(&["-t", "-f", "SSID,SIGNAL,SECURITY,ACTIVE", "dev", "wifi"])
        .output()
    {
        Ok(out) => out,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks_map = std::collections::HashMap::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 4 {
            continue;
        }
        let ssid = parts[0].trim().to_string();
        if ssid.is_empty() {
            continue;
        }
        let signal = parts[1].trim().parse::<u8>().unwrap_or(0);
        let security = parts[2].trim().to_string();
        let active = parts[3].trim().eq_ignore_ascii_case("yes");

        let entry = networks_map.entry(ssid.clone()).or_insert(crate::state::WifiNetwork {
            ssid: ssid.clone(),
            signal,
            security: security.clone(),
            active,
        });

        if active {
            entry.active = true;
            entry.signal = signal;
        } else if !entry.active && signal > entry.signal {
            *entry = crate::state::WifiNetwork {
                ssid,
                signal,
                security,
                active,
            };
        }
    }

    let mut list: Vec<_> = networks_map.into_values().collect();
    list.sort_by(|a, b| {
        if a.active != b.active {
            b.active.cmp(&a.active)
        } else {
            b.signal.cmp(&a.signal)
        }
    });

    list
}

fn collect_files_recursive(path: std::path::PathBuf, base_path: &std::path::Path, files: &mut Vec<(String, std::path::PathBuf)>) -> std::io::Result<()> {
    if path.is_file() {
        let rel_path = path.strip_prefix(base_path)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        files.push((rel_path, path));
    } else if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            collect_files_recursive(entry.path(), base_path, files)?;
        }
    }
    Ok(())
}

fn read_from_clipboard() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(&["-NoProfile", "-Command", "Get-Clipboard"])
            .output()
            .ok()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("pbpaste").output().ok()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("wl-paste").output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
        if let Ok(output) = std::process::Command::new("xclip")
            .args(&["-o", "-selection", "clipboard"])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
        if let Ok(output) = std::process::Command::new("xsel")
            .args(&["-o", "-b"])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
    }
    None
}



