use crate::state::{FileManagerState, ViewMode, SortBy, SortOrder};
use crate::theme::{ThemeMode, ACCENT_COLOR_OPTIONS, HIGHLIGHT_COLOR_OPTIONS};
use super::common::NF_FA_FOLDER;
use zenthra::{Color, Ui, FontWeight, WindowAction, Align, Id};

pub fn draw_title_bar(ui: &mut Ui, state: &mut FileManagerState) {
    let colors = state.colors();

    // Reset hover flag for menus in title bar since we do not use menu_bar wrapper
    let hover_flag_key = Id::from_u64(999999902);
    ui.interaction_state.insert(hover_flag_key, 0.0);

    const BTN_W: f32 = 20.0;
    const BAR_H: f32 = 36.0;
    const RIGHT_W: f32 = BTN_W * 3.0 + 36.0; // 96.0

    ui.container()
        .full_width()
        .height(BAR_H)
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .row()
        .valign(Align::Center)
        .no_wrap()
        .show(|ui| {
            let left_start = ui.cursor_x;

            // LEFT: compact icon  +  inline menu items
            ui.container()
                .row()
                .gap(12.0)
                .valign(Align::Center)
                .padding_left(12.0)
                .show(|ui| {
                    // App Icon & Name
                    ui.container()
                        .row()
                        .gap(8.0)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text(NF_FA_FOLDER)
                                .size(14.0)
                                .color(colors.accent)
                                .show();
                            ui.text("ZenFile")
                                .size(12.0)
                                .weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });

                    // Menus directly (no full-width menu_bar wrapper)
                    ui.menu("File").show(|ui| {
                        if ui.menu_item("New File").shortcut("Ctrl+N").show().clicked {
                            let mut path = state.current_dir.join("New File.txt");
                            if path.exists() {
                                let mut count = 2;
                                while state.current_dir.join(format!("New File ({}).txt", count)).exists() {
                                    count += 1;
                                }
                                path = state.current_dir.join(format!("New File ({}).txt", count));
                            }
                            let _ = std::fs::write(&path, "");
                            state.scan_current_dir();
                            ui.request_redraw();
                        }
                        if ui.menu_item("New Folder").shortcut("Ctrl+Shift+N").show().clicked {
                            let mut path = state.current_dir.join("New Folder");
                            if path.exists() {
                                let mut count = 2;
                                while state.current_dir.join(format!("New Folder ({})", count)).exists() {
                                    count += 1;
                                }
                                path = state.current_dir.join(format!("New Folder ({})", count));
                            }
                            let _ = std::fs::create_dir(&path);
                            state.scan_current_dir();
                            ui.request_redraw();
                        }
                        let has_selection = state.selected_idx.is_some();
                        if ui.menu_item("Rename").shortcut("F2").show().clicked && has_selection {
                            if let Some(idx) = state.selected_idx {
                                if idx < state.items.len() {
                                    let item = &state.items[idx];
                                    super::common::start_rename(ui, state, item.path.clone(), item.name.clone());
                                    ui.request_redraw();
                                }
                            }
                        }
                        if ui.menu_item("Delete").shortcut("Del").show().clicked && has_selection {
                            if let Some(idx) = state.selected_idx {
                                if idx < state.items.len() {
                                    let item = &state.items[idx];
                                    state.delete_confirm = Some(item.path.clone());
                                    ui.request_redraw();
                                }
                            }
                        }
                        if ui.menu_item("Close Window").shortcut("Ctrl+W").show().clicked {
                            ui.window_actions.push(WindowAction::Close);
                        }
                    });
                    ui.menu("Edit").show(|ui| {
                        let has_selection = !state.selected_paths.is_empty();
                        if ui.menu_item("Cut").shortcut("Ctrl+X").show().clicked && has_selection {
                            state.cut_selected();
                        }
                        if ui.menu_item("Copy").shortcut("Ctrl+C").show().clicked && has_selection {
                            state.copy_selected();
                        }
                        let has_clip = state.clipboard.is_some();
                        if ui.menu_item("Paste").shortcut("Ctrl+V").show().clicked && has_clip {
                            state.paste_clipboard();
                            ui.request_redraw();
                        }
                    });
                    ui.menu("View").show(|ui| {
                        // ── View Mode ──
                        let list_mark = if state.view_mode == ViewMode::List { "● " } else { "  " };
                        if ui.menu_item(&format!("{}List", list_mark)).shortcut("Ctrl+1").show().clicked {
                            state.view_mode = ViewMode::List;
                            ui.request_redraw();
                        }
                        let med_mark = if state.view_mode == ViewMode::Medium { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Medium Icons", med_mark)).shortcut("Ctrl+2").show().clicked {
                            state.view_mode = ViewMode::Medium;
                            ui.request_redraw();
                        }
                        let lrg_mark = if state.view_mode == ViewMode::Large { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Large Icons", lrg_mark)).shortcut("Ctrl+3").show().clicked {
                            state.view_mode = ViewMode::Large;
                            ui.request_redraw();
                        }
                        let xl_mark = if state.view_mode == ViewMode::ExtraLarge { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Extra Large Icons", xl_mark)).shortcut("Ctrl+4").show().clicked {
                            state.view_mode = ViewMode::ExtraLarge;
                            ui.request_redraw();
                        }

                        // ── Sort By ──
                        ui.spacing(4.0);
                        let sn = if state.sort_by == SortBy::Name { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Sort by Name", sn)).show().clicked {
                            state.sort_by = SortBy::Name;
                            state.scan_current_dir();
                            ui.request_redraw();
                        }
                        let ss = if state.sort_by == SortBy::Size { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Sort by Size", ss)).show().clicked {
                            state.sort_by = SortBy::Size;
                            state.scan_current_dir();
                            ui.request_redraw();
                        }
                        let st = if state.sort_by == SortBy::Type { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Sort by Type", st)).show().clicked {
                            state.sort_by = SortBy::Type;
                            state.scan_current_dir();
                            ui.request_redraw();
                        }
                        let sd = if state.sort_by == SortBy::DateModified { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Sort by Date", sd)).show().clicked {
                            state.sort_by = SortBy::DateModified;
                            state.scan_current_dir();
                            ui.request_redraw();
                        }

                        // ── Sort Order ──
                        let asc = if state.sort_order == SortOrder::Ascending { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Ascending", asc)).show().clicked {
                            state.sort_order = SortOrder::Ascending;
                            state.scan_current_dir();
                            ui.request_redraw();
                        }
                        let desc = if state.sort_order == SortOrder::Descending { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Descending", desc)).show().clicked {
                            state.sort_order = SortOrder::Descending;
                            state.scan_current_dir();
                            ui.request_redraw();
                        }

                        // ── Panels & Misc ──
                        ui.spacing(4.0);
                        let sidebar_label = if state.sidebar_visible { "Hide Sidebar" } else { "Show Sidebar" };
                        if ui.menu_item(sidebar_label).shortcut("Ctrl+B").show().clicked {
                            state.sidebar_visible = !state.sidebar_visible;
                            ui.request_redraw();
                        }
                        let details_label = if state.details_visible { "Hide Details" } else { "Show Details" };
                        if ui.menu_item(details_label).shortcut("Ctrl+D").show().clicked {
                            state.details_visible = !state.details_visible;
                            ui.request_redraw();
                        }
                        if ui.menu_item("Refresh Files").shortcut("F5").show().clicked {
                            state.scan_current_dir();
                            ui.request_redraw();
                        }
                    });
                    ui.menu("Theme").show(|ui| {
                        let dark_mark = if state.theme == ThemeMode::Dark { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Dark Theme", dark_mark)).show().clicked {
                            state.theme = ThemeMode::Dark;
                            ui.request_redraw();
                        }
                        let light_mark = if state.theme == ThemeMode::Light { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Light Theme", light_mark)).show().clicked {
                            state.theme = ThemeMode::Light;
                            ui.request_redraw();
                        }

                        ui.spacing(4.0);

                        let flat_mark = if state.flat_folders { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Flat Folder Icons", flat_mark)).show().clicked {
                            state.flat_folders = !state.flat_folders;
                            ui.request_redraw();
                        }

                        ui.spacing(4.0);

                        ui.sub_menu("Accent Color").show(|ui| {
                            for (color_id, color_name) in ACCENT_COLOR_OPTIONS {
                                let is_active = state.accent_color == *color_id;
                                let label = if is_active {
                                    format!("● {}", color_name)
                                } else {
                                    format!("  {}", color_name)
                                };
                                if ui.menu_item(&label).show().clicked {
                                    state.accent_color = color_id.to_string();
                                    ui.request_redraw();
                                }
                            }
                        });

                        ui.sub_menu("Highlight Color").show(|ui| {
                            for (color_id, color_name) in HIGHLIGHT_COLOR_OPTIONS {
                                let is_active = state.highlight_color == *color_id;
                                let label = if is_active {
                                    format!("● {}", color_name)
                                } else {
                                    format!("  {}", color_name)
                                };
                                if ui.menu_item(&label).show().clicked {
                                    state.highlight_color = color_id.to_string();
                                    ui.request_redraw();
                                }
                            }
                        });

                        ui.sub_menu("Folder Color").show(|ui| {
                            let colors_list = [
                                ("gray", "Gray"),
                                ("yellow", "Yellow"),
                                ("blue", "Blue"),
                                ("green", "Green"),
                                ("red", "Red"),
                                ("orange", "Orange"),
                                ("purple", "Purple"),
                                ("pink", "Pink"),
                                ("turquoise", "Turquoise"),
                                ("violet", "Violet"),
                                ("lime", "Lime"),
                                ("white", "White"),
                            ];
                            for (color_id, color_name) in colors_list {
                                let is_active = state.folder_color == color_id;
                                let label = if is_active { format!("● {}", color_name) } else { format!("  {}", color_name) };
                                if ui.menu_item(&label).show().clicked {
                                    state.folder_color = color_id.to_string();
                                    ui.request_redraw();
                                }
                            }
                        });
                    });
                    ui.menu("Help").show(|ui| {
                        if ui.menu_item("About ZenFile").show().clicked {
                            state.show_about = true;
                            ui.request_redraw();
                        }
                    });
                });

            let left_w = ui.cursor_x - left_start;
            let drag_w = (ui.available_width - left_w - RIGHT_W).max(0.0);

            // CENTER: drag zone
            let drag_resp = ui.container()
                .width(drag_w)
                .height(BAR_H)
                .show(|_| {});

            if drag_resp.pressed {
                ui.window_actions.push(WindowAction::Drag);
            }

            // RIGHT: window controls
            ui.container()
                .width(RIGHT_W)
                .height(BAR_H)
                .row()
                .valign(Align::Center)
                .padding_right(12.0)
                .gap(12.0)
                .show(|ui| {
                    // Minimize
                    let min_key = Id::from_u64(999999903);
                    let min_hov = ui.interaction_state.get(&min_key).copied().unwrap_or(0.0) > 0.5;
                    let min_resp = ui.container()
                        .width(BTN_W).height(BTN_W)
                        .radius_all(BTN_W / 2.0)
                        .bg(if min_hov {
                            match state.theme {
                                ThemeMode::Dark => Color::rgba(1.0, 1.0, 1.0, 0.08),
                                ThemeMode::Light => Color::rgba(0.0, 0.0, 0.0, 0.06),
                            }
                        } else { Color::TRANSPARENT })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{f068}").size(8.0).color(colors.text_muted).show();
                        });
                    let h = min_resp.hovered;
                    let prev = ui.interaction_state.insert(min_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if min_resp.clicked {
                        ui.window_actions.push(WindowAction::Minimize);
                    }

                    // Maximize
                    let max_key = Id::from_u64(999999904);
                    let max_hov = ui.interaction_state.get(&max_key).copied().unwrap_or(0.0) > 0.5;
                    let max_resp = ui.container()
                        .width(BTN_W).height(BTN_W)
                        .radius_all(BTN_W / 2.0)
                        .bg(if max_hov {
                            match state.theme {
                                ThemeMode::Dark => Color::rgba(1.0, 1.0, 1.0, 0.08),
                                ThemeMode::Light => Color::rgba(0.0, 0.0, 0.0, 0.06),
                            }
                        } else { Color::TRANSPARENT })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{f0c8}").size(7.5).color(colors.text_muted).show();
                        });
                    let h = max_resp.hovered;
                    let prev = ui.interaction_state.insert(max_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if max_resp.clicked {
                        ui.window_actions.push(WindowAction::Maximize);
                    }

                    // Close
                    let cls_key = Id::from_u64(999999905);
                    let cls_hov = ui.interaction_state.get(&cls_key).copied().unwrap_or(0.0) > 0.5;
                    let cls_resp = ui.container()
                        .width(BTN_W).height(BTN_W)
                        .radius_all(BTN_W / 2.0)
                        .bg(if cls_hov { Color::rgb(0.85, 0.15, 0.15) } else { Color::TRANSPARENT })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            let fg = if cls_hov { Color::WHITE } else { colors.text_muted };
                            ui.text("\u{f00d}").size(8.0).color(fg).show();
                        });
                    let h = cls_resp.hovered;
                    let prev = ui.interaction_state.insert(cls_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if cls_resp.clicked {
                        ui.window_actions.push(WindowAction::Close);
                    }
                });
        });

    // Click outside (light dismiss) handling for menus
    let active_menu_key = Id::from_u64(999999900);
    let active_submenu_key = Id::from_u64(999999901);
    let hover_flag_key = Id::from_u64(999999902);

    let clicked = ui.clicked;
    let active_menu_id = ui.interaction_state.get(&active_menu_key).copied().map(|v| v as u64).unwrap_or(0);
    let hover_flag = ui.interaction_state.get(&hover_flag_key).copied().unwrap_or(0.0) > 0.5;

    if active_menu_id != 0 && clicked && !hover_flag {
        ui.interaction_state.insert(active_menu_key, 0.0);
        ui.interaction_state.insert(active_submenu_key, 0.0);
        ui.request_redraw();
    }
}
