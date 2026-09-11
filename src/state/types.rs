use std::path::PathBuf;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    Sending,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TransferEntry {
    pub filename: String,
    pub total_bytes: u64,
    pub sent_bytes: std::sync::Arc<std::sync::atomic::AtomicU64>,
    pub status: std::sync::Arc<std::sync::Mutex<TransferStatus>>,
    pub speed_mbps: std::sync::Arc<std::sync::Mutex<f32>>,
    #[allow(dead_code)]
    pub started_at: std::time::Instant,
    pub cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DashboardSelection {
    None,
    Drive(std::path::PathBuf),
    Cpu,
    Ram,
    Directory(std::path::PathBuf),
}

#[derive(Debug, Clone)]
pub struct ZendropDevice {
    pub ip: String,
    #[allow(dead_code)]
    pub mac: String,
    pub hostname: String,
    #[allow(dead_code)]
    pub iface: String,
}

#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub active: bool,
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
    pub zendrop_transfers: Vec<TransferEntry>,
    pub zendrop_notif_open: bool,
    pub zendrop_notif_btn_pos: Option<(f32, f32)>,
    pub pending_refresh: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub wifi_btn_rect: Option<(f32, f32, f32, f32)>,
    pub bell_btn_rect: Option<(f32, f32, f32, f32)>,
    pub wifi_panel_rect: Option<(f32, f32, f32, f32)>,
    pub bell_panel_rect: Option<(f32, f32, f32, f32)>,
    
    // Real Pairing States
    pub zendrop_paired: Vec<(String, String, String)>,
    pub zendrop_pair_request: std::sync::Arc<std::sync::Mutex<Option<(String, String)>>>,
    pub zendrop_pair_result: std::sync::Arc<std::sync::atomic::AtomicI32>,
    pub zendrop_pairing_active: bool,
    #[allow(dead_code)]
    pub zendrop_pair_ip_input: String,
    pub zendrop_scanned: std::sync::Arc<std::sync::Mutex<Vec<(String, String, std::time::Instant)>>>,
    pub zendrop_scan_results: std::sync::Arc<std::sync::Mutex<Option<(Vec<ZendropDevice>, Vec<WifiNetwork>, String)>>>,
    pub zendrop_scanning: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub zendrop_device_name: std::sync::Arc<std::sync::Mutex<String>>,
}

impl FileManagerState {
    pub fn colors(&self) -> ThemeColors {
        ThemeColors::resolve(self.theme, &self.accent_color, &self.highlight_color)
    }

    pub fn new() -> Self {
        // Start in user home directory (or current directory fallback)
        let initial_dir = dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));
        
        let default_name = std::env::var("USER").unwrap_or_else(|_| "Desktop".to_string()) + "-ZenFile";
        let device_name_arc = std::sync::Arc::new(std::sync::Mutex::new(default_name));

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
            zendrop_transfers: Vec::new(),
            zendrop_notif_open: false,
            zendrop_notif_btn_pos: None,
            pending_refresh: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            wifi_btn_rect: None,
            bell_btn_rect: None,
            wifi_panel_rect: None,
            bell_panel_rect: None,
            zendrop_paired: Vec::new(),
            zendrop_pair_request: std::sync::Arc::new(std::sync::Mutex::new(None)),
            zendrop_pair_result: std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0)),
            zendrop_pairing_active: false,
            zendrop_pair_ip_input: String::new(),
            zendrop_scanned: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            zendrop_scan_results: std::sync::Arc::new(std::sync::Mutex::new(None)),
            zendrop_scanning: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            zendrop_device_name: device_name_arc.clone(),
        };
        state.load_device_name();
        state.load_paired_devices();
        state.load_tags();
        state.init_background_services();
        state
    }
}
