use crate::state::FileManagerState;
use super::common::{
    NF_FA_HOME, NF_FA_DESKTOP, NF_FA_DOWNLOAD, NF_FA_FILE_ALT, NF_FA_HDD,
    NF_FA_MUSIC, NF_FA_PICTURE, NF_FA_FILM,
    is_drag_drop_hovered, drop_target_bg,
};
use zenthra::{Color, Ui, Align, FontWeight};
use std::path::PathBuf;

pub fn draw_sidebar(ui: &mut Ui, state: &mut FileManagerState) {
    let colors = state.colors();

    ui.container()
        .width(state.sidebar_width)
        .fill_y()
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .padding(15.0, 15.0, 15.0, 15.0)
        .column()
        .show(|ui| {
            // Sidebar title
            ui.text("SHORTCUTS")
                .size(9.5)
                .weight(FontWeight::Bold)
                .color(colors.text_muted)
                .show();

            ui.spacing(10.0);

            let mut draw_shortcut = |ui: &mut Ui, icon: &str, label: &str, path: Option<PathBuf>| {
                if let Some(target_path) = path {
                    use std::hash::{Hash, Hasher};
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    target_path.hash(&mut hasher);
                    let shortcut_id = zenthra::Id::from_u64(hasher.finish());

                    let mut is_drop_hovered = false;
                    if state.dragging_item.is_some() {
                        is_drop_hovered = is_drag_drop_hovered(ui, shortcut_id);
                    }

                    let is_active = state.current_dir == target_path;
                    let bg_color = if is_drop_hovered {
                        drop_target_bg(&colors, true)
                    } else if is_active {
                        colors.bg_active
                    } else {
                        Color::TRANSPARENT
                    };

                    let resp = ui.container()
                        .id(shortcut_id)
                        .row()
                        .gap(10.0)
                        .valign(Align::Center)
                        .fill_x()
                        .padding(6.0, 10.0, 6.0, 10.0)
                        .bg(bg_color)
                        .hover_bg(if is_active { colors.bg_active } else { colors.highlight })
                        .radius_all(6.0)
                        .show(|ui| {
                            ui.text(icon)
                                .size(12.0)
                                .color(if is_active { colors.text_primary } else { colors.text_muted })
                                .show();
                            ui.text(label)
                                .size(11.5)
                                .color(if is_active { colors.text_primary } else { colors.text_muted })
                                .show();
                        });

                    if resp.clicked {
                        state.change_dir(target_path.clone());
                        ui.request_redraw();
                    }
                    if (resp.hovered || is_drop_hovered) && !ui.mouse_down {
                        if let Some(src_path) = state.dragging_item.clone() {
                            if state.selected_paths.contains(&src_path) {
                                let paths: Vec<_> = state.selected_paths.iter().cloned().collect();
                                for p in paths {
                                    state.move_item(&p, &target_path);
                                }
                                state.selected_paths.clear();
                            } else {
                                state.move_item(&src_path, &target_path);
                            }
                            state.dragging_item = None;
                            ui.request_redraw();
                        }
                    }
                    ui.spacing(2.0);
                }
            };

            draw_shortcut(ui, NF_FA_HOME, "Home", dirs::home_dir());
            draw_shortcut(ui, NF_FA_DESKTOP, "Desktop", dirs::desktop_dir());
            draw_shortcut(ui, NF_FA_DOWNLOAD, "Downloads", dirs::download_dir());
            draw_shortcut(ui, NF_FA_FILE_ALT, "Documents", dirs::document_dir());
            draw_shortcut(ui, NF_FA_MUSIC, "Music", dirs::audio_dir().or_else(|| dirs::home_dir().map(|h| h.join("Music"))));
            draw_shortcut(ui, NF_FA_PICTURE, "Pictures", dirs::picture_dir().or_else(|| dirs::home_dir().map(|h| h.join("Pictures"))));
            draw_shortcut(ui, NF_FA_FILM, "Videos", dirs::video_dir().or_else(|| dirs::home_dir().map(|h| h.join("Videos"))));
            draw_shortcut(ui, NF_FA_HDD, "Root Directory", Some(PathBuf::from("/")));

            ui.spacing(15.0);

            ui.text("DEVICES")
                .size(9.5)
                .weight(FontWeight::Bold)
                .color(colors.text_muted)
                .show();

            ui.spacing(10.0);
            
            draw_shortcut(ui, NF_FA_HDD, "System Disk", Some(PathBuf::from("/")));
        });
}
