use crate::state::FileManagerState;
use super::common::{
    NF_FA_HOME, NF_FA_DESKTOP, NF_FA_DOWNLOAD, NF_FA_FILE_ALT, NF_FA_HDD,
    NF_FA_MUSIC, NF_FA_PICTURE, NF_FA_FILM, NF_FA_TACHOMETER,
    is_drag_drop_hovered, drop_target_bg,
};
use zenthra::{Color, Ui, Align, FontWeight};
use std::path::PathBuf;

pub fn draw_sidebar(ui: &mut Ui, state: &mut FileManagerState) {
    let colors = state.colors();
    let drives = state.detect_drives();

    let active_menu_key = zenthra::Id::from_u64(999999900);
    let active_menu_id = ui.interaction_state.get(&active_menu_key).copied().map(|v| v as u64).unwrap_or(0);
    let is_menu_open = state.context_menu_pos.is_some() || active_menu_id != 0;
    let show_hover = !is_menu_open;

    let mut sidebar_container = ui.container()
        .width(state.sidebar_width)
        .fill_y()
        .padding(15.0, 15.0, 15.0, 15.0)
        .column();

    if state.theme == crate::theme::ThemeMode::Glassmorphism {
        sidebar_container = sidebar_container
            .bg(colors.bg_sidebar.with_alpha(0.45))
            .border(Color::rgba(255.0/255.0, 255.0/255.0, 255.0/255.0, 0.04), 1.0)
            .backdrop_filter(zenthra::BackdropFilter::new().blur(15.0, zenthra::style::blur::Type::Glassmorphism));
    } else {
        sidebar_container = sidebar_container
            .bg(colors.bg_sidebar)
            .border(colors.border, 1.0);
    }

    sidebar_container.show(|ui| {
            // Sidebar title
            ui.text("SHORTCUTS")
                .size(9.5)
                .weight(FontWeight::Bold)
                .color(colors.text_muted)
                .show();

            ui.spacing(10.0);

            // Dashboard / This PC Link
            let is_dashboard_active = state.active_screen == crate::state::ActiveScreen::Dashboard;
            let dashboard_id = zenthra::Id::from_u64(888888890);
            let mut dashboard_container = ui.container()
                .id(dashboard_id)
                .row()
                .gap(10.0)
                .valign(Align::Center)
                .fill_x()
                .padding(6.0, 10.0, 6.0, 10.0)
                .radius_all(6.0);

            if is_dashboard_active && state.theme == crate::theme::ThemeMode::Glassmorphism {
                dashboard_container = dashboard_container
                    .bg(colors.bg_active.with_alpha(0.18))
                    .border(Color::rgba(255.0/255.0, 255.0/255.0, 255.0/255.0, 0.04), 1.0)
                    .backdrop_filter(zenthra::BackdropFilter::new().blur(12.0, zenthra::style::blur::Type::Glassmorphism));
            } else {
                let bg_color = if is_dashboard_active { colors.bg_active } else { Color::TRANSPARENT };
                dashboard_container = dashboard_container
                    .bg(bg_color)
                    .hover_bg(if is_dashboard_active { colors.bg_active } else { colors.highlight });
            }

            let resp = dashboard_container.show(|ui| {
                ui.text("\u{f148a}")
                    .size(12.0)
                    .color(if is_dashboard_active { colors.text_primary } else { colors.text_muted })
                    .show();
                ui.text("Dashboard")
                    .size(11.5)
                    .color(if is_dashboard_active { colors.text_primary } else { colors.text_muted })
                    .show();
            });

            if resp.clicked {
                state.active_screen = crate::state::ActiveScreen::Dashboard;
                ui.request_redraw();
            }

            ui.spacing(2.0);

            {
                let active_screen = state.active_screen;
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

                        let is_active = state.current_dir == target_path && active_screen == crate::state::ActiveScreen::Browser;
                        let bg_color = if is_drop_hovered {
                            drop_target_bg(&colors, true)
                        } else if is_active {
                            colors.bg_active
                        } else {
                            Color::TRANSPARENT
                        };

                        let mut shortcut_container = ui.container()
                            .id(shortcut_id)
                            .row()
                            .gap(10.0)
                            .valign(Align::Center)
                            .fill_x()
                            .padding(6.0, 10.0, 6.0, 10.0)
                            .radius_all(6.0);

                        if is_active && state.theme == crate::theme::ThemeMode::Glassmorphism {
                            shortcut_container = shortcut_container
                                .bg(colors.bg_active.with_alpha(0.18))
                                .border(Color::rgba(255.0/255.0, 255.0/255.0, 255.0/255.0, 0.04), 1.0)
                                .backdrop_filter(zenthra::BackdropFilter::new().blur(12.0, zenthra::style::blur::Type::Glassmorphism));
                        } else {
                            shortcut_container = shortcut_container
                                .bg(bg_color)
                                .hover_bg(if is_active { if show_hover { colors.bg_active } else { Color::TRANSPARENT } } else if show_hover { colors.highlight } else { Color::TRANSPARENT });
                        }

                        let resp = shortcut_container.show(|ui| {
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
                            state.active_screen = crate::state::ActiveScreen::Browser;
                            if target_path.to_string_lossy().starts_with("/dev/") {
                                state.mount_and_change_dir_by_dev(&target_path);
                            } else {
                                state.change_dir(target_path.clone());
                            }
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
            }

            ui.spacing(15.0);

            ui.text("DEVICES")
                .size(9.5)
                .weight(FontWeight::Bold)
                .color(colors.text_muted)
                .show();

            ui.spacing(10.0);
            
            for (idx, drive) in drives.into_iter().enumerate() {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                drive.name.hash(&mut hasher);
                idx.hash(&mut hasher);
                let drive_row_id = zenthra::Id::from_u64(hasher.finish());
                
                let is_active = state.current_dir == drive.path;
                let bg_color = if is_active { colors.bg_active } else { Color::TRANSPARENT };
                
                let mut drive_row = ui.container()
                    .id(drive_row_id)
                    .row()
                    .halign(Align::SpaceBetween)
                    .valign(Align::Center)
                    .fill_x()
                    .padding(4.0, 8.0, 4.0, 8.0)
                    .radius_all(6.0);
                    
                if is_active && state.theme == crate::theme::ThemeMode::Glassmorphism {
                    drive_row = drive_row
                        .bg(colors.bg_active.with_alpha(0.18))
                        .border(Color::rgba(255.0/255.0, 255.0/255.0, 255.0/255.0, 0.04), 1.0)
                        .backdrop_filter(zenthra::BackdropFilter::new().blur(12.0, zenthra::style::blur::Type::Glassmorphism));
                } else {
                    drive_row = drive_row
                        .bg(bg_color)
                        .hover_bg(if is_active { if show_hover { colors.bg_active } else { Color::TRANSPARENT } } else if show_hover { colors.highlight } else { Color::TRANSPARENT });
                }
                
                drive_row.show(|ui| {
                    let left_id = zenthra::Id::from_u64(hasher.finish() ^ 9999901);
                    let left_section = ui.container()
                        .id(left_id)
                        .row()
                        .gap(10.0)
                        .valign(Align::Center);
                        
                    let resp = left_section.show(|ui| {
                        ui.text(NF_FA_HDD)
                            .size(12.0)
                            .color(if is_active { colors.text_primary } else { colors.text_muted })
                            .show();
                        ui.text(&drive.name)
                            .size(11.5)
                            .max_width(110.0)
                            .color(if is_active { colors.text_primary } else { colors.text_muted })
                            .show();
                    });
                    
                    if resp.clicked {
                        if !drive.is_mounted {
                            state.mount_and_change_dir_by_dev(&drive.path);
                        } else {
                            state.change_dir(drive.path.clone());
                        }
                        ui.request_redraw();
                    }
                    
                    let right_section = ui.container()
                        .row()
                        .gap(6.0)
                        .valign(Align::Center);
                        
                    right_section.show(|ui| {
                        use crate::ui_layout::common::{NF_FA_EJECT, NF_FA_MOUNT};
                        
                        if !drive.device_node.is_empty() {
                            if drive.is_mounted {
                                let eject_id = zenthra::Id::from_u64(hasher.finish() ^ 9999902);
                                let eject_btn = ui.container()
                                    .id(eject_id)
                                    .padding(5.0, 7.0, 5.0, 7.0)
                                    .radius_all(4.0)
                                    .min_width(26.0)
                                    .border(colors.border, 1.0)
                                    .hover_bg(colors.highlight);
                                let resp_eject = eject_btn.show(|ui| {
                                    ui.text(NF_FA_EJECT)
                                        .size(13.0)
                                        .ellipsis(false)
                                        .color(colors.text_muted)
                                        .show();
                                });
                                if resp_eject.clicked {
                                    state.unmount_dev(&drive.device_node);
                                    ui.request_redraw();
                                }
                            } else {
                                let mount_id = zenthra::Id::from_u64(hasher.finish() ^ 9999903);
                                let mount_btn = ui.container()
                                    .id(mount_id)
                                    .padding(5.0, 7.0, 5.0, 7.0)
                                    .radius_all(4.0)
                                    .min_width(26.0)
                                    .border(colors.border, 1.0)
                                    .hover_bg(colors.highlight);
                                let resp_mount = mount_btn.show(|ui| {
                                    ui.text(NF_FA_MOUNT)
                                        .size(13.0)
                                        .ellipsis(false)
                                        .color(colors.accent)
                                        .show();
                                });
                                if resp_mount.clicked {
                                    state.mount_and_change_dir_by_dev(&drive.path);
                                    ui.request_redraw();
                                }
                            }
                        }
                    });
                });
                
                ui.spacing(2.0);
            }
        });
        ui.request_redraw_after(std::time::Duration::from_secs(1));
}
