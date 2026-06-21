use crate::state::FileManagerState;
use super::common::{
    NF_FA_ARROW_LEFT, NF_FA_ARROW_RIGHT, NF_FA_ARROW_UP, NF_FA_REFRESH, NF_FA_SEARCH,
    is_drag_drop_hovered, drop_target_bg,
};
use zenthra::{Color, Ui, Align};

pub fn draw_navigation_bar(ui: &mut Ui, state: &mut FileManagerState) {
    let colors = state.colors();

    ui.container()
        .fill_x()
        .height(42.0)
        .bg(colors.bg_base)
        .border(colors.border, 1.0)
        .padding(6.0, 12.0, 6.0, 12.0)
        .row()
        .valign(Align::Center)
        .show(|ui| {
            let start_x = ui.cursor_x;
            // Sidebar Toggle Button
            let toggle_icon = if state.sidebar_visible {
                "\u{ebf3}"
            } else {
                "\u{ec02}"
            };
            let toggle_btn = ui.button(toggle_icon)
                .width(28.0)
                .bg(colors.bg_panel)
                .hover_bg(colors.highlight)
                .text_color(if state.sidebar_visible { colors.accent } else { colors.text_muted })
                .radius_all(4.0)
                .padding(5.0, 0.0, 5.0, 0.0)
                .size(12.0)
                .show();
            if toggle_btn.clicked {
                state.sidebar_visible = !state.sidebar_visible;
                ui.request_redraw();
            }

            ui.spacing(8.0);

            // Navigation stack buttons
            ui.container()
                .row()
                .gap(4.0)
                .valign(Align::Center)
                .show(|ui| {
                    // Back
                    let can_go_back = state.history_idx > 0;
                    let back_btn = ui.button(NF_FA_ARROW_LEFT)
                        .size(11.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(if can_go_back { colors.highlight } else { Color::TRANSPARENT })
                        .text_color(if can_go_back { colors.text_primary } else { colors.text_dim })
                        .radius_all(4.0)
                        .padding(4.0, 6.0, 4.0, 6.0)
                        .show();
                    if back_btn.clicked && can_go_back {
                        state.go_back();
                        ui.request_redraw();
                    }

                    // Forward
                    let can_go_forward = state.history_idx + 1 < state.history.len();
                    let fwd_btn = ui.button(NF_FA_ARROW_RIGHT)
                        .size(11.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(if can_go_forward { colors.highlight } else { Color::TRANSPARENT })
                        .text_color(if can_go_forward { colors.text_primary } else { colors.text_dim })
                        .radius_all(4.0)
                        .padding(4.0, 6.0, 4.0, 6.0)
                        .show();
                    if fwd_btn.clicked && can_go_forward {
                        state.go_forward();
                        ui.request_redraw();
                    }

                    // Up
                    let has_parent = state.current_dir.parent().is_some();
                    let up_btn = ui.button(NF_FA_ARROW_UP)
                        .size(11.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(if has_parent { colors.highlight } else { Color::TRANSPARENT })
                        .text_color(if has_parent { colors.text_primary } else { colors.text_dim })
                        .radius_all(4.0)
                        .padding(4.0, 6.0, 4.0, 6.0)
                        .show();
                    if up_btn.clicked && has_parent {
                        state.go_up();
                        ui.request_redraw();
                    }

                    // Refresh
                    let refresh_btn = ui.button(NF_FA_REFRESH)
                        .size(11.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(colors.highlight)
                        .text_color(colors.text_primary)
                        .radius_all(4.0)
                        .padding(4.0, 6.0, 4.0, 6.0)
                        .show();
                    if refresh_btn.clicked {
                        state.scan_current_dir();
                        ui.request_redraw();
                    }
                });

            ui.spacing(12.0);

            let left_w = ui.cursor_x - start_x;
            let right_w = 385.0;
            let breadcrumbs_w = (ui.available_width - left_w - right_w).max(100.0);

            // Breadcrumbs Container (Clickable path segments)
            ui.container()
                .width(breadcrumbs_w - 28.0)
                .row()
                .gap(4.0)
                .valign(Align::Center)
                .clip(true)
                .show(|ui| {
                    let path = state.current_dir.clone();
                    let mut ancestors = Vec::new();
                    let mut current = Some(path.as_path());
                    while let Some(p) = current {
                        if !p.as_os_str().is_empty() {
                            ancestors.push(p);
                        }
                        current = p.parent();
                    }
                    ancestors.reverse();

                    // Only show the last 4 ancestors to fit breadcrumbs neatly
                    let start_idx = ancestors.len().saturating_sub(4);
                    if start_idx > 0 {
                        ui.text("... \u{f105}").color(colors.text_muted).size(10.0).show();
                    }

                    let total_ancestors = ancestors.len();
                    for (i, p) in ancestors.iter().enumerate().skip(start_idx) {
                        let name = p.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("/");
                        
                        let display_name = if name.is_empty() || name == "/" {
                            "Root"
                        } else {
                            name
                        };

                        let is_last = i == total_ancestors - 1;
                        
                        use std::hash::{Hash, Hasher};
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        p.to_path_buf().hash(&mut hasher);
                        let segment_id = zenthra::Id::from_u64(hasher.finish());

                        let mut is_drop_hovered = false;
                        if state.dragging_item.is_some() {
                            is_drop_hovered = is_drag_drop_hovered(ui, segment_id);
                        }

                        let segment_btn = ui.container()
                            .id(segment_id)
                            .padding(2.0, 6.0, 2.0, 6.0)
                            .radius_all(4.0)
                            .bg(if is_drop_hovered {
                                drop_target_bg(&colors, true)
                            } else {
                                Color::TRANSPARENT
                            })
                            .hover_bg(colors.highlight)
                            .show(|ui| {
                                ui.text(display_name)
                                    .size(11.5)
                                    .color(if is_last { colors.accent } else { colors.text_primary })
                                    .show();
                            });

                        if segment_btn.clicked {
                            state.change_dir(p.to_path_buf());
                            ui.request_redraw();
                        }

                        if (segment_btn.hovered || is_drop_hovered) && !ui.mouse_down {
                            if let Some(src_path) = state.dragging_item.clone() {
                                let dest_path = p.to_path_buf();
                                if state.selected_paths.contains(&src_path) {
                                    let paths: Vec<_> = state.selected_paths.iter().cloned().collect();
                                    for path_to_move in paths {
                                        state.move_item(&path_to_move, &dest_path);
                                    }
                                    state.selected_paths.clear();
                                } else {
                                    state.move_item(&src_path, &dest_path);
                                }
                                state.dragging_item = None;
                                ui.request_redraw();
                            }
                        }

                        if !is_last {
                            ui.text("\u{f105}")
                                .color(colors.text_dim)
                                .size(10.0)
                                .show();
                        }
                    }
                });

            // Copy Path Button beside breadcrumbs
            let copy_path_btn = ui.button("\u{f0c5}") // NF_FA_COPY
                .width(24.0)
                .size(11.0)
                .bg(Color::TRANSPARENT)
                .hover_bg(colors.highlight)
                .text_color(colors.text_muted)
                .radius_all(4.0)
                .padding(4.0, 0.0, 4.0, 0.0)
                .show();
            if copy_path_btn.clicked {
                state.copy_path_to_clipboard(&state.current_dir.to_string_lossy());
            }

            ui.spacing(12.0);

            // Search Bar Input
            ui.container()
                .width(200.0)
                .row()
                .valign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    ui.text(NF_FA_SEARCH).color(colors.text_muted).size(11.0).show();
                    
                    let prev_query = state.search_query.clone();
                    ui.input(&mut state.search_query, "search_input")
                        .width(170.0)
                        .min_width(0.0)
                        .size(11.0)
                        .color(colors.text_primary)
                        .bg(colors.bg_panel)
                        .border(colors.border, 1.0)
                        .radius_all(4.0)
                        .padding(3.0, 6.0, 3.0, 6.0)
                        .show();

                    if state.search_query != prev_query {
                        state.selected_paths.clear();
                        state.select_anchor = None;
                        ui.request_redraw();
                    }
                });

            ui.spacing(12.0);

            // 1. List View & Grid View Buttons
            let is_list = state.view_mode == crate::state::ViewMode::List;
            
            // List View Button
            let list_btn = ui.button("\u{f03a}") // NF_FA_LIST
                .width(26.0)
                .size(11.5)
                .bg(if is_list { colors.bg_panel } else { Color::TRANSPARENT })
                .hover_bg(colors.highlight)
                .text_color(if is_list { colors.accent } else { colors.text_muted })
                .radius_all(4.0)
                .padding(5.0, 0.0, 5.0, 0.0)
                .show();
            if list_btn.clicked {
                state.view_mode = crate::state::ViewMode::List;
                ui.request_redraw();
            }

            ui.spacing(4.0);

            // Grid View Button
            let grid_btn = ui.button("\u{f00a}") // NF_FA_TH
                .width(26.0)
                .size(11.5)
                .bg(if !is_list { colors.bg_panel } else { Color::TRANSPARENT })
                .hover_bg(colors.highlight)
                .text_color(if !is_list { colors.accent } else { colors.text_muted })
                .radius_all(4.0)
                .padding(5.0, 0.0, 5.0, 0.0)
                .show();
            if grid_btn.clicked {
                if is_list {
                    state.view_mode = crate::state::ViewMode::Large; // default grid size
                    ui.request_redraw();
                }
            }

            ui.spacing(12.0);

            // 2. Zoom Out & Zoom In (Grid Icon Size) Controls
            ui.container()
                .width(53.0)
                .row()
                .bg(colors.bg_panel)
                .border(colors.border, 1.0)
                .radius_all(4.0)
                .gap(1.0)
                .padding(1.0, 1.0, 1.0, 1.0)
                .show(|ui| {
                    let can_zoom_out = state.view_mode != crate::state::ViewMode::List;
                    let zoom_out_btn = ui.button("\u{f010}") // search-minus
                        .width(24.0)
                        .size(11.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(if can_zoom_out { colors.highlight } else { Color::TRANSPARENT })
                        .text_color(if can_zoom_out { colors.text_primary } else { colors.text_dim })
                        .radius_all(3.0)
                        .padding(4.0, 0.0, 4.0, 0.0)
                        .show();
                    if zoom_out_btn.clicked {
                        match state.view_mode {
                            crate::state::ViewMode::ExtraLarge => {
                                state.view_mode = crate::state::ViewMode::Large;
                            }
                            crate::state::ViewMode::Large => {
                                state.view_mode = crate::state::ViewMode::Medium;
                            }
                            crate::state::ViewMode::Medium => {
                                state.view_mode = crate::state::ViewMode::List;
                            }
                            crate::state::ViewMode::List => {}
                        }
                        ui.request_redraw();
                    }

                    let can_zoom_in = state.view_mode != crate::state::ViewMode::ExtraLarge;
                    let zoom_in_btn = ui.button("\u{f00e}") // search-plus
                        .width(24.0)
                        .size(11.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(if can_zoom_in { colors.highlight } else { Color::TRANSPARENT })
                        .text_color(if can_zoom_in { colors.text_primary } else { colors.text_dim })
                        .radius_all(3.0)
                        .padding(4.0, 0.0, 4.0, 0.0)
                        .show();
                    if zoom_in_btn.clicked {
                        match state.view_mode {
                            crate::state::ViewMode::List => {
                                state.view_mode = crate::state::ViewMode::Medium;
                            }
                            crate::state::ViewMode::Medium => {
                                state.view_mode = crate::state::ViewMode::Large;
                            }
                            crate::state::ViewMode::Large => {
                                state.view_mode = crate::state::ViewMode::ExtraLarge;
                            }
                            crate::state::ViewMode::ExtraLarge => {}
                        }
                        ui.request_redraw();
                    }
                });

            ui.spacing(12.0);

            // 3. Right Details Panel Toggle Button
            let right_toggle_icon = if state.details_visible {
                "\u{ebf4}" // Codicon layout-sidebar-right
            } else {
                "\u{ec01}" // Codicon layout-sidebar-right-off
            };
            let right_toggle_btn = ui.button(right_toggle_icon)
                .width(28.0)
                .bg(colors.bg_panel)
                .hover_bg(colors.highlight)
                .text_color(if state.details_visible { colors.accent } else { colors.text_muted })
                .radius_all(4.0)
                .padding(5.0, 0.0, 5.0, 0.0)
                .size(12.0)
                .show();
            if right_toggle_btn.clicked {
                state.details_visible = !state.details_visible;
                ui.request_redraw();
            }
        });
}
