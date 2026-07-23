use crate::state::{FileManagerState, TransferStatus};
use zenthra::{Color, Ui, FontWeight, Align, Id};

pub fn draw_notif_panel(ui: &mut Ui, state: &mut FileManagerState) {
    let colors = state.colors();

    let panel_w = 340.0_f32;
    let (panel_x, panel_y) = if let Some((btn_center_x, btn_bottom_y)) = state.zendrop_notif_btn_pos {
        let x = (btn_center_x - panel_w / 2.0).clamp(12.0, ui.width as f32 - panel_w - 12.0);
        (x, btn_bottom_y)
    } else {
        let x = (ui.width as f32 - panel_w - 12.0).max(0.0);
        (x, 46.0_f32)
    };

    let panel_id = Id::from_u64(999999922);
    let resolved = super::common::resolve_widget_id(ui, panel_id);
    if let Some(rect) = ui.screen_layout_cache.get(&resolved) {
        state.bell_panel_rect = Some((rect.origin.x, rect.origin.y, rect.size.width, rect.size.height));
    }

    let mut panel = ui.container()
        .id(panel_id)
        .absolute(panel_x, panel_y)
        .overlay()
        .width(panel_w)
        .column()
        .padding(0.0, 0.0, 0.0, 0.0)
        .radius_all(10.0)
        .border(colors.border, 1.0)
        .shadow(Color::rgba(0.0, 0.0, 0.0, 0.35), 0.0, 4.0, 24.0);

    if state.theme == crate::theme::ThemeMode::Glassmorphism {
        panel = panel
            .bg(colors.bg_panel.with_alpha(0.92))
            .backdrop_filter(zenthra::BackdropFilter::new().blur(20.0, zenthra::style::blur::Type::Glassmorphism));
    } else {
        panel = panel.bg(colors.bg_panel);
    }

    panel.show(|ui| {
        // ── Header ──────────────────────────────────────────────────────────
        ui.container()
            .fill_x()
            .padding(14.0, 12.0, 12.0, 14.0)
            .row()
            .gap(8.0)
            .valign(Align::Center)
            .show(|ui| {
                ui.text("\u{f0f3}") // bell
                    .size(12.0)
                    .color(colors.accent)
                    .show();

                ui.text("Transfers")
                    .size(12.5)
                    .weight(FontWeight::Bold)
                    .color(colors.text_primary)
                    .show();

                // Clear all button (right-aligned)
                ui.container().halign(Align::Right).show(|ui| {
                    let clear_btn = ui.button("Clear All")
                        .size(9.5)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(colors.highlight)
                        .text_color(colors.text_muted)
                        .radius_all(4.0)
                        .padding(6.0, 4.0, 6.0, 4.0)
                        .show();
                    if clear_btn.clicked {
                        // Remove only completed/failed entries
                        state.zendrop_transfers.retain(|e| {
                            *e.status.lock().unwrap() == TransferStatus::Sending
                        });
                        ui.request_redraw();
                    }
                });
            });

        ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});

        // ── Transfer list ────────────────────────────────────────────────────
        let transfers = state.zendrop_transfers.clone();

        if transfers.is_empty() {
            ui.container()
                .fill_x()
                .padding(20.0, 24.0, 20.0, 20.0)
                .column()
                .gap(6.0)
                .halign(Align::Center)
                .show(|ui| {
                    ui.text("\u{f00c}")
                        .size(20.0)
                        .color(colors.text_dim)
                        .show();
                    ui.text("No transfers yet")
                        .size(11.0)
                        .color(colors.text_dim)
                        .show();
                });
        } else {
            ui.container()
                .fill_x()
                .max_height(400.0)
                .scroll_y(true)
                .column()
                .show(|ui| {
                    for (idx, entry) in transfers.iter().enumerate() {
                        let sent = entry.sent_bytes.load(std::sync::atomic::Ordering::Relaxed);
                        let total = entry.total_bytes.max(1);
                        let fill = (sent as f32 / total as f32).min(1.0);
                        let pct = (fill * 100.0) as u32;
                        let status = entry.status.lock().unwrap().clone();
                        let speed = *entry.speed_mbps.lock().unwrap();

                        let is_active = status == TransferStatus::Sending;
                        let is_done   = status == TransferStatus::Done;
                        let is_failed = status == TransferStatus::Failed;

                        if is_active { ui.needs_redraw = true; }

                        // Separator between rows
                        if idx > 0 {
                            ui.container().fill_x().height(1.0).bg(colors.border.with_alpha(0.5)).show(|_| {});
                        }

                        ui.container()
                            .fill_x()
                            .padding(14.0, 10.0, 14.0, 14.0)
                            .column()
                            .gap(6.0)
                            .show(|ui| {
                                // ── Row 1: icon + filename + status badge ──
                                ui.container().fill_x().row().gap(8.0).valign(Align::Center).show(|ui| {
                                    let (icon, icon_color) = if is_done {
                                        ("\u{f00c}", Color::rgb(0.35, 0.82, 0.45))
                                    } else if is_failed {
                                        ("\u{f00d}", Color::rgb(0.9, 0.3, 0.3))
                                    } else {
                                        ("\u{f382}", colors.accent) // cloud upload arrow
                                    };
                                    ui.text(icon).size(12.0).color(icon_color).show();

                                    // Filename
                                    let name = if entry.filename.len() > 28 {
                                        format!("{}…", &entry.filename[..26])
                                    } else {
                                        entry.filename.clone()
                                    };
                                    ui.text(&name)
                                        .size(11.0)
                                        .weight(FontWeight::Bold)
                                        .color(colors.text_primary)
                                        .show();

                                    // Status badge (right)
                                    ui.container().halign(Align::Right).show(|ui| {
                                        if is_done {
                                            ui.text("Done")
                                                .size(9.0)
                                                .color(Color::rgb(0.35, 0.82, 0.45))
                                                .show();
                                        } else if is_failed {
                                            ui.text("Cancelled")
                                                .size(9.0)
                                                .color(Color::rgb(0.9, 0.3, 0.3))
                                                .show();
                                        } else {
                                            ui.text(&format!("{}%", pct))
                                                .size(9.5)
                                                .weight(FontWeight::Bold)
                                                .color(colors.accent)
                                                .show();
                                        }
                                    });
                                });

                                // ── Row 2: progress bar ──
                                ui.container()
                                    .fill_x()
                                    .height(5.0)
                                    .radius_all(2.5)
                                    .bg(colors.border)
                                    .show(|ui| {
                                        let bar_w = fill * ui.available_width;
                                        let bar_color = if is_done { Color::rgb(0.35, 0.82, 0.45) }
                                            else if is_failed { Color::rgb(0.9, 0.3, 0.3) }
                                            else { colors.accent };
                                        if bar_w > 0.0 {
                                            ui.container()
                                                .width(bar_w)
                                                .fill_y()
                                                .radius_all(2.5)
                                                .bg(bar_color)
                                                .show(|_| {});
                                        }
                                    });

                                // ── Row 3: detail stats + cancel button ──
                                ui.container().fill_x().row().gap(8.0).valign(Align::Center).show(|ui| {
                                    if is_active {
                                        // Bytes transferred / total
                                        let sent_mb = sent as f64 / 1_048_576.0;
                                        let total_mb = total as f64 / 1_048_576.0;
                                        let remaining_mb = (total_mb - sent_mb).max(0.0);
                                        ui.text(&format!("{:.1} / {:.1} MB  ·  {:.1} MB left",
                                            sent_mb, total_mb, remaining_mb))
                                            .size(9.0)
                                            .color(colors.text_muted)
                                            .show();
                                    } else if is_done {
                                        let total_mb = total as f64 / 1_048_576.0;
                                        if speed > 0.0 {
                                            ui.text(&format!("{:.1} MB  ·  {:.1} MB/s", total_mb, speed))
                                                .size(9.0)
                                                .color(colors.text_muted)
                                                .show();
                                        } else {
                                            ui.text(&format!("{:.1} MB  ·  Completed", total_mb))
                                                .size(9.0)
                                                .color(colors.text_muted)
                                                .show();
                                        }
                                    } else {
                                        ui.text("Transfer cancelled")
                                            .size(9.0)
                                            .color(colors.text_dim)
                                            .show();
                                    }

                                    // Cancel button (only for active)
                                    if is_active {
                                        ui.container().halign(Align::Right).show(|ui| {
                                            let cancel_btn = ui.button("Cancel")
                                                .size(9.0)
                                                .bg(Color::rgba(0.9, 0.3, 0.3, 0.15))
                                                .hover_bg(Color::rgba(0.9, 0.3, 0.3, 0.3))
                                                .text_color(Color::rgb(0.9, 0.3, 0.3))
                                                .radius_all(4.0)
                                                .padding(6.0, 3.0, 6.0, 3.0)
                                                .show();
                                            if cancel_btn.clicked {
                                                // Signal the thread to stop
                                                if let Some(e) = state.zendrop_transfers.get(idx) {
                                                    e.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                                                }
                                                ui.request_redraw();
                                            }
                                        });
                                    }
                                });
                            });
                    }
                });
        }

        ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});

        // ── Footer: close panel ───────────────────────────────────────────
        ui.container()
            .fill_x()
            .padding(10.0, 8.0, 10.0, 10.0)
            .row()
            .halign(Align::Right)
            .show(|ui| {
                let close_btn = ui.button("Close")
                    .size(10.0)
                    .bg(Color::TRANSPARENT)
                    .hover_bg(colors.highlight)
                    .text_color(colors.text_muted)
                    .radius_all(4.0)
                    .padding(8.0, 4.0, 8.0, 4.0)
                    .show();
                if close_btn.clicked {
                    state.zendrop_notif_open = false;
                    ui.request_redraw();
                }
            });
    });
}
