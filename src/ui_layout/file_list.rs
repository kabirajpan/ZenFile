use crate::state::{FileManagerState, format_size, ViewMode};
use std::hash::{Hash, Hasher};
use super::common::{
    get_item_icon_path, get_folder_icon_path, tag_name_to_color,
    truncate_filename, truncate_str, format_date, open_file,
    is_drag_drop_hovered, drop_target_bg,
};
use zenthra::{Color, ImageSource, ObjectFit, Ui, FontWeight, Align, Id, PlatformEvent};

pub fn draw_file_list(ui: &mut Ui, state: &mut FileManagerState, width: f32) {
    let colors = state.colors();

    // Inline Rename Key & Click Handler
    if let Some(rename_path) = state.renaming_item.clone() {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::Hasher;
        rename_path.hash(&mut hasher);
        let rename_input_id = Id::from_u64(hasher.finish());

        let mut commit_rename = false;
        let mut cancel_rename = false;

        // 1. Check for enter or escape key press
        for event in &ui.input_events {
            if let PlatformEvent::KeyDown { key } = event {
                if *key == winit::keyboard::KeyCode::Enter {
                    commit_rename = true;
                } else if *key == winit::keyboard::KeyCode::Escape {
                    cancel_rename = true;
                }
            }
        }

        // 2. Check for click outside
        if ui.clicked {
            if let Some((rect, _)) = ui.get_recorded_layout(rename_input_id) {
                let rx = rect.origin.x + ui.offset_x;
                let ry = rect.origin.y + ui.offset_y;
                let rw = rect.size.width;
                let rh = rect.size.height;
                if ui.mouse_x < rx || ui.mouse_x > rx + rw || ui.mouse_y < ry || ui.mouse_y > ry + rh {
                    commit_rename = true;
                }
            }
        }

        if commit_rename {
            let mut new_path = rename_path.clone();
            new_path.set_file_name(&state.rename_buffer);
            if !new_path.exists() && !state.rename_buffer.trim().is_empty() {
                if std::fs::rename(&rename_path, &new_path).is_ok() {
                    state.renaming_item = None;
                    state.scan_current_dir();
                } else {
                    state.renaming_item = None;
                }
            } else {
                state.renaming_item = None;
            }
            ui.request_redraw();
        } else if cancel_rename {
            state.renaming_item = None;
            ui.request_redraw();
        }
    }

    let filtered_items = state.get_filtered_items();

    // Process Keyboard Events for Selection
    let input_events = ui.input_events.clone();
    for event in &input_events {
        match event {
            PlatformEvent::KeyDown { key } => {
                match key {
                    winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
                        state.ctrl_pressed = true;
                    }
                    winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
                        state.shift_pressed = true;
                    }
                    winit::keyboard::KeyCode::KeyA => {
                        if state.ctrl_pressed {
                            state.selected_paths.clear();
                            for item in &filtered_items {
                                state.selected_paths.insert(item.path.clone());
                            }
                            if !filtered_items.is_empty() {
                                state.selected_idx = Some(0);
                            }
                            ui.request_redraw();
                        }
                    }
                    winit::keyboard::KeyCode::KeyC => {
                        if state.ctrl_pressed {
                            state.copy_selected();
                            ui.request_redraw();
                        }
                    }
                    winit::keyboard::KeyCode::KeyX => {
                        if state.ctrl_pressed {
                            state.cut_selected();
                            ui.request_redraw();
                        }
                    }
                    winit::keyboard::KeyCode::KeyV => {
                        if state.ctrl_pressed {
                            state.paste_clipboard();
                            ui.request_redraw();
                        }
                    }
                    winit::keyboard::KeyCode::Escape => {
                        state.clear_selection();
                        ui.request_redraw();
                    }
                    winit::keyboard::KeyCode::Delete => {
                        if let Some(idx) = state.selected_idx {
                            if idx < state.items.len() {
                                state.delete_confirm = Some(state.items[idx].path.clone());
                                ui.request_redraw();
                            }
                        }
                    }
                    winit::keyboard::KeyCode::ArrowDown => {
                        let current_idx = state.last_clicked_idx.unwrap_or(0);
                        if current_idx + 1 < filtered_items.len() {
                            let next_idx = current_idx + 1;
                            state.last_clicked_idx = Some(next_idx);
                            if state.shift_pressed {
                                let anchor = state.select_anchor.unwrap_or(current_idx);
                                state.selected_paths.clear();
                                state.select_range(anchor, next_idx);
                            } else {
                                state.select_single(next_idx);
                            }
                            ui.request_redraw();
                        }
                    }
                    winit::keyboard::KeyCode::ArrowUp => {
                        let current_idx = state.last_clicked_idx.unwrap_or(0);
                        if current_idx > 0 {
                            let prev_idx = current_idx - 1;
                            state.last_clicked_idx = Some(prev_idx);
                            if state.shift_pressed {
                                let anchor = state.select_anchor.unwrap_or(current_idx);
                                state.selected_paths.clear();
                                state.select_range(anchor, prev_idx);
                            } else {
                                state.select_single(prev_idx);
                            }
                            ui.request_redraw();
                        }
                    }
                    _ => {}
                }
            }
            PlatformEvent::KeyUp { key } => {
                match key {
                    winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
                        state.ctrl_pressed = false;
                    }
                    winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
                        state.shift_pressed = false;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    let is_list = state.view_mode == ViewMode::List;
    let main_padding = if is_list { 8.0 } else { 2.0 };

    let mut item_right_clicked = false;
    let mut item_clicked = false;
    let mut item_pressed = false;
    let mut clicked_idx = None;
    state.item_rects.clear();

    let list_resp = ui.container()
        .width(width)
        .fill_y()
        .column()
        .bg(colors.bg_panel)
        .padding(main_padding, main_padding, main_padding, main_padding)
        .show(|ui| {
            let avail_w = ui.available_width;
            let inner_w = avail_w - 24.0; // 12.0 padding on left and right
            
            let col_name_w = (inner_w * 0.45).max(150.0);
            let col_size_w = (inner_w * 0.15).max(60.0);
            let col_type_w = (inner_w * 0.18).max(80.0);
            let col_date_w = (inner_w - col_name_w - col_size_w - col_type_w).max(80.0);

            // Column headers (list mode only)
            if is_list {
                ui.container()
                    .fill_x()
                    .height(30.0)
                    .bg(colors.bg_panel)
                    .border(colors.border, 1.0)
                    .radius_all(6.0)
                    .row()
                    .clip(true)
                    .valign(Align::Center)
                    .padding(0.0, 12.0, 0.0, 12.0)
                    .show(|ui| {
                        ui.container().width(col_name_w).show(|ui| {
                            ui.text("Name").size(11.0).weight(FontWeight::Bold).color(colors.text_muted).show();
                        });
                        ui.container().width(col_size_w).show(|ui| {
                            ui.text("Size").size(11.0).weight(FontWeight::Bold).color(colors.text_muted).show();
                        });
                        ui.container().width(col_type_w).show(|ui| {
                            ui.text("Type").size(11.0).weight(FontWeight::Bold).color(colors.text_muted).show();
                        });
                        ui.container().width(col_date_w).padding_right(12.0).show(|ui| {
                            ui.text("Date Modified").size(11.0).weight(FontWeight::Bold).color(colors.text_muted).show();
                        });
                    });

                ui.spacing(6.0);
            }

            if filtered_items.is_empty() {
                ui.container()
                    .fill()
                    .align(zenthra::Align::Center)
                    .show(|ui| {
                        ui.text("This folder is empty or matches no search results")
                            .color(colors.text_dim)
                            .size(11.5)
                            .show();
                    });
                return;
            }

            if is_list {
                let item_h = 28.0_f32;
                ui.lazy_container()
                    .id("file_list_virtual")
                    .column()
                    .item_size(avail_w, item_h)
                    .count(filtered_items.len())
                    .gap(2.0)
                    .show(|ui, idx| {
                        let item = &filtered_items[idx];
                        let is_selected = state.selected_paths.contains(&item.path);

                        let icon_path = get_item_icon_path(&state.icon_theme, &item.category, &item.extension);

                        let final_icon_path = if item.is_dir {
                            get_folder_icon_path(&state.icon_theme, &item.name, &state.folder_color, state.flat_folders)
                        } else {
                            icon_path.clone()
                        };

                        let is_menu_open = state.context_menu_pos.is_some();
                        let show_hover = !is_menu_open;

                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        item.path.hash(&mut hasher);
                        let item_id = Id::from_u64(hasher.finish());

                        let mut is_drop_hovered = false;
                        if item.is_dir && state.dragging_item.is_some() {
                            is_drop_hovered = is_drag_drop_hovered(ui, item_id);
                        }

                        let bg_color = if is_drop_hovered {
                            drop_target_bg(&colors, true)
                        } else if is_selected {
                            colors.bg_active
                        } else {
                            Color::TRANSPARENT
                        };

                        let resp = ui.container()
                            .id(item_id)
                            .row()
                            .fill_x()
                            .valign(Align::Center)
                            .padding(4.0, 12.0, 4.0, 12.0)
                            .bg(bg_color)
                            .hover_bg(if is_selected {
                                colors.bg_active
                            } else if show_hover {
                                colors.highlight
                            } else {
                                Color::TRANSPARENT
                            })
                            .radius_all(4.0)
                            .clip(true)
                            .show(|ui| {
                                ui.container().width(col_name_w).row().gap(8.0).valign(Align::Center).show(|ui| {
                                    ui.image(ImageSource::Path(final_icon_path))
                                        .size(16.0, 16.0)
                                        .fit(ObjectFit::Contain)
                                        .show();
                                    
                                    if state.renaming_item.as_ref() == Some(&item.path) {
                                        ui.input(&mut state.rename_buffer, &item.path)
                                            .width(col_name_w - 35.0)
                                            .min_width(0.0)
                                            .size(11.0)
                                            .bg(colors.bg_base)
                                            .color(colors.text_primary)
                                            .border(colors.accent.with_alpha(0.8), 0.6)
                                            .radius_all(4.0)
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .show();
                                    } else {
                                        let name_limit = ((col_name_w - 30.0) / 7.0).max(10.0) as usize;
                                        let display_name = truncate_filename(&item.name, name_limit);
                                        
                                        ui.container().row().gap(4.0).valign(Align::Center).show(|ui| {
                                            if let Some(tag_color_name) = state.file_tags.get(&item.path) {
                                                let tag_color = tag_name_to_color(tag_color_name);
                                                if tag_color != Color::TRANSPARENT {
                                                    ui.container()
                                                        .width(6.0)
                                                        .height(6.0)
                                                        .radius_all(3.0)
                                                        .bg(tag_color)
                                                        .show(|_| {});
                                                }
                                            }
                                            ui.text(&display_name)
                                                .color(if is_selected { colors.accent } else { colors.text_primary })
                                                .size(11.5)
                                                .show();
                                        });
                                    }
                                });

                                ui.container().width(col_size_w).show(|ui| {
                                    let size_text = if item.is_dir { "--".to_string() } else { format_size(item.size) };
                                    ui.text(&size_text).color(colors.text_muted).size(11.0).show();
                                });

                                ui.container().width(col_type_w).show(|ui| {
                                    let type_limit = (col_type_w / 7.0).max(8.0) as usize;
                                    let type_text = truncate_str(&item.display_type, type_limit);
                                    ui.text(&type_text).color(colors.text_muted).size(11.0).show();
                                });

                                ui.container().width(col_date_w).padding_right(12.0).show(|ui| {
                                    let date_text = format_date(item);
                                    ui.text(&date_text).color(colors.text_muted).size(11.0).show();
                                });
                            });

                        // Record screen position for marquee selection
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        item_id.hash(&mut hasher);
                        if let Some(parent) = ui.semantic_stack.last() {
                            parent.hash(&mut hasher);
                        }
                        let resolved_item_id = Id::from_u64(hasher.finish());

                        if let Some(rect) = ui.next_screen_layout_cache.get(&resolved_item_id) {
                            state.item_rects.push((item.path.clone(), rect.origin.x, rect.origin.y, rect.size.width, rect.size.height));
                        }

                        if resp.pressed {
                            item_pressed = true;
                        }

                        if resp.clicked {
                            if ui.right_clicked {
                                println!("DEBUG ITEM RIGHT CLICK (list): path={:?}", item.path);
                                if !state.selected_paths.contains(&item.path) {
                                    state.select_single(idx);
                                }
                                state.context_menu_pos = Some((ui.mouse_x, ui.mouse_y));
                                state.context_menu_target = state.items.iter().position(|it| it.path == item.path);
                                item_right_clicked = true;
                                ui.request_redraw();
                            } else if state.drag_select_start.is_none() {
                                println!("DEBUG ITEM LEFT CLICK (list): path={:?}", item.path);
                                if state.renaming_item.as_ref() == Some(&item.path) {
                                    // Do nothing, let input consume click
                                } else {
                                    clicked_idx = Some(idx);
                                }
                            }
                        }

                        // Drag Detection with Threshold (only when no marquee active)
                        if resp.pressed && state.drag_pressed_item.is_none() && state.drag_select_start.is_none() {
                            state.drag_pressed_item = Some(item.path.clone());
                            state.drag_start_pos = Some((ui.mouse_x, ui.mouse_y));
                        }

                        if ui.mouse_down && state.dragging_item.is_none() && state.drag_select_start.is_none() {
                            if let (Some(path), Some(start_pos)) = (&state.drag_pressed_item, state.drag_start_pos) {
                                if path == &item.path {
                                    let dx = ui.mouse_x - start_pos.0;
                                    let dy = ui.mouse_y - start_pos.1;
                                    if dx.abs() > 12.0 || dy.abs() > 12.0 {
                                        state.dragging_item = Some(path.clone());
                                        if let Some((rect, _)) = ui.get_recorded_layout(item_id) {
                                            let item_x = rect.origin.x + ui.offset_x;
                                            let item_y = rect.origin.y + ui.offset_y;
                                            state.drag_item_offset = Some((start_pos.0 - item_x, start_pos.1 - item_y));
                                        }
                                        ui.request_redraw();
                                    }
                                }
                            }
                        }

                        // Drop Detection onto folder
                        if item.is_dir && resp.hovered && !ui.mouse_down {
                            if let Some(src_path) = state.dragging_item.clone() {
                                if state.selected_paths.contains(&src_path) {
                                    let paths: Vec<_> = state.selected_paths.iter().cloned().collect();
                                    for p in paths {
                                        state.move_item(&p, &item.path);
                                    }
                                    state.selected_paths.clear();
                                } else {
                                    state.move_item(&src_path, &item.path);
                                }
                                state.dragging_item = None;
                                state.drag_pressed_item = None;
                                state.drag_start_pos = None;
                                state.drag_item_offset = None;
                                ui.request_redraw();
                            }
                        }
                    });
            } else {
                // Grid modes: Medium, Large, ExtraLarge (Highly optimized, tight paddings)
                let (tile_w, tile_h, icon_size, name_max, font_size) = match state.view_mode {
                    ViewMode::Medium     => (76.0_f32,  68.0_f32,  40.0_f32, 10, 9.5_f32),
                    ViewMode::Large      => (112.0_f32, 96.0_f32,  64.0_f32, 14, 10.5_f32),
                    ViewMode::ExtraLarge => (152.0_f32, 132.0_f32, 96.0_f32, 18, 11.0_f32),
                    _ => unreachable!(),
                };

                // Highly optimized virtualized grid using LazyContainer
                ui.lazy_container()
                    .item_size(tile_w, tile_h)
                    .count(filtered_items.len())
                    .gap(3.0)
                    .padding(1.0, 1.0, 1.0, 1.0)
                    .row()
                    .wrap(zenthra::Wrap::Wrap)
                    .id("file_manager_grid")
                    .show(|ui, idx| {
                        let item = &filtered_items[idx];
                        let is_selected = state.selected_paths.contains(&item.path);
                        let icon_path = get_item_icon_path(&state.icon_theme, &item.category, &item.extension);

                        let is_menu_open = state.context_menu_pos.is_some();
                        let show_hover = !is_menu_open;

                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        item.path.hash(&mut hasher);
                        let item_id = Id::from_u64(hasher.finish());

                        let mut is_drop_hovered = false;
                        if item.is_dir && state.dragging_item.is_some() {
                            is_drop_hovered = is_drag_drop_hovered(ui, item_id);
                        }

                        let bg_color = if is_drop_hovered {
                            drop_target_bg(&colors, true)
                        } else if is_selected {
                            colors.bg_active
                        } else {
                            Color::TRANSPARENT
                        };

                        let resp = ui.container()
                            .id(item_id)
                            .width(tile_w)
                            .height(tile_h)
                            .column()
                            .align(Align::Center)
                            .valign(Align::Center)
                            .gap(2.0)
                            .bg(bg_color)
                            .hover_bg(if is_selected {
                                colors.bg_active
                            } else if show_hover {
                                colors.highlight
                            } else {
                                Color::TRANSPARENT
                            })
                            .radius_all(6.0)
                            .padding(2.0, 2.0, 2.0, 2.0)
                            .clip(true)
                            .show(|ui| {
                                let final_icon_path = if item.is_dir {
                                    get_folder_icon_path(&state.icon_theme, &item.name, &state.folder_color, state.flat_folders)
                                } else {
                                    icon_path.clone()
                                };

                                ui.image(ImageSource::Path(final_icon_path))
                                    .size(icon_size, icon_size)
                                    .fit(ObjectFit::Contain)
                                    .show();

                                // Filename + Tag Dot
                                if state.renaming_item.as_ref() == Some(&item.path) {
                                    ui.input(&mut state.rename_buffer, &item.path)
                                        .width(tile_w - 8.0)
                                        .min_width(0.0)
                                        .size(font_size - 0.5)
                                        .bg(colors.bg_base)
                                        .color(colors.text_primary)
                                        .border(colors.accent.with_alpha(0.8), 0.6)
                                        .radius_all(4.0)
                                        .padding(1.0, 2.0, 1.0, 2.0)
                                        .show();
                                } else {
                                    let display_name = truncate_filename(&item.name, name_max);
                                    ui.container().row().gap(4.0).valign(Align::Center).show(|ui| {
                                        if let Some(tag_color_name) = state.file_tags.get(&item.path) {
                                            let tag_color = tag_name_to_color(tag_color_name);
                                            if tag_color != Color::TRANSPARENT {
                                                ui.container()
                                                    .width(6.0)
                                                    .height(6.0)
                                                    .radius_all(3.0)
                                                    .bg(tag_color)
                                                    .show(|_| {});
                                            }
                                        }
                                        ui.text(&display_name)
                                            .size(font_size)
                                            .color(if is_selected { colors.accent } else { colors.text_primary })
                                            .show();
                                    });
                                }
                            });

                        // Record screen position for marquee selection
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        item_id.hash(&mut hasher);
                        if let Some(parent) = ui.semantic_stack.last() {
                            parent.hash(&mut hasher);
                        }
                        let resolved_item_id = Id::from_u64(hasher.finish());

                        if let Some(rect) = ui.next_screen_layout_cache.get(&resolved_item_id) {
                            state.item_rects.push((item.path.clone(), rect.origin.x, rect.origin.y, rect.size.width, rect.size.height));
                        }

                        if resp.pressed {
                            item_pressed = true;
                        }

                        if resp.clicked {
                            if ui.right_clicked {
                                println!("DEBUG ITEM RIGHT CLICK (grid): path={:?}", item.path);
                                if !state.selected_paths.contains(&item.path) {
                                    state.select_single(idx);
                                }
                                state.context_menu_pos = Some((ui.mouse_x, ui.mouse_y));
                                state.context_menu_target = state.items.iter().position(|it| it.path == item.path);
                                item_right_clicked = true;
                                ui.request_redraw();
                            } else if state.drag_select_start.is_none() {
                                println!("DEBUG ITEM LEFT CLICK (grid): path={:?}", item.path);
                                if state.renaming_item.as_ref() == Some(&item.path) {
                                    // Do nothing, let input consume click
                                } else {
                                    clicked_idx = Some(idx);
                                }
                            }
                        }

                        // Drag Detection with Threshold (only when no marquee active)
                        if resp.pressed && state.drag_pressed_item.is_none() && state.drag_select_start.is_none() {
                            state.drag_pressed_item = Some(item.path.clone());
                            state.drag_start_pos = Some((ui.mouse_x, ui.mouse_y));
                        }

                        if ui.mouse_down && state.dragging_item.is_none() && state.drag_select_start.is_none() {
                            if let (Some(path), Some(start_pos)) = (&state.drag_pressed_item, state.drag_start_pos) {
                                if path == &item.path {
                                    let dx = ui.mouse_x - start_pos.0;
                                    let dy = ui.mouse_y - start_pos.1;
                                    if dx.abs() > 12.0 || dy.abs() > 12.0 {
                                        state.dragging_item = Some(path.clone());
                                        if let Some((rect, _)) = ui.get_recorded_layout(item_id) {
                                            let item_x = rect.origin.x + ui.offset_x;
                                            let item_y = rect.origin.y + ui.offset_y;
                                            state.drag_item_offset = Some((start_pos.0 - item_x, start_pos.1 - item_y));
                                        }
                                        ui.request_redraw();
                                    }
                                }
                            }
                        }

                        // Drop Detection onto folder in grid
                        if item.is_dir && resp.hovered && !ui.mouse_down {
                            if let Some(src_path) = state.dragging_item.clone() {
                                if state.selected_paths.contains(&src_path) {
                                    let paths: Vec<_> = state.selected_paths.iter().cloned().collect();
                                    for p in paths {
                                        state.move_item(&p, &item.path);
                                    }
                                    state.selected_paths.clear();
                                } else {
                                    state.move_item(&src_path, &item.path);
                                }
                                state.dragging_item = None;
                                state.drag_pressed_item = None;
                                state.drag_start_pos = None;
                                state.drag_item_offset = None;
                                ui.request_redraw();
                            }
                        }
                    });
            }

            if let Some(idx) = clicked_idx {
                item_clicked = true;
                let now = std::time::Instant::now();
                let is_double = if let (Some(last_time), Some(last_idx)) = (state.last_click_time, state.last_clicked_idx) {
                    let elapsed = now.duration_since(last_time).as_millis();
                    last_idx == idx && elapsed >= 80 && elapsed < 400
                } else {
                    false
                };

                if is_double {
                    state.last_click_time = None;
                    state.last_clicked_idx = None;
                    state.deferred_click_idx = None;
                    let target_path = filtered_items[idx].path.clone();
                    if filtered_items[idx].is_dir {
                        state.change_dir(target_path);
                    } else {
                        open_file(&target_path);
                    }
                } else {
                    state.last_click_time = Some(now);
                    state.last_clicked_idx = Some(idx);

                    let path = &filtered_items[idx].path;
                    let is_already_selected = state.selected_paths.contains(path);
                    let has_modifiers = state.ctrl_pressed || state.shift_pressed;

                    if is_already_selected && !has_modifiers {
                        state.deferred_click_idx = Some(idx);
                    } else {
                        state.deferred_click_idx = None;
                        if state.ctrl_pressed {
                            state.toggle_select(idx);
                        } else if state.shift_pressed && state.select_anchor.is_some() {
                            let anchor = state.select_anchor.unwrap();
                            state.selected_paths.clear();
                            state.select_range(anchor, idx);
                        } else {
                            state.select_single(idx);
                        }
                    }
                }
                ui.request_redraw();
            }
        });

    let ended_drag_select = !ui.mouse_down && state.drag_select_start.is_some();

    // Handle background drag-selection start
    if list_resp.pressed && !item_pressed && !item_right_clicked && state.dragging_item.is_none() && state.drag_pressed_item.is_none() && state.context_menu_pos.is_none() {
        if state.drag_select_start.is_none() {
            state.drag_select_start = Some((ui.mouse_x, ui.mouse_y));
            state.drag_select_current = Some((ui.mouse_x, ui.mouse_y));
            if !state.ctrl_pressed && !state.shift_pressed {
                state.selected_paths.clear();
            }
            println!("DEBUG MARQUEE START: pos=({}, {})", ui.mouse_x, ui.mouse_y);
            ui.request_redraw();
        }
    }

    // Update drag-selection rectangle and select intersecting files
    if ui.mouse_down && state.drag_select_start.is_some() {
        state.drag_select_current = Some((ui.mouse_x, ui.mouse_y));
        
        if let (Some(start), Some(curr)) = (state.drag_select_start, state.drag_select_current) {
            let x1 = start.0.min(curr.0);
            let y1 = start.1.min(curr.1);
            let w = (start.0 - curr.0).abs();
            let h = (start.1 - curr.1).abs();

            let mut intersected = std::collections::HashSet::new();
            for (path, rx, ry, rw, rh) in &state.item_rects {
                let x_overlap = *rx < x1 + w && *rx + *rw > x1;
                let y_overlap = *ry < y1 + h && *ry + *rh > y1;
                if x_overlap && y_overlap {
                    intersected.insert(path.clone());
                }
            }

            let old_paths: std::collections::HashSet<_> = state.selected_paths.iter().cloned().collect();
            if intersected != old_paths {
                println!("DEBUG MARQUEE UPDATE: intersected={:?}", intersected);
            }

            if state.ctrl_pressed {
                for p in intersected {
                    state.selected_paths.insert(p);
                }
            } else {
                state.selected_paths = intersected;
            }
            ui.request_redraw();
        }
    }

    // Drag-selection end
    if ended_drag_select {
        println!("DEBUG MARQUEE END: final_selection={:?}", state.selected_paths);
        state.drag_select_start = None;
        state.drag_select_current = None;
        ui.request_redraw();
    }

    // Render Drag-Selection Marquee Overlay
    if let (Some(start), Some(curr)) = (state.drag_select_start, state.drag_select_current) {
        let x1 = start.0.min(curr.0);
        let y1 = start.1.min(curr.1);
        let w = (start.0 - curr.0).abs();
        let h = (start.1 - curr.1).abs();

        ui.container()
            .overlay()
            .absolute(x1, y1)
            .width(w)
            .height(h)
            .bg(Color::rgba(255.0 / 255.0, 214.0 / 255.0, 0.0 / 255.0, 0.15))
            .border(Color::rgba(255.0 / 255.0, 214.0 / 255.0, 0.0 / 255.0, 0.6), 1.0)
            .show(|_| {});
    }

    if list_resp.clicked {
        if ui.right_clicked {
            if !item_right_clicked {
                println!("DEBUG EMPTY SPACE RIGHT CLICK");
                state.clear_selection();
                state.context_menu_pos = Some((ui.mouse_x, ui.mouse_y));
                state.context_menu_target = None;
                ui.request_redraw();
            }
        } else if !item_clicked && !ended_drag_select && state.context_menu_pos.is_none() {
            // Left click empty area clears selection
            println!("DEBUG EMPTY SPACE LEFT CLICK");
            state.last_click_time = None;
            state.last_clicked_idx = None;
            state.clear_selection();
            ui.request_redraw();
        }
    }

    // Drag-and-drop end of gesture release cleanup
    if !ui.mouse_down && (state.dragging_item.is_some() || state.drag_pressed_item.is_some()) {
        if state.dragging_item.is_none() {
            if let Some(idx) = state.deferred_click_idx {
                state.select_single(idx);
            }
        }
        state.deferred_click_idx = None;
        state.dragging_item = None;
        state.drag_pressed_item = None;
        state.drag_start_pos = None;
        state.drag_item_offset = None;
        ui.request_redraw();
    }

    // Render Drag Thumbnail tracking cursor (only when NOT in marquee selection)
    if state.drag_select_start.is_none() {
        if let Some(drag_path) = &state.dragging_item {
            let is_stack = state.selected_paths.len() > 1 && state.selected_paths.contains(drag_path);
            let drag_count = if is_stack { state.selected_paths.len() } else { 1 };

            let stack_paths = if is_stack {
                let mut paths: Vec<_> = state.selected_paths.iter().cloned().collect();
                if let Some(pos) = paths.iter().position(|p| p == drag_path) {
                    paths.swap(0, pos);
                }
                paths.truncate(3);
                paths
            } else {
                vec![drag_path.clone()]
            };

            let mouse_x = ui.mouse_x;
            let mouse_y = ui.mouse_y;

            let (ox, oy) = state.drag_item_offset.unwrap_or_else(|| {
                if state.view_mode == ViewMode::List {
                    (90.0, 14.0)
                } else {
                    let (tile_w, tile_h, _, _, _) = match state.view_mode {
                        ViewMode::Medium     => (76.0_f32,  68.0_f32,  40.0_f32, 10, 9.5_f32),
                        ViewMode::Large      => (112.0_f32, 96.0_f32,  64.0_f32, 14, 10.5_f32),
                        ViewMode::ExtraLarge => (152.0_f32, 132.0_f32, 96.0_f32, 18, 11.0_f32),
                        _ => (76.0_f32, 68.0_f32, 40.0_f32, 10, 9.5_f32),
                    };
                    (tile_w / 2.0, tile_h / 2.0)
                }
            });

            if state.view_mode == ViewMode::List {
                for (i, path) in stack_paths.iter().enumerate().rev() {
                    let level = i;
                    let x_offset = level as f32 * 6.0;
                    let y_offset = level as f32 * 6.0;
                    let opacity = match level {
                        0 => 1.0,
                        1 => 0.65,
                        _ => 0.35,
                    };

                    let filename = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    let is_dir = path.is_dir();
                    let icon_path = if is_dir {
                        get_folder_icon_path(&state.icon_theme, &filename, &state.folder_color, state.flat_folders)
                    } else {
                        let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                        let (_, category) = crate::state::get_file_info(path);
                        get_item_icon_path(&state.icon_theme, &category, &extension)
                    };

                    let img_src = ImageSource::Path(icon_path.clone());
                    let (w, h) = if let Some((orig_w, orig_h)) = ui.image_sizes.get(&img_src) {
                        let orig_w = *orig_w as f32;
                        let orig_h = *orig_h as f32;
                        let ratio = orig_w / orig_h;
                        if ratio > 1.0 {
                            (16.0, 16.0 / ratio)
                        } else {
                            (16.0 * ratio, 16.0)
                        }
                    } else {
                        (16.0, 16.0)
                    };

                    ui.container()
                        .overlay()
                        .absolute(mouse_x - ox + x_offset, mouse_y - oy - y_offset)
                        .width(180.0)
                        .height(28.0)
                        .padding(4.0, 12.0, 4.0, 12.0)
                        .row()
                        .gap(8.0)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.image(img_src)
                                .size(w, h)
                                .fit(ObjectFit::Contain)
                                .opacity(opacity)
                                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.65), 0.0, 3.0, 6.0)
                                .show();
                            if level == 0 {
                                ui.text(&filename)
                                    .size(11.5)
                                    .color(colors.text_primary)
                                    .shadow(Color::rgba(0.0, 0.0, 0.0, 0.65), 0.0, 1.0, 2.0)
                                    .show();
                            }
                        });
                }

                if is_stack {
                    ui.container()
                        .overlay()
                        .absolute(mouse_x - ox + 22.0, mouse_y - oy - 2.0)
                        .width(18.0)
                        .height(18.0)
                        .bg(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0))
                        .radius_all(9.0)
                        .row()
                        .align(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text(&drag_count.to_string())
                                .size(9.0)
                                .color(Color::WHITE)
                                .bold()
                                .show();
                        });
                }
            } else {
                let (tile_w, tile_h, icon_size, name_max, font_size) = match state.view_mode {
                    ViewMode::Medium     => (76.0_f32,  68.0_f32,  40.0_f32, 10, 9.5_f32),
                    ViewMode::Large      => (112.0_f32, 96.0_f32,  64.0_f32, 14, 10.5_f32),
                    ViewMode::ExtraLarge => (152.0_f32, 132.0_f32, 96.0_f32, 18, 11.0_f32),
                    _ => (76.0_f32, 68.0_f32, 40.0_f32, 10, 9.5_f32),
                };

                for (i, path) in stack_paths.iter().enumerate().rev() {
                    let level = i;
                    let x_offset = level as f32 * 8.0;
                    let y_offset = level as f32 * 8.0;
                    let opacity = match level {
                        0 => 1.0,
                        1 => 0.65,
                        _ => 0.35,
                    };

                    let filename = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    let is_dir = path.is_dir();
                    let icon_path = if is_dir {
                        get_folder_icon_path(&state.icon_theme, &filename, &state.folder_color, state.flat_folders)
                    } else {
                        let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                        let (_, category) = crate::state::get_file_info(path);
                        get_item_icon_path(&state.icon_theme, &category, &extension)
                    };

                    let img_src = ImageSource::Path(icon_path.clone());
                    let (w, h) = if let Some((orig_w, orig_h)) = ui.image_sizes.get(&img_src) {
                        let orig_w = *orig_w as f32;
                        let orig_h = *orig_h as f32;
                        let ratio = orig_w / orig_h;
                        if ratio > 1.0 {
                            (icon_size, icon_size / ratio)
                        } else {
                            (icon_size * ratio, icon_size)
                        }
                    } else {
                        (icon_size, icon_size)
                    };

                    ui.container()
                        .overlay()
                        .absolute(mouse_x - ox + x_offset, mouse_y - oy - y_offset)
                        .width(tile_w)
                        .height(tile_h)
                        .padding(2.0, 2.0, 2.0, 2.0)
                        .column()
                        .align(Align::Center)
                        .valign(Align::Center)
                        .gap(2.0)
                        .show(|ui| {
                            ui.image(img_src)
                                .size(w, h)
                                .fit(ObjectFit::Contain)
                                .opacity(opacity)
                                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.65), 0.0, 4.0, 8.0)
                                .show();
                            if level == 0 {
                                let display_name = truncate_filename(&filename, name_max);
                                ui.text(&display_name)
                                    .size(font_size)
                                    .color(colors.text_primary)
                                    .shadow(Color::rgba(0.0, 0.0, 0.0, 0.65), 0.0, 1.0, 2.0)
                                    .show();
                            }
                        });
                }

                if is_stack {
                    let badge_x = mouse_x - ox + (tile_w + icon_size) / 2.0 - 8.0;
                    let badge_y = mouse_y - oy + (tile_h - icon_size) / 3.0 - 8.0;
                    ui.container()
                        .overlay()
                        .absolute(badge_x, badge_y)
                        .width(20.0)
                        .height(20.0)
                        .bg(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0))
                        .radius_all(10.0)
                        .row()
                        .align(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text(&drag_count.to_string())
                                .size(10.0)
                                .color(Color::WHITE)
                                .bold()
                                .show();
                        });
                }
            }
        }
    }
}
