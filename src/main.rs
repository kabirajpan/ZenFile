mod state;
mod theme;
mod ui_layout;

use state::FileManagerState;
use theme::ThemeMode;
use zenthra::prelude::*;
use zenthra::Id;

fn main() {
    env_logger::init();

    // Initialize the modular state
    let mut state = FileManagerState::new();

    App::new()
        .title("ZenFile")
        .size(1100, 680)
        .decorations(false)
        .load_font_path("../../assets/fonts/SymbolsNerdFont-Regular.ttf")
        .with_ui(move |ui| {
            // Expose the active theme flag to interaction_state so widgets can read it
            let theme_val = if state.theme == ThemeMode::Light { 1.0 } else { 0.0 };
            ui.interaction_state.insert(Id::from_u64(999999999), theme_val);

            let colors = state.colors();

            // Main Background Container (lays out Title Bar, Navigation, Content, and Status Bar vertically)
            ui.container()
                .fill()
                .bg(colors.bg_base)
                .show(|ui| {
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
                                    .width(4.0)
                                    .fill_y()
                                    .bg(colors.border)
                                    .hover_bg(colors.highlight)
                                    .show(|_| {});

                                if (splitter_res.hovered || state.active_resize_sidebar) && state.dragging_item.is_none() {
                                    ui.cursor_icon = CursorIcon::ColResize;
                                }

                                if splitter_res.pressed && state.drag_select_start.is_none() && state.dragging_item.is_none() {
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

                            // Main file grid list
                            ui_layout::draw_file_list(ui, &mut state, file_list_w);

                            // Right preview details pane
                            if state.details_visible {
                                // Details Splitter handle
                                let splitter_res = ui.container()
                                    .width(4.0)
                                    .fill_y()
                                    .bg(colors.border)
                                    .hover_bg(colors.highlight)
                                    .show(|_| {});

                                if (splitter_res.hovered || state.active_resize_details) && state.dragging_item.is_none() {
                                    ui.cursor_icon = CursorIcon::ColResize;
                                }

                                if splitter_res.pressed && state.drag_select_start.is_none() && state.dragging_item.is_none() {
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

                                ui_layout::draw_preview_pane(ui, &mut state);
                            }
                        });

                    // 4. Status Bar
                    ui_layout::draw_status_bar(ui, &mut state);

                    // 5. Floating dialogs (e.g. About window)
                    ui_layout::draw_about_window(ui, &mut state);
                    ui_layout::draw_context_menu(ui, &mut state);
                    ui_layout::draw_info_window(ui, &mut state);
                });
        })
        .run();
}
