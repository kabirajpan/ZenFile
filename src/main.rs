mod state;
mod theme;
mod ui_layout;
mod assets;

use state::FileManagerState;
use theme::ThemeMode;
use zenthra::prelude::*;
use zenthra::Id;

fn main() {
    env_logger::init();

    // Initialize the modular state
    let mut state = FileManagerState::new();

    let font_bytes = assets::Assets::get("fonts/SymbolsNerdFont-Regular.ttf")
        .expect("Failed to find Nerd Font in embedded assets")
        .data
        .into_owned();

    App::new()
        .title("ZenFile")
        .size(1100, 680)
        .decorations(false)
        .load_font_data(font_bytes)
        .with_ui(move |ui| {
            // Close panels if clicked outside of them
            if ui.clicked {
                if state.zendrop_open {
                    let clicked_panel = state.wifi_panel_rect.map(|(rx, ry, rw, rh)| {
                        ui.mouse_x >= rx && ui.mouse_x <= rx + rw && ui.mouse_y >= ry && ui.mouse_y <= ry + rh
                    }).unwrap_or(false);
                    let clicked_btn = state.wifi_btn_rect.map(|(rx, ry, rw, rh)| {
                        ui.mouse_x >= rx && ui.mouse_x <= rx + rw && ui.mouse_y >= ry && ui.mouse_y <= ry + rh
                    }).unwrap_or(false);
                    if !clicked_panel && !clicked_btn {
                        state.zendrop_open = false;
                        ui.request_redraw();
                    }
                }
                if state.zendrop_notif_open {
                    let clicked_panel = state.bell_panel_rect.map(|(rx, ry, rw, rh)| {
                        ui.mouse_x >= rx && ui.mouse_x <= rx + rw && ui.mouse_y >= ry && ui.mouse_y <= ry + rh
                    }).unwrap_or(false);
                    let clicked_btn = state.bell_btn_rect.map(|(rx, ry, rw, rh)| {
                        ui.mouse_x >= rx && ui.mouse_x <= rx + rw && ui.mouse_y >= ry && ui.mouse_y <= ry + rh
                    }).unwrap_or(false);
                    if !clicked_panel && !clicked_btn {
                        state.zendrop_notif_open = false;
                        ui.request_redraw();
                    }
                }
            }

            let colors = state.colors();

            // Inject theme context for component tree
            provide_context(colors);
            provide_context(state.theme);

            // Expose the active theme flag to interaction_state for menu compatibility
            let theme_val = if state.theme == ThemeMode::Light { 1.0 } else { 0.0 };
            ui.interaction_state.insert(Id::from_u64(999999999), theme_val);

            let glass_val = if state.theme == ThemeMode::Glassmorphism { 1.0 } else { 0.0 };
            ui.interaction_state.insert(Id::from_u64(999999998), glass_val);

            // Main Background Container (lays out Title Bar, Navigation, Content, and Status Bar vertically)
            ui.container()
                .fill()
                .bg(colors.bg_base)
                .show(|ui| {
                    if let Some(msg) = state.zendrop_status_msg.lock().unwrap().take() {
                        state.zendrop_send_status = msg;
                        ui.request_redraw();
                    }

                    // Subtle background glows to show through the glassmorphic panels
                    if state.theme == ThemeMode::Glassmorphism {
                        let glow1_color = theme::named_color(&state.glassmorphism_glow1_color, state.theme);
                        let glow2_color = theme::named_color(&state.glassmorphism_glow2_color, state.theme);
                        ui.container()
                            .absolute(40.0, 80.0)
                            .width(320.0)
                            .height(320.0)
                            .bg(glow1_color.with_alpha(0.08)) // 8% accent highlight
                            .radius_all(160.0)
                            .show(|_| {});

                        ui.container()
                            .absolute(750.0, 200.0)
                            .width(360.0)
                            .height(360.0)
                            .bg(glow2_color.with_alpha(0.08)) // 8% accent highlight
                            .radius_all(180.0)
                            .show(|_| {});
                    }
                    // 1. Custom window title bar & controls
                    ui_layout::draw_title_bar(ui, &mut state);

                    // 2. Navigation toolbar
                    ui_layout::draw_navigation_bar(ui, &mut state);



                    // 3. Central work area (Sidebar + File List + Preview panel)
                    ui.container()
                        .row()
                        .fill()
                        .gap(0.0)
                        .show(|ui| {
                            let colors = state.colors();

                            // Left sidebar shortcuts
                            if state.sidebar_visible {
                                ui_layout::draw_sidebar(ui, &mut state);

                                 // Sidebar Splitter handle
                                 let splitter_res = ui.container()
                                     .id(Id::from_u64(888888881))
                                     .width(4.0)
                                     .fill_y()
                                     .bg(colors.border)
                                     .hover_bg(colors.highlight)
                                     .show(|_| {});

                                 if (splitter_res.hovered || state.active_resize_sidebar) && state.dragging_item.is_none() {
                                     ui.cursor_icon = CursorIcon::ColResize;
                                 }

                                 if splitter_res.pressed && ui.clicked && state.dragging_item.is_none() {
                                     state.active_resize_sidebar = true;
                                 }
                                if !ui.mouse_down {
                                    state.active_resize_sidebar = false;
                                }
                                if state.active_resize_sidebar {
                                    state.sidebar_width = ui.mouse_x.clamp(160.0, 450.0);
                                    ui.request_redraw();
                                }
                            }

                            // Calculate file list width manually
                            let sidebar_w = if state.sidebar_visible { state.sidebar_width } else { 0.0 };
                            let details_w = if state.details_visible { state.details_width } else { 0.0 };
                            let splitters_w = if state.sidebar_visible { 4.0 } else { 0.0 } + if state.details_visible { 4.0 } else { 0.0 };
                            let file_list_w = (ui.available_width - sidebar_w - details_w - splitters_w).max(200.0);

                            // Main content (File Grid or Dashboard)
                            match state.active_screen {
                                crate::state::ActiveScreen::Browser => {
                                    ui_layout::draw_file_list(ui, &mut state, file_list_w);
                                }
                                crate::state::ActiveScreen::Dashboard => {
                                    ui_layout::draw_dashboard(ui, &mut state, file_list_w);
                                }
                            }

                            // Right preview / info pane
                            if state.details_visible {
                                 // Details Splitter handle
                                 let splitter_res = ui.container()
                                     .id(Id::from_u64(888888882))
                                     .width(4.0)
                                     .fill_y()
                                     .bg(colors.border)
                                     .hover_bg(colors.highlight)
                                     .show(|_| {});

                                 if (splitter_res.hovered || state.active_resize_details) && state.dragging_item.is_none() {
                                     ui.cursor_icon = CursorIcon::ColResize;
                                 }

                                 if splitter_res.pressed && ui.clicked && state.dragging_item.is_none() {
                                     state.active_resize_details = true;
                                 }
                                if !ui.mouse_down {
                                    state.active_resize_details = false;
                                }
                                if state.active_resize_details {
                                    let window_w = ui.width as f32;
                                    state.details_width = (window_w - ui.mouse_x).clamp(200.0, 500.0);
                                    ui.request_redraw();
                                }

                                match state.active_screen {
                                    crate::state::ActiveScreen::Browser => {
                                        ui_layout::draw_preview_pane(ui, &mut state);
                                    }
                                    crate::state::ActiveScreen::Dashboard => {
                                        ui_layout::draw_dashboard_sidebar(ui, &mut state);
                                    }
                                }
                            }
                        });

                    // 4. Status Bar
                    ui_layout::draw_status_bar(ui, &mut state);

                    if state.pending_refresh.load(std::sync::atomic::Ordering::SeqCst) {
                        state.scan_current_dir();
                        state.pending_refresh.store(false, std::sync::atomic::Ordering::SeqCst);
                        ui.request_redraw();
                    }

                    if let Ok(mut lock) = state.zendrop_scan_results.lock() {
                        if let Some((devices, networks, name)) = lock.take() {
                            state.zendrop_devices = devices;
                            state.zendrop_networks = networks;
                            state.zendrop_network_name = name;
                            state.zendrop_scanning.store(false, std::sync::atomic::Ordering::SeqCst);
                            ui.request_redraw();
                        }
                    }

                    ui_layout::draw_about_window(ui, &mut state);
                    ui_layout::draw_context_menu(ui, &mut state);
                    ui_layout::draw_info_window(ui, &mut state);
                    ui_layout::draw_wifi_dialog(ui, &mut state);
                    ui_layout::draw_zendrop_pair_dialog(ui, &mut state);
                    ui_layout::draw_zendrop_send_dialog(ui, &mut state);

                    // 6. ZenDrop device panel (wifi icon)
                    if state.zendrop_open {
                        ui_layout::draw_zendrop_panel(ui, &mut state);
                    }

                    // 7. Notification / transfer progress panel (bell icon)
                    if state.zendrop_notif_open {
                        ui_layout::draw_notif_panel(ui, &mut state);
                    }
                });
        })
        .run();
}
