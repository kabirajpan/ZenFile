use crate::state::FileManagerState;
use super::common::truncate_filename;
use zenthra::{Ui, Align};

pub fn draw_status_bar(ui: &mut Ui, state: &mut FileManagerState) {
    let colors = state.colors();
    let items_count = state.items.len();
    let folders_count = state.items.iter().filter(|i| i.is_dir).count();
    let files_count = items_count.saturating_sub(folders_count);

    ui.container()
        .fill_x()
        .height(24.0)
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .padding(4.0, 15.0, 4.0, 15.0)
        .row()
        .valign(Align::Center)
        .show(|ui| {
            let status_text = format!("{} items | {} folders, {} files", items_count, folders_count, files_count);
            ui.text(&status_text)
                .size(10.0)
                .color(colors.text_muted)
                .show();

            // Right alignment for system/context info
            ui.container()
                .fill_x()
                .align(zenthra::Align::Right)
                .show(|ui| {
                    let path_text = state.current_dir.to_string_lossy().to_string();
                    let truncated_path = truncate_filename(&path_text, 50);
                    ui.text(&truncated_path)
                        .size(10.0)
                        .color(colors.text_dim)
                        .show();
                });
        });
}
