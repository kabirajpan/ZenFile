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
            // Expose the active theme flag to interaction_state so widgets can read it
            let theme_val = if state.theme == ThemeMode::Light { 1.0 } else { 0.0 };
            ui.interaction_state.insert(Id::from_u64(999999999), theme_val);

            let glass_val = if state.theme == ThemeMode::Glassmorphism { 1.0 } else { 0.0 };
            ui.interaction_state.insert(Id::from_u64(999999998), glass_val);

            let colors = state.colors();

            // Expose active theme colors to widgets
            ui.interaction_state.insert(Id::from_u64(999999980), colors.accent.r);
            ui.interaction_state.insert(Id::from_u64(999999981), colors.accent.g);
            ui.interaction_state.insert(Id::from_u64(999999982), colors.accent.b);
            ui.interaction_state.insert(Id::from_u64(999999983), colors.accent.a);

            ui.interaction_state.insert(Id::from_u64(999999970), colors.highlight.r);
            ui.interaction_state.insert(Id::from_u64(999999971), colors.highlight.g);
            ui.interaction_state.insert(Id::from_u64(999999972), colors.highlight.b);
            ui.interaction_state.insert(Id::from_u64(999999973), colors.highlight.a);

            ui.interaction_state.insert(Id::from_u64(999999960), colors.text_primary.r);
            ui.interaction_state.insert(Id::from_u64(999999961), colors.text_primary.g);
            ui.interaction_state.insert(Id::from_u64(999999962), colors.text_primary.b);
            ui.interaction_state.insert(Id::from_u64(999999963), colors.text_primary.a);

            ui.interaction_state.insert(Id::from_u64(999999950), colors.text_muted.r);
            ui.interaction_state.insert(Id::from_u64(999999951), colors.text_muted.g);
            ui.interaction_state.insert(Id::from_u64(999999952), colors.text_muted.b);
            ui.interaction_state.insert(Id::from_u64(999999953), colors.text_muted.a);

            ui.interaction_state.insert(Id::from_u64(999999940), colors.bg_panel.r);
            ui.interaction_state.insert(Id::from_u64(999999941), colors.bg_panel.g);
            ui.interaction_state.insert(Id::from_u64(999999942), colors.bg_panel.b);
            ui.interaction_state.insert(Id::from_u64(999999943), colors.bg_panel.a);

            ui.interaction_state.insert(Id::from_u64(999999930), colors.border.r);
            ui.interaction_state.insert(Id::from_u64(999999931), colors.border.g);
            ui.interaction_state.insert(Id::from_u64(999999932), colors.border.b);
            ui.interaction_state.insert(Id::from_u64(999999933), colors.border.a);

            // Main Background Container (lays out Title Bar, Navigation, Content, and Status Bar vertically)
            ui.container()
                .fill()
                .bg(colors.bg_base)
                .show(|ui| {
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

                            // Main file grid list
                            ui_layout::draw_file_list(ui, &mut state, file_list_w);

                            // Right preview details pane
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
