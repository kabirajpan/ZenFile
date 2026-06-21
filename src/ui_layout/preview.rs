use crate::state::{FileManagerState, format_size};
use super::common::{
    get_item_icon_path, open_file, truncate_filename, truncate_str,
    NF_FA_EXTERNAL_LINK, NF_FA_EDIT, NF_FA_TRASH,
};
use zenthra::{Color, ImageSource, ObjectFit, Ui, FontWeight, Align};

pub fn draw_preview_pane(ui: &mut Ui, state: &mut FileManagerState) {
    let colors = state.colors();

    ui.container()
        .width(state.details_width)
        .fill_y()
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .padding(15.0, 15.0, 15.0, 15.0)
        .column()
        .show(|ui| {
            // Check if there is a valid selected item
            let selected_item = state.selected_idx.and_then(|idx| state.items.get(idx).cloned());

            match selected_item {
                None => {
                    ui.container()
                        .fill()
                        .align(zenthra::Align::Center)
                        .show(|ui| {
                            ui.text("Select a file or folder\nto see details")
                                .color(colors.text_dim)
                                .size(11.5)
                                .show();
                        });
                }
                Some(item) => {
                    // Show item details
                    ui.text("SELECTION DETAILS")
                        .size(9.5)
                        .weight(FontWeight::Bold)
                        .color(colors.text_muted)
                        .show();

                    ui.spacing(12.0);

                    // Render Preview Thumbnail or Icon Box
                    ui.container()
                        .fill_x()
                        .height(130.0)
                        .bg(colors.bg_panel)
                        .border(colors.border, 1.0)
                        .radius_all(8.0)
                        .align(zenthra::Align::Center)
                        .clip(true)
                        .show(|ui| {
                            if !item.is_dir && item.category == "image" {
                                ui.image(ImageSource::Path(item.path.clone()))
                                    .size(250.0, 120.0)
                                    .fit(ObjectFit::Contain)
                                    .show();
                            } else {
                                let icon_path = get_item_icon_path(&state.icon_theme, &item.category, &item.extension);
                                ui.image(ImageSource::Path(icon_path))
                                    .size(48.0, 48.0)
                                    .fit(ObjectFit::Contain)
                                    .show();
                            }
                        });

                    ui.spacing(15.0);

                    // Name, Type, and Path responsive sizing
                    let avail_width = state.details_width - 30.0; // padding 15 left/right
                    let max_chars_name = ((avail_width - 20.0) / 7.5).max(12.0) as usize;
                    let max_chars_type = ((avail_width - 20.0) / 6.5).max(12.0) as usize;
                    let max_chars_path = ((avail_width - 90.0) / 6.5).max(12.0) as usize;

                    let name_disp = truncate_filename(&item.name, max_chars_name);
                    ui.text(&name_disp)
                        .size(13.0)
                        .weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();

                    ui.spacing(5.0);
                    
                    let display_type_disp = truncate_str(&item.display_type, max_chars_type);
                    ui.text(&display_type_disp)
                        .size(11.0)
                        .color(colors.text_muted)
                        .show();

                    ui.spacing(15.0);

                    // Action buttons or Rename/Delete dialog states
                    if let Some(del_path) = &state.delete_confirm {
                        if del_path == &item.path {
                            let is_narrow_dlg = state.details_width < 260.0;
                            ui.container()
                                .fill_x()
                                .bg(Color::rgb(45.0 / 255.0, 15.0 / 255.0, 15.0 / 255.0))
                                .border(Color::rgb(220.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0), 1.0)
                                .radius_all(6.0)
                                .padding(10.0, 10.0, 10.0, 10.0)
                                .column()
                                .show(|ui| {
                                    ui.text("Delete this item permanently?")
                                        .size(11.0)
                                        .color(colors.text_primary)
                                        .show();
                                    ui.spacing(10.0);
                                    
                                    let btn_layout = ui.container().gap(8.0);
                                    let btn_layout = if is_narrow_dlg {
                                        btn_layout.column().fill_x()
                                    } else {
                                        btn_layout.row()
                                    };
                                    
                                    btn_layout.show(|ui| {
                                        let yes_btn = ui.button("Delete")
                                            .bg(Color::rgb(220.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0))
                                            .text_color(Color::WHITE)
                                            .padding(4.0, 10.0, 4.0, 10.0)
                                            .radius_all(4.0)
                                            .size(11.0);
                                        let yes = if is_narrow_dlg { yes_btn.fill_x() } else { yes_btn }.show();
                                        
                                        if yes.clicked {
                                            let res = if item.is_dir {
                                                std::fs::remove_dir_all(&item.path)
                                            } else {
                                                std::fs::remove_file(&item.path)
                                            };
                                            if res.is_ok() {
                                                state.delete_confirm = None;
                                                state.scan_current_dir();
                                            }
                                            ui.request_redraw();
                                        }

                                        let no_btn = ui.button("Cancel")
                                            .bg(colors.bg_panel)
                                            .text_color(colors.text_primary)
                                            .padding(4.0, 10.0, 4.0, 10.0)
                                            .radius_all(4.0)
                                            .size(11.0);
                                        let no = if is_narrow_dlg { no_btn.fill_x() } else { no_btn }.show();
                                        
                                        if no.clicked {
                                            state.delete_confirm = None;
                                            ui.request_redraw();
                                        }
                                    });
                                });
                            return;
                        }
                    }

                    // Metadata details
                    ui.container()
                        .fill_x()
                        .column()
                        .gap(6.0)
                        .show(|ui| {
                            let draw_detail = |ui: &mut Ui, label: &str, value: &str| {
                                ui.container().row().fill_x().show(|ui| {
                                    ui.container().width(80.0).show(|ui| {
                                        ui.text(label).size(10.5).color(colors.text_muted).show();
                                    });
                                    ui.text(value).size(10.5).color(colors.text_primary).show();
                                });
                            };

                            let size_val = if item.is_dir { "Folder" } else { &format_size(item.size) };
                            draw_detail(ui, "Size:", size_val);
                            
                            let path_disp = truncate_filename(&item.path.to_string_lossy(), max_chars_path);
                            ui.container().row().fill_x().valign(Align::Center).show(|ui| {
                                ui.container().width(80.0).show(|ui| {
                                    ui.text("Location:").size(10.5).color(colors.text_muted).show();
                                });
                                ui.text(&path_disp).size(10.5).color(colors.text_primary).show();
                                ui.spacing(4.0);
                                let copy_btn = ui.button("\u{f0c5}") // NF_FA_COPY
                                    .width(18.0)
                                    .size(9.5)
                                    .bg(Color::TRANSPARENT)
                                    .hover_bg(colors.highlight)
                                    .text_color(colors.text_muted)
                                    .radius_all(3.0)
                                    .padding(2.0, 0.0, 2.0, 0.0)
                                    .show();
                                if copy_btn.clicked {
                                    state.copy_path_to_clipboard(&item.path.to_string_lossy());
                                }
                            });
                        });

                    ui.spacing(15.0);

                    // Text snippet preview panel
                    if let Some(snippet) = &state.text_preview {
                        ui.container()
                            .fill_x()
                            .height(120.0)
                            .bg(colors.bg_panel)
                            .border(colors.border, 1.0)
                            .radius_all(6.0)
                            .padding(8.0, 8.0, 8.0, 8.0)
                            .scrollable(true, true)
                            .clip(true)
                            .show(|ui| {
                                ui.text(snippet)
                                    .size(10.0)
                                    .color(colors.text_muted)
                                    .show();
                            });
                        ui.spacing(15.0);
                    }

                    // Bottom Control Buttons
                    let is_narrow = state.details_width < 290.0;
                    
                    let btn_container = ui.container().gap(10.0);
                    let btn_container = if is_narrow {
                        btn_container.column().fill_x()
                    } else {
                        btn_container.row()
                    };
                    
                    btn_container.show(|ui| {
                        // Open Button
                        let open_btn_c = ui.container()
                            .row()
                            .gap(6.0)
                            .valign(Align::Center)
                            .align(Align::Center)
                            .bg(colors.bg_panel)
                            .hover_bg(colors.highlight)
                            .radius_all(6.0)
                            .padding(6.0, 12.0, 6.0, 12.0);
                        let open_btn_c = if is_narrow { open_btn_c.fill_x() } else { open_btn_c };
                        let open_btn = open_btn_c.show(|ui| {
                            ui.text(NF_FA_EXTERNAL_LINK).size(11.0).color(colors.text_primary).show();
                            ui.text("Open").size(11.0).color(colors.text_primary).show();
                        });
                        if open_btn.clicked {
                            open_file(&item.path);
                        }

                        // Rename Button
                        let rename_btn_c = ui.container()
                            .row()
                            .gap(6.0)
                            .valign(Align::Center)
                            .align(Align::Center)
                            .bg(colors.bg_panel)
                            .hover_bg(colors.highlight)
                            .radius_all(6.0)
                            .padding(6.0, 12.0, 6.0, 12.0);
                        let rename_btn_c = if is_narrow { rename_btn_c.fill_x() } else { rename_btn_c };
                        let rename_btn = rename_btn_c.show(|ui| {
                            ui.text(NF_FA_EDIT).size(11.0).color(colors.text_primary).show();
                            ui.text("Rename").size(11.0).color(colors.text_primary).show();
                        });
                        if rename_btn.clicked {
                            super::common::start_rename(ui, state, item.path.clone(), item.name.clone());
                            ui.request_redraw();
                        }

                        // Delete Button
                        let delete_btn_c = ui.container()
                            .row()
                            .gap(6.0)
                            .valign(Align::Center)
                            .align(Align::Center)
                            .bg(colors.bg_panel)
                            .hover_bg(Color::rgb(220.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0))
                            .radius_all(6.0)
                            .padding(6.0, 12.0, 6.0, 12.0);
                        let delete_btn_c = if is_narrow { delete_btn_c.fill_x() } else { delete_btn_c };
                        let delete_btn = delete_btn_c.show(|ui| {
                            ui.text(NF_FA_TRASH).size(11.0).color(colors.text_primary).show();
                            ui.text("Delete").size(11.0).color(colors.text_primary).show();
                        });
                        if delete_btn.clicked {
                            state.delete_confirm = Some(item.path.clone());
                            ui.request_redraw();
                        }
                    });
                }
            }
        });
}
