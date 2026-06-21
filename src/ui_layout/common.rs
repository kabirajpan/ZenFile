use zenthra::{Color, Id, Ui};
use std::path::{Path, PathBuf};
use std::hash::{Hash, Hasher};

/// Matches the ID resolution used by `container().id(...)`.
pub fn resolve_widget_id(ui: &Ui, id: Id) -> Id {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    id.hash(&mut hasher);
    if let Some(parent) = ui.semantic_stack.last() {
        parent.hash(&mut hasher);
    }
    Id::from_u64(hasher.finish())
}

/// True when the cursor is over a widget's previous-frame screen rect.
pub fn is_drag_drop_hovered(ui: &Ui, base_id: Id) -> bool {
    let resolved = resolve_widget_id(ui, base_id);
    if let Some(rect) = ui.screen_layout_cache.get(&resolved) {
        ui.mouse_x >= rect.origin.x
            && ui.mouse_x <= rect.origin.x + rect.size.width
            && ui.mouse_y >= rect.origin.y
            && ui.mouse_y <= rect.origin.y + rect.size.height
    } else {
        false
    }
}

pub fn drop_target_bg(colors: &crate::theme::ThemeColors, is_drop_hovered: bool) -> Color {
    if is_drop_hovered {
        colors.highlight
    } else {
        Color::TRANSPARENT
    }
}

pub fn tag_name_to_color(name: &str) -> Color {
    match name {
        "red"    => Color::rgb(252.0 / 255.0, 60.0 / 255.0, 56.0 / 255.0),
        "orange" => Color::rgb(253.0 / 255.0, 151.0 / 255.0, 31.0 / 255.0),
        "yellow" => Color::rgb(244.0 / 255.0, 191.0 / 255.0, 58.0 / 255.0),
        "green"  => Color::rgb(46.0 / 255.0, 204.0 / 255.0, 113.0 / 255.0),
        "blue"   => Color::rgb(52.0 / 255.0, 152.0 / 255.0, 219.0 / 255.0),
        "purple" => Color::rgb(155.0 / 255.0, 89.0 / 255.0, 182.0 / 255.0),
        _ => Color::TRANSPARENT,
    }
}


// Nerd Font Icon constants
pub const NF_FA_FOLDER: &str = "\u{f07b}";
pub const NF_FA_DESKTOP: &str = "\u{f108}";
pub const NF_FA_HOME: &str = "\u{f015}";
pub const NF_FA_DOWNLOAD: &str = "\u{f019}";
pub const NF_FA_FILE_ALT: &str = "\u{f15c}";
pub const NF_FA_HDD: &str = "\u{f0a0}";
pub const NF_FA_MUSIC: &str = "\u{f001}";
pub const NF_FA_PICTURE: &str = "\u{f03e}";
pub const NF_FA_FILM: &str = "\u{f008}";

pub const NF_FA_ARROW_LEFT: &str = "\u{f060}";
pub const NF_FA_ARROW_RIGHT: &str = "\u{f061}";
pub const NF_FA_ARROW_UP: &str = "\u{f062}";
pub const NF_FA_REFRESH: &str = "\u{f021}";
pub const NF_FA_SEARCH: &str = "\u{f002}";
pub const NF_FA_TRASH: &str = "\u{f1f8}";
pub const NF_FA_EDIT: &str = "\u{f040}";
pub const NF_FA_EXTERNAL_LINK: &str = "\u{f08e}";

/// Helper to get image icon path for a file category and extension using the active theme
pub fn get_item_icon_path(theme_name: &str, category: &str, extension: &str) -> PathBuf {
    let theme = crate::theme::IconTheme::new(theme_name);
    theme.get_icon_path(category, extension)
}

pub fn get_folder_icon_path(theme_name: &str, _folder_name: &str, folder_color: &str, flat_folders: bool) -> PathBuf {
    let prefix = if Path::new("apps/file_manager").exists() {
        "apps/file_manager/"
    } else {
        ""
    };
    let folders_base = PathBuf::from(format!("{}assets/themes/{}/folders", prefix, theme_name));
    let color = folder_color.to_lowercase();
    if flat_folders {
        let folder_dir = if color == "gray" {
            "gray-folder-layers".to_string()
        } else if color == "turquoise" {
            "torquoise-folder-layer".to_string()
        } else {
            format!("{}-folder-layer", color)
        };
        folders_base.join(folder_dir).join("Folder.png")
    } else {
        let file_name = if color == "turquoise" {
            "turquoise.png".to_string()
        } else {
            format!("{}.png", color)
        };
        folders_base.join(file_name)
    }
}

/// Helper to format date from a FileItem
pub fn format_date(item: &crate::state::FileItem) -> String {
    item.modified.map(|time| {
        if let Ok(duration) = time.duration_since(std::time::UNIX_EPOCH) {
            let secs = duration.as_secs();
            jiff::Timestamp::from_second(secs as i64)
                .map(|t| t.to_string())
                .unwrap_or_else(|_| "Unknown".to_string())
                .chars().take(10).collect::<String>()
        } else {
            "Unknown".to_string()
        }
    }).unwrap_or_else(|| "Unknown".to_string())
}

/// Helper function to open file in system default
pub fn open_file(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(&["/C", "start", "", &path.to_string_lossy()])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(path)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn();
    }
}

/// Helper to truncate files gracefully, maintaining the extension
pub fn truncate_filename(name: &str, max_len: usize) -> String {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() <= max_len {
        return name.to_string();
    }
    
    let ext_pos = name.rfind('.');
    if let Some(pos) = ext_pos {
        let ext = &name[pos..];
        let ext_chars: Vec<char> = ext.chars().collect();
        if ext_chars.len() < max_len / 2 {
            let left_len = max_len - ext_chars.len() - 3;
            let left: String = chars.iter().take(left_len).collect();
            return format!("{}...{}", left, ext);
        }
    }
    
    let half = max_len / 2 - 2;
    let left: String = chars.iter().take(half).collect();
    let right: String = chars.iter().skip(chars.len() - half).collect();
    format!("{}...{}", left, right)
}

pub fn truncate_str(s: &str, max_len: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_len {
        s.to_string()
    } else {
        let truncated: String = chars.iter().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

pub fn start_rename(ui: &mut Ui, state: &mut crate::state::FileManagerState, path: PathBuf, name: String) {
    state.renaming_item = Some(path.clone());
    state.rename_buffer = name;
    
    // Set focus to the input box deterministically
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::Hasher;
    path.hash(&mut hasher);
    let rename_input_id = Id::from_u64(hasher.finish());
    ui.focused_id = Some(rename_input_id);
}
