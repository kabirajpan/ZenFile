use crate::state::{FileManagerState, format_size};
use crate::theme::ThemeColors;
use super::common::{
    get_folder_icon_path, get_item_icon_path, format_date, tag_name_to_color,
};
use zenthra::{Color, ImageSource, ObjectFit, Ui, FontWeight, Align, Response};

pub fn draw_about_window(ui: &mut Ui, state: &mut FileManagerState) {
    if !state.show_about {
        return;
    }

    let colors = state.colors();
    let mut about_pos = [state.about_x, state.about_y];

    ui.window("About ZenFile", &mut state.show_about, &mut about_pos)
        .size(360.0, 240.0)
        .modal(true)
        .bg(colors.bg_panel)
        .border(colors.accent, 1.5)
        .radius_all(12.0)
        .header_bg(colors.bg_base)
        .header_text_color(colors.text_primary)
        .header_height(40.0)
        .closable(true)
        .show(|ui| {
            ui.container()
                .full_width()
                .padding_all(14.0)
                .column()
                .gap(8.0)
                .show(|ui| {
                    ui.text("A clean, virtualized file browser application built directly on top of the Zenthra immediate-mode GUI framework.")
                        .size(11.0)
                        .color(colors.text_muted)
                        .show();

                    ui.spacing(10.0);

                    ui.container()
                        .column()
                        .gap(5.0)
                        .show(|ui| {
                            ui.container().row().show(|ui| {
                                ui.container().width(70.0).show(|ui| {
                                    ui.text("Version:").size(10.5).color(colors.text_dim).show();
                                });
                                ui.text("0.1.0").size(10.5).color(colors.text_muted).show();
                            });

                            ui.spacing(4.0);

                            ui.container().row().show(|ui| {
                                ui.container().width(70.0).show(|ui| {
                                    ui.text("Engine:").size(10.5).color(colors.text_dim).show();
                                });
                                ui.text("Zenthra v0.1.1").size(10.5).color(colors.text_muted).show();
                            });

                            ui.spacing(4.0);

                            ui.container().row().show(|ui| {
                                ui.container().width(70.0).show(|ui| {
                                    ui.text("Developer:").size(10.5).color(colors.text_dim).show();
                                });
                                ui.text("kabirajpan").size(10.5).color(colors.text_muted).show();
                            });
                        });
                });
        });

    if about_pos[0] != state.about_x || about_pos[1] != state.about_y {
        state.about_x = about_pos[0];
        state.about_y = about_pos[1];
        ui.request_redraw();
    }
}

pub fn draw_context_menu(ui: &mut Ui, state: &mut FileManagerState) {
    let Some((mx, my)) = state.context_menu_pos else {
        return;
    };

    let colors = state.colors();
    let menu_w = 200.0;
    
    // Calculate menu height dynamically based on options.
    let mut menu_h = 12.0; // padding top/bottom (6.0 each)
    if let Some(target_idx) = state.context_menu_target {
        let is_dir = state.items.get(target_idx).map(|it| it.is_dir).unwrap_or(false);
        menu_h += 26.0; // Open
        if is_dir {
            menu_h += 26.0; // Open in Terminal
        }
        menu_h += 26.0; // Rename
        menu_h += 26.0; // Copy
        menu_h += 26.0; // Copy Path
        menu_h += 26.0; // Cut
        menu_h += 26.0; // Paste
        menu_h += 9.0;  // Divider
        menu_h += 26.0; // Get Info
        menu_h += 26.0; // Move to Trash
        menu_h += 9.0;  // Divider
        menu_h += 20.0; // Tags
    } else {
        menu_h += 26.0; // New Folder
        menu_h += 26.0; // New File
        menu_h += 26.0; // Paste
        menu_h += 26.0; // Open in Terminal
        menu_h += 26.0; // Refresh
    }

    if ui.clicked {
        let mouse_in_menu = ui.mouse_x >= mx && ui.mouse_x <= mx + menu_w && ui.mouse_y >= my && ui.mouse_y <= my + menu_h;
        if !mouse_in_menu {
            state.context_menu_pos = None;
            state.context_menu_target = None;
            ui.request_redraw();
            return;
        }
    }

    let mut x = mx;
    let mut y = my;
    if x + menu_w > ui.available_width {
        x = ui.available_width - menu_w - 5.0;
    }
    if y + menu_h > ui.max_y {
        y = ui.max_y - menu_h - 5.0;
    }

    ui.container()
        .id("context_menu_root")
        .overlay()
        .absolute(x, y)
        .width(menu_w)
        .height(menu_h)
        .bg(colors.bg_panel)
        .border(colors.border, 1.0)
        .radius_all(8.0)
        .padding(6.0, 6.0, 6.0, 6.0)
        .column()
        .show(|ui| {
            if let Some(target_idx) = state.context_menu_target {
                let item = match state.items.get(target_idx) {
                    Some(it) => it.clone(),
                    None => return,
                };

                // Option: Open
                let open_btn = draw_menu_item(ui, "Open", colors, "ctx_open");
                if open_btn.clicked {
                    let mut paths_to_open = Vec::new();
                    if state.selected_paths.contains(&item.path) {
                        paths_to_open = state.selected_paths.iter().cloned().collect();
                    } else {
                        paths_to_open.push(item.path.clone());
                    }

                    for path in paths_to_open {
                        if path.is_dir() {
                            state.current_dir = path.clone();
                            state.history.truncate(state.history_idx + 1);
                            state.history.push(path.clone());
                            state.history_idx = state.history.len() - 1;
                            state.scan_current_dir();
                        } else {
                            super::common::open_file(&path);
                        }
                    }
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Option: Open in Terminal (directories only)
                if item.is_dir {
                    let term_btn = draw_menu_item(ui, "Open in Terminal", colors, "ctx_open_terminal");
                    if term_btn.clicked {
                        state.open_terminal_in(&item.path);
                        state.context_menu_pos = None;
                        state.context_menu_target = None;
                        ui.request_redraw();
                    }
                }

                // Option: Rename
                let rename_btn = draw_menu_item(ui, "Rename...", colors, "ctx_rename");
                if rename_btn.clicked {
                    super::common::start_rename(ui, state, item.path.clone(), item.name.clone());
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Option: Copy
                let copy_btn = draw_menu_item(ui, "Copy", colors, "ctx_copy");
                if copy_btn.clicked {
                    state.copy_selected();
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Option: Copy Path
                let copy_path_btn = draw_menu_item(ui, "Copy Path", colors, "ctx_copy_path");
                if copy_path_btn.clicked {
                    let mut paths_to_copy = Vec::new();
                    if state.selected_paths.contains(&item.path) {
                        paths_to_copy = state.selected_paths.iter().cloned().collect();
                    } else {
                        paths_to_copy.push(item.path.clone());
                    }
                    paths_to_copy.sort();
                    let paths_str = paths_to_copy.iter()
                        .map(|p| p.to_string_lossy().to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                    state.copy_path_to_clipboard(&paths_str);
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Option: Cut
                let cut_btn = draw_menu_item(ui, "Cut", colors, "ctx_cut");
                if cut_btn.clicked {
                    state.cut_selected();
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Option: Paste
                let can_paste = state.clipboard.is_some();
                let paste_btn = draw_menu_item_disabled(ui, "Paste", colors, !can_paste, "ctx_paste");
                if paste_btn.clicked && can_paste {
                    state.paste_clipboard();
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Divider
                ui.spacing(4.0);
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                ui.spacing(4.0);

                // Option: Get Info
                let info_btn = draw_menu_item(ui, "Get Info", colors, "ctx_get_info");
                if info_btn.clicked {
                    state.info_window_target = Some(target_idx);
                    state.info_window_open = true;
                    state.info_window_pos = [x + menu_w + 10.0, y];
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Option: Move to Trash
                let trash_btn = draw_menu_item_danger(ui, "Move to Trash", colors, "ctx_move_to_trash");
                if trash_btn.clicked {
                    let mut paths_to_trash = Vec::new();
                    if state.selected_paths.contains(&item.path) {
                        paths_to_trash = state.selected_paths.iter().cloned().collect();
                    } else {
                        paths_to_trash.push(item.path.clone());
                    }

                    let paths_to_trash_thread = paths_to_trash.clone();
                    std::thread::spawn(move || {
                        for path in &paths_to_trash_thread {
                            let _ = std::process::Command::new("gio")
                                .arg("trash")
                                .arg(path)
                                .status()
                                .map(|s| s.success())
                                .or_else(|_| {
                                    if path.is_dir() {
                                        std::fs::remove_dir_all(path).map(|_| true)
                                    } else {
                                        std::fs::remove_file(path).map(|_| true)
                                    }
                                });
                        }
                    });

                    let paths_set: std::collections::HashSet<_> = paths_to_trash.into_iter().collect();
                    state.items.retain(|it| !paths_set.contains(&it.path));
                    state.clear_selection();
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Divider
                ui.spacing(4.0);
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                ui.spacing(4.0);

                // Tags sub-layout: colored dots at the bottom
                ui.container()
                    .fill_x()
                    .row()
                    .valign(Align::Center)
                    .gap(6.0)
                    .padding_left(8.0)
                    .show(|ui| {
                        let tag_options: [(&str, Color); 7] = [
                            ("red", tag_name_to_color("red")),
                            ("orange", tag_name_to_color("orange")),
                            ("yellow", tag_name_to_color("yellow")),
                            ("green", tag_name_to_color("green")),
                            ("blue", tag_name_to_color("blue")),
                            ("purple", tag_name_to_color("purple")),
                            ("none", Color::rgb(120.0 / 255.0, 120.0 / 255.0, 120.0 / 255.0)),
                        ];
                        for (tag_name, tag_color) in tag_options {
                            let dot = ui.container()
                                .id(format!("tag_{}", tag_name))
                                .width(12.0)
                                .height(12.0)
                                .radius_all(6.0)
                                .bg(tag_color)
                                .hover_bg(Color::WHITE)
                                .show(|_| {});
                            if dot.clicked {
                                let mut paths_to_tag = Vec::new();
                                if state.selected_paths.contains(&item.path) {
                                    paths_to_tag = state.selected_paths.iter().cloned().collect();
                                } else {
                                    paths_to_tag.push(item.path.clone());
                                }

                                for path in paths_to_tag {
                                    if tag_name == "none" {
                                        state.file_tags.remove(&path);
                                    } else {
                                        state.file_tags.insert(path.clone(), tag_name.to_string());
                                    }
                                }
                                state.save_tags();
                                state.context_menu_pos = None;
                                state.context_menu_target = None;
                                ui.request_redraw();
                            }
                        }
                    });

            } else {
                // Background context menu
                // Option: New Folder
                let new_folder_btn = draw_menu_item(ui, "New Folder", colors, "ctx_new_folder");
                if new_folder_btn.clicked {
                    let mut i = 1;
                    let mut new_dir = state.current_dir.join("New Folder");
                    while new_dir.exists() {
                        new_dir = state.current_dir.join(format!("New Folder {}", i));
                        i += 1;
                    }
                    if std::fs::create_dir(&new_dir).is_ok() {
                        state.scan_current_dir();
                        let name = new_dir.file_name().unwrap().to_string_lossy().into_owned();
                        super::common::start_rename(ui, state, new_dir, name);
                    }
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Option: New File
                let new_file_btn = draw_menu_item(ui, "New File", colors, "ctx_new_file");
                if new_file_btn.clicked {
                    let mut i = 1;
                    let mut new_file = state.current_dir.join("New File.txt");
                    while new_file.exists() {
                        new_file = state.current_dir.join(format!("New File {}.txt", i));
                        i += 1;
                    }
                    if std::fs::File::create(&new_file).is_ok() {
                        state.scan_current_dir();
                        let name = new_file.file_name().unwrap().to_string_lossy().into_owned();
                        super::common::start_rename(ui, state, new_file, name);
                    }
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Option: Paste
                let can_paste = state.clipboard.is_some();
                let paste_btn = draw_menu_item_disabled(ui, "Paste", colors, !can_paste, "ctx_paste");
                if paste_btn.clicked && can_paste {
                    state.paste_clipboard();
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Option: Open in Terminal
                let term_btn = draw_menu_item(ui, "Open in Terminal", colors, "ctx_open_terminal_bg");
                if term_btn.clicked {
                    state.open_terminal_in(&state.current_dir);
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }

                // Option: Refresh
                let refresh_btn = draw_menu_item(ui, "Refresh", colors, "ctx_refresh");
                if refresh_btn.clicked {
                    state.scan_current_dir();
                    state.context_menu_pos = None;
                    state.context_menu_target = None;
                    ui.request_redraw();
                }
            }
        });
}

fn draw_menu_item(ui: &mut Ui, label: &str, colors: ThemeColors, id_str: &str) -> Response {
    ui.container()
        .id(id_str)
        .fill_x()
        .height(26.0)
        .row()
        .valign(Align::Center)
        .padding(2.0, 8.0, 2.0, 8.0)
        .bg(Color::TRANSPARENT)
        .hover_bg(colors.highlight)
        .radius_all(4.0)
        .show(|ui| {
            ui.text(label)
                .size(11.0)
                .color(colors.text_primary)
                .show();
        })
}

fn draw_menu_item_disabled(ui: &mut Ui, label: &str, colors: ThemeColors, disabled: bool, id_str: &str) -> Response {
    let mut c = ui.container()
        .id(id_str)
        .fill_x()
        .height(26.0)
        .row()
        .valign(Align::Center)
        .padding(2.0, 8.0, 2.0, 8.0)
        .bg(Color::TRANSPARENT)
        .radius_all(4.0);
    if !disabled {
        c = c.hover_bg(colors.highlight);
    }
    c.show(|ui| {
        ui.text(label)
            .size(11.0)
            .color(if disabled { colors.text_muted } else { colors.text_primary })
            .show();
    })
}

fn draw_menu_item_danger(ui: &mut Ui, label: &str, colors: ThemeColors, id_str: &str) -> Response {
    ui.container()
        .id(id_str)
        .fill_x()
        .height(26.0)
        .row()
        .valign(Align::Center)
        .padding(2.0, 8.0, 2.0, 8.0)
        .bg(Color::TRANSPARENT)
        .hover_bg(Color::rgb(231.0 / 255.0, 76.0 / 255.0, 60.0 / 255.0))
        .radius_all(4.0)
        .show(|ui| {
            ui.text(label)
                .size(11.0)
                .color(colors.text_primary)
                .show();
        })
}

pub fn draw_info_window(ui: &mut Ui, state: &mut FileManagerState) {
    if !state.info_window_open {
        return;
    }
    let Some(target_idx) = state.info_window_target else {
        return;
    };
    let item = match state.items.get(target_idx) {
        Some(it) => it.clone(),
        None => {
            state.info_window_open = false;
            return;
        }
    };

    let colors = state.colors();

    ui.window("Get Info", &mut state.info_window_open, &mut state.info_window_pos)
        .size(320.0, 280.0)
        .bg(colors.bg_panel)
        .border(colors.accent, 1.0)
        .radius_all(10.0)
        .header_bg(colors.bg_base)
        .header_text_color(colors.text_primary)
        .header_height(36.0)
        .closable(true)
        .show(|ui| {
            ui.container()
                .full_width()
                .padding_all(16.0)
                .column()
                .gap(10.0)
                .show(|ui| {
                    // Header: Icon + Name
                    ui.container().row().gap(12.0).valign(Align::Center).show(|ui| {
                        let final_icon_path = if item.is_dir {
                            get_folder_icon_path(&state.icon_theme, &item.name, &state.folder_color, state.flat_folders)
                        } else {
                            get_item_icon_path(&state.icon_theme, &item.category, &item.extension)
                        };
                        ui.image(ImageSource::Path(final_icon_path))
                            .size(36.0, 36.0)
                            .fit(ObjectFit::Contain)
                            .show();

                        ui.text(&item.name)
                            .size(13.0)
                            .weight(FontWeight::Bold)
                            .color(colors.text_primary)
                            .show();
                    });

                    // Divider
                    ui.spacing(4.0);
                    ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                    ui.spacing(4.0);

                    // Properties
                    let props = [
                        ("Kind:", if item.is_dir { "Folder".to_string() } else { format!("{} ({})", item.display_type, item.extension) }),
                        ("Size:", if item.is_dir { "Calculating...".to_string() } else { format_size(item.size) }),
                        ("Where:", item.path.parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default()),
                        ("Created / Modified:", format_date(&item)),
                    ];

                    for (label, val) in props {
                        ui.container().row().gap(8.0).show(|ui| {
                            ui.container().width(70.0).show(|ui| {
                                ui.text(label).color(colors.text_muted).size(11.0).show();
                            });
                            ui.container().width(200.0).show(|ui| {
                                ui.text(&val).color(colors.text_primary).size(11.0).show();
                            });
                        });
                    }

                    // Divider
                    ui.spacing(4.0);
                    ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                    ui.spacing(4.0);

                    // Tag section inside info window
                    ui.container().row().gap(8.0).valign(Align::Center).show(|ui| {
                        ui.container().width(70.0).show(|ui| {
                            ui.text("Tag Color:").color(colors.text_muted).size(11.0).show();
                        });
                        let current_tag = state.file_tags.get(&item.path).map(|s| s.as_str()).unwrap_or("none");
                        ui.text(current_tag).color(colors.accent).size(11.0).weight(FontWeight::Bold).show();
                    });
                });
        });
}
