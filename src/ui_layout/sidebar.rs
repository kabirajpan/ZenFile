use crate::state::FileManagerState;
use crate::theme::ThemeColors;
use super::common::{
    NF_FA_HOME, NF_FA_DESKTOP, NF_FA_DOWNLOAD, NF_FA_FILE_ALT, NF_FA_HDD,
    NF_FA_MUSIC, NF_FA_PICTURE, NF_FA_FILM,
};
use zenthra::{Color, Ui, Align, FontWeight};
use std::path::PathBuf;

pub fn draw_sidebar(ui: &mut Ui, state: &mut FileManagerState) {
    let colors = ThemeColors::get(state.theme);

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
                    let is_active = state.current_dir == target_path;
                    let resp = ui.container()
                        .row()
                        .gap(10.0)
                        .valign(Align::Center)
                        .fill_x()
                        .padding(6.0, 10.0, 6.0, 10.0)
                        .bg(if is_active { colors.bg_active } else { Color::TRANSPARENT })
                        .hover_bg(if is_active { colors.accent.with_alpha(0.25) } else { colors.border })
                        .radius_all(6.0)
                        .show(|ui| {
                            ui.text(icon)
                                .size(12.0)
                                .color(if is_active { colors.accent } else { colors.text_muted })
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
                    if resp.hovered && !ui.mouse_down {
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
