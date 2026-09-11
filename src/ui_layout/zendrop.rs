use crate::state::FileManagerState;
use zenthra::{Ui, FontWeight, Align, Id};

pub fn draw_zendrop_panel(ui: &mut Ui, state: &mut FileManagerState) {
    let colors = state.colors();

    // Scan if we just opened or it's been >5 seconds
    let should_scan = match state.last_zendrop_scan {
        None => true,
        Some(t) => t.elapsed().as_secs() >= 5,
    };
    if should_scan {
        state.refresh_zendrop();
    }

    let panel_w = 320.0_f32;
    let (panel_x, panel_y) = if let Some((btn_center_x, btn_bottom_y)) = state.zendrop_btn_pos {
        let x = (btn_center_x - panel_w / 2.0).clamp(12.0, ui.width as f32 - panel_w - 12.0);
        (x, btn_bottom_y)
    } else {
        let x = (ui.width as f32 - panel_w - 12.0).max(0.0);
        (x, 46.0_f32)
    };

    let panel_id = Id::from_u64(999999921);
    let resolved = super::common::resolve_widget_id(ui, panel_id);
    if let Some(rect) = ui.screen_layout_cache.get(&resolved) {
        state.wifi_panel_rect = Some((rect.origin.x, rect.origin.y, rect.size.width, rect.size.height));
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
        .shadow(zenthra::Color::rgba(0.0, 0.0, 0.0, 0.31), 0.0, 4.0, 24.0);

    if state.theme == crate::theme::ThemeMode::Glassmorphism {
        panel = panel
            .bg(colors.bg_panel.with_alpha(0.85))
            .backdrop_filter(zenthra::BackdropFilter::new().blur(16.0, zenthra::style::blur::Type::Glassmorphism));
    } else {
        panel = panel.bg(colors.bg_panel);
    }

    panel.show(|ui| {
        // Header
        ui.container()
            .fill_x()
            .padding(14.0, 14.0, 12.0, 14.0)
            .row()
            .gap(8.0)
            .valign(Align::Center)
            .show(|ui| {
                ui.text("\u{f1eb}")
                    .size(13.0)
                    .color(colors.accent)
                    .show();

                ui.container().column().gap(1.0).show(|ui| {
                    ui.text("ZenDrop")
                        .size(12.5)
                        .weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                    let is_scanning = state.zendrop_scanning.load(std::sync::atomic::Ordering::SeqCst);
                    let subtitle = if is_scanning {
                        "Scanning network...".to_string()
                    } else {
                        format!("Network: {}", state.zendrop_network_name)
                    };
                    ui.text(&subtitle)
                        .size(9.5)
                        .color(colors.accent)
                        .show();
                });

                ui.container().halign(Align::Right).show(|ui| {
                    let refresh_id = Id::from_u64(999_111_001);
                    let refresh_btn = ui.container()
                        .id(refresh_id)
                        .width(26.0)
                        .height(26.0)
                        .radius_all(5.0)
                        .hover_bg(colors.highlight)
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{f021}")
                                .size(11.0)
                                .color(colors.text_muted)
                                .show();
                        });
                    if refresh_btn.clicked {
                        state.refresh_zendrop();
                        ui.request_redraw();
                    }
                });
            });

        ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});

        // Local Device Name
        ui.container()
            .fill_x()
            .padding(10.0, 10.0, 6.0, 10.0)
            .column()
            .gap(4.0)
            .show(|ui| {
                ui.text("LOCAL DEVICE NAME")
                    .size(9.0)
                    .weight(FontWeight::Bold)
                    .color(colors.text_muted)
                    .show();
                
                let mut name_tmp = state.zendrop_device_name.lock().unwrap().clone();
                let prev_name = name_tmp.clone();
                
                ui.input(&mut name_tmp, "zendrop_local_device_name_input")
                    .fill_x()
                    .size(11.0)
                    .radius_all(6.0)
                    .border(colors.border, 1.0)
                    .bg(colors.bg_base)
                    .show();
                
                if name_tmp != prev_name && !name_tmp.trim().is_empty() {
                    *state.zendrop_device_name.lock().unwrap() = name_tmp.clone();
                    state.save_device_name(&name_tmp);
                }
            });

        ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});

        // ─── Available networks ───────────────────────────────────────────
        ui.container()
            .fill_x()
            .padding(10.0, 10.0, 4.0, 10.0)
            .show(|ui| {
                ui.text("AVAILABLE NETWORKS")
                    .size(9.0)
                    .weight(FontWeight::Bold)
                    .color(colors.text_muted)
                    .show();
            });


        let networks = state.zendrop_networks.clone();
        if networks.is_empty() {
            ui.container()
                .fill_x()
                .padding(12.0, 12.0, 12.0, 12.0)
                .show(|ui| {
                    ui.text("No Wi-Fi networks found")
                        .size(10.5)
                        .color(colors.text_dim)
                        .show();
                });
        } else {
            ui.container()
                .fill_x()
                .max_height(140.0)
                .scroll_y(true)
                .column()
                .padding(8.0, 8.0, 8.0, 8.0)
                .gap(4.0)
                .show(|ui| {
                    for (idx, net) in networks.iter().enumerate() {
                        let row_id = Id::from_u64(999_333_100 + idx as u64);
                        let is_hov = ui.interaction_state.get(&row_id).copied().unwrap_or(0.0) > 0.5;

                        let row = ui.container()
                            .id(row_id)
                            .fill_x()
                            .padding(6.0, 8.0, 6.0, 8.0)
                            .radius_all(6.0)
                            .row()
                            .gap(10.0)
                            .valign(Align::Center)
                            .bg(if is_hov { colors.highlight } else { zenthra::Color::TRANSPARENT })
                            .show(|ui| {
                                ui.text("\u{f1eb}")
                                    .size(11.0)
                                    .color(if net.active { colors.accent } else { colors.text_muted })
                                    .show();

                                ui.container().column().gap(1.0).show(|ui| {
                                    ui.text(&net.ssid)
                                        .size(10.5)
                                        .weight(FontWeight::Bold)
                                        .color(colors.text_primary)
                                        .show();
                                    ui.text(&format!("Signal: {}%", net.signal))
                                        .size(9.0)
                                        .color(colors.text_dim)
                                        .show();
                                });

                                // Connect / Disconnect button
                                ui.container().halign(Align::Right).show(|ui| {
                                    let btn_id = Id::from_u64(999_444_100 + idx as u64);
                                    let btn_text = if net.active { "Disconnect" } else { "Connect" };
                                    let btn_color = if net.active { zenthra::Color::rgb(220.0/255.0, 60.0/255.0, 60.0/255.0) } else { colors.accent };
                                    let btn = ui.button(btn_text)
                                        .id(btn_id)
                                        .width(76.0)
                                        .size(10.0)
                                        .radius_all(4.0)
                                        .bg(btn_color)
                                        .text_color(colors.bg_base)
                                        .padding(4.0, 0.0, 4.0, 0.0)
                                        .show();

                                    if btn.clicked {
                                        if net.active {
                                            state.disconnect_from_wifi(net.ssid.clone());
                                        } else {
                                            let is_secured = !net.security.is_empty() && !net.security.contains("OPN");
                                            if is_secured {
                                                state.wifi_connect_ssid = Some(net.ssid.clone());
                                                state.wifi_connect_password.clear();
                                                state.wifi_connect_error = None;
                                                state.wifi_connecting = false;
                                            } else {
                                                state.connect_to_wifi(net.ssid.clone(), None);
                                            }
                                        }
                                        ui.request_redraw();
                                    }
                                });
                            });

                        let h = row.hovered;
                        ui.interaction_state.insert(row_id, if h { 1.0 } else { 0.0 });
                    }
                });
        }

        // Divider
        ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});

        // ─── Share section ────────────────────────────────────────────────
        ui.container()
            .fill_x()
            .padding(10.0, 10.0, 4.0, 10.0)
            .show(|ui| {
                ui.text("SHARE")
                    .size(9.0)
                    .weight(FontWeight::Bold)
                    .color(colors.text_muted)
                    .show();
            });

        ui.container()
            .fill_x()
            .column()
            .padding(8.0, 8.0, 8.0, 8.0)
            .gap(6.0)
            .show(|ui| {
                // 1. Send Clipboard
                let clip_id = Id::from_u64(999_555_001);
                let clip_hov = ui.interaction_state.get(&clip_id).copied().unwrap_or(0.0) > 0.5;
                let clip_row = ui.container()
                    .id(clip_id)
                    .fill_x()
                    .padding(8.0, 8.0, 8.0, 8.0)
                    .radius_all(6.0)
                    .row()
                    .gap(10.0)
                    .valign(Align::Center)
                    .bg(if clip_hov { colors.highlight } else { zenthra::Color::TRANSPARENT })
                    .show(|ui| {
                        ui.text("\u{f0ea}")
                            .size(12.0)
                            .color(colors.text_muted)
                            .show();
                        ui.text("Send Clipboard")
                            .size(11.0)
                            .color(colors.text_primary)
                            .show();
                    });
                let clip_h = clip_row.hovered;
                ui.interaction_state.insert(clip_id, if clip_h { 1.0 } else { 0.0 });
                if clip_row.clicked {
                    state.zendrop_error = None;
                    state.zendrop_send_mode = crate::state::ZendropSendMode::Clipboard;
                    state.zendrop_show_devices = !state.zendrop_show_devices;
                    ui.request_redraw();
                }

                // 2. Share File
                let file_id = Id::from_u64(999_555_002);
                let file_hov = ui.interaction_state.get(&file_id).copied().unwrap_or(0.0) > 0.5;
                let file_row = ui.container()
                    .id(file_id)
                    .fill_x()
                    .padding(8.0, 8.0, 8.0, 8.0)
                    .radius_all(6.0)
                    .row()
                    .gap(10.0)
                    .valign(Align::Center)
                    .bg(if file_hov { colors.highlight } else { zenthra::Color::TRANSPARENT })
                    .show(|ui| {
                        ui.text("\u{f016}")
                            .size(12.0)
                            .color(colors.text_muted)
                            .show();
                        ui.text("Share File")
                            .size(11.0)
                            .color(colors.text_primary)
                            .show();
                    });
                let file_h = file_row.hovered;
                ui.interaction_state.insert(file_id, if file_h { 1.0 } else { 0.0 });
                if file_row.clicked {
                    if state.selected_paths.is_empty() {
                        state.zendrop_error = Some("Please select a file or folder.".to_string());
                        state.zendrop_show_devices = false;
                    } else {
                        state.zendrop_error = None;
                        state.zendrop_send_mode = crate::state::ZendropSendMode::Files;
                        state.zendrop_show_devices = !state.zendrop_show_devices;
                    }
                    ui.request_redraw();
                }

                // 3. Others (label/disabled option)
                let others_id = Id::from_u64(999_555_003);
                ui.container()
                    .id(others_id)
                    .fill_x()
                    .padding(8.0, 8.0, 8.0, 8.0)
                    .radius_all(6.0)
                    .row()
                    .gap(10.0)
                    .valign(Align::Center)
                    .show(|ui| {
                        ui.text("\u{f0c9}")
                            .size(12.0)
                            .color(colors.text_dim)
                            .show();
                        ui.text("Others Option (Label)")
                            .size(11.0)
                            .color(colors.text_dim)
                            .show();
                    });

                // Display selection warning/error if present
                if let Some(ref err) = state.zendrop_error {
                    ui.spacing(4.0);
                    ui.text(err)
                        .size(10.0)
                        .color(zenthra::Color::rgb(220.0/255.0, 60.0/255.0, 60.0/255.0))
                        .show();
                }
            });

        // ─── Paired devices list (if toggled/open) ────────────────────────
        if state.zendrop_show_devices {
            ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
            ui.container()
                .fill_x()
                .padding(10.0, 8.0, 2.0, 10.0)
                .show(|ui| {
                    ui.text("PAIRED DEVICES (TAP TO SEND)")
                        .size(8.5)
                        .weight(FontWeight::Bold)
                        .color(colors.accent)
                        .show();
                });

            let paired = state.zendrop_paired.clone();
            if paired.is_empty() {
                ui.container()
                    .fill_x()
                    .padding(12.0, 12.0, 12.0, 12.0)
                    .show(|ui| {
                        ui.text("No paired devices found. Pair a device below.")
                            .size(10.0)
                            .color(colors.text_dim)
                            .show();
                    });
            } else {
                ui.container()
                    .fill_x()
                    .max_height(140.0)
                    .scroll_y(true)
                    .column()
                    .padding(6.0, 6.0, 6.0, 6.0)
                    .gap(4.0)
                    .show(|ui| {
                        for (idx, (name, ip, _)) in paired.iter().enumerate() {
                            let forget_btn_id = Id::from_u64(999_888_100 + idx as u64);
                            
                            ui.container()
                                .fill_x()
                                .row()
                                .gap(4.0)
                                .valign(Align::Center)
                                .show(|ui| {
                                    let dev_row_id = Id::from_u64(999_666_200 + idx as u64);
                                    let is_hov = ui.interaction_state.get(&dev_row_id).copied().unwrap_or(0.0) > 0.5;

                                    let dev_row = ui.container()
                                        .id(dev_row_id)
                                        .width(260.0)
                                        .padding(6.0, 6.0, 6.0, 6.0)
                                        .radius_all(6.0)
                                        .row()
                                        .gap(8.0)
                                        .valign(Align::Center)
                                        .bg(if is_hov { colors.highlight } else { zenthra::Color::TRANSPARENT })
                                        .show(|ui| {
                                            ui.text("\u{f10b}") // phone icon
                                                .size(12.0)
                                                .color(colors.accent)
                                                .show();
                                            ui.container().column().gap(1.0).show(|ui| {
                                                ui.text(name)
                                                    .size(10.5)
                                                    .weight(FontWeight::Bold)
                                                    .color(colors.text_primary)
                                                    .show();
                                                ui.text(ip)
                                                    .size(9.0)
                                                    .color(colors.text_dim)
                                                    .show();
                                            });
                                        });

                                    let h = dev_row.hovered;
                                    ui.interaction_state.insert(dev_row_id, if h { 1.0 } else { 0.0 });

                                    if dev_row.clicked {
                                        let device = crate::state::ZendropDevice {
                                            ip: ip.clone(),
                                            mac: "".to_string(),
                                            hostname: name.clone(),
                                            iface: "".to_string(),
                                        };
                                        state.start_zendrop_send(device);
                                        ui.request_redraw();
                                    }

                                    // Forget/Delete button
                                    let is_forget_hov = ui.interaction_state.get(&forget_btn_id).copied().unwrap_or(0.0) > 0.5;
                                    let forget_btn = ui.container()
                                        .id(forget_btn_id)
                                        .width(28.0)
                                        .height(28.0)
                                        .radius_all(4.0)
                                        .bg(if is_forget_hov { zenthra::Color::rgba(220.0/255.0, 60.0/255.0, 60.0/255.0, 0.15) } else { zenthra::Color::TRANSPARENT })
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text("\u{f1f8}") // trash icon (fa-trash-o)
                                                .size(10.0)
                                                .color(if is_forget_hov { zenthra::Color::rgb(220.0/255.0, 60.0/255.0, 60.0/255.0) } else { colors.text_muted })
                                                .show();
                                        });

                                    let fh = forget_btn.hovered;
                                    ui.interaction_state.insert(forget_btn_id, if fh { 1.0 } else { 0.0 });

                                    if forget_btn.clicked {
                                        state.forget_paired_device(ip.clone());
                                        ui.request_redraw();
                                    }
                                });
                        }
                    });
            }

            // ─── Scanned Devices / Discovery Section ────────────────────────
            ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
            ui.container()
                .fill_x()
                .padding(10.0, 10.0, 4.0, 10.0)
                .show(|ui| {
                    ui.text("DISCOVERED DEVICED (TAP TO PAIR)")
                        .size(8.5)
                        .weight(FontWeight::Bold)
                        .color(colors.accent)
                        .show();
                });

            let scanned_list = {
                let scanned = state.zendrop_scanned.lock().unwrap();
                scanned.clone()
            };
            
            // Filter out already paired devices
            let unpaired_scanned: Vec<_> = scanned_list.into_iter()
                .filter(|(_name, ip, _)| {
                    !state.zendrop_paired.iter().any(|(_, paired_ip, _)| paired_ip == ip)
                })
                .collect();

            if unpaired_scanned.is_empty() {
                ui.container()
                    .fill_x()
                    .padding(12.0, 12.0, 12.0, 12.0)
                    .show(|ui| {
                        ui.text("Scanning local network for devices...")
                            .size(10.0)
                            .color(colors.text_dim)
                            .show();
                    });
            } else {
                ui.container()
                    .fill_x()
                    .max_height(140.0)
                    .scroll_y(true)
                    .column()
                    .padding(6.0, 6.0, 6.0, 6.0)
                    .gap(4.0)
                    .show(|ui| {
                        for (idx, (name, ip, _)) in unpaired_scanned.iter().enumerate() {
                            let scan_row_id = Id::from_u64(999_777_100 + idx as u64);
                            let is_hov = ui.interaction_state.get(&scan_row_id).copied().unwrap_or(0.0) > 0.5;

                            let scan_row = ui.container()
                                .id(scan_row_id)
                                .fill_x()
                                .padding(8.0, 8.0, 8.0, 8.0)
                                .radius_all(6.0)
                                .row()
                                .gap(8.0)
                                .valign(Align::Center)
                                .bg(if is_hov { colors.highlight } else { zenthra::Color::TRANSPARENT })
                                .show(|ui| {
                                    ui.text("\u{f10b}") // phone icon
                                        .size(12.0)
                                        .color(colors.text_muted)
                                        .show();
                                    ui.container().column().gap(1.0).show(|ui| {
                                        ui.text(name)
                                            .size(10.5)
                                            .weight(FontWeight::Bold)
                                            .color(colors.text_primary)
                                            .show();
                                        ui.text(ip)
                                            .size(9.0)
                                            .color(colors.text_dim)
                                            .show();
                                    });
                                });

                            let h = scan_row.hovered;
                            ui.interaction_state.insert(scan_row_id, if h { 1.0 } else { 0.0 });

                            if scan_row.clicked {
                                state.pair_device(ip.clone());
                                ui.request_redraw();
                            }
                        }
                    });
            }

            // Show status msg during pairing
            let msg = {
                let m = state.zendrop_status_msg.lock().unwrap();
                m.clone()
            };
            if let Some(m) = msg {
                ui.container()
                    .fill_x()
                    .padding(10.0, 4.0, 10.0, 10.0)
                    .show(|ui| {
                        ui.text(&m)
                            .size(9.0)
                            .color(colors.accent)
                            .show();
                    });
            }
        }
    });
}

#[allow(dead_code)]
fn device_icon(hostname: &str, iface: &str) -> &'static str {
    let h = hostname.to_lowercase();
    if h.contains("router") || h.contains("gateway") || h.contains("modem") {
        "\u{f6ff}"
    } else if h.contains("phone") || h.contains("android") || h.contains("iphone") {
        "\u{f10b}"
    } else if h.contains("tv") || h.contains("chromecast") {
        "\u{f26c}"
    } else if h.contains("mac") || h.contains("apple") {
        "\u{f179}"
    } else if h.contains("pi") || h.contains("raspberry") {
        "\u{f315}"
    } else if iface.starts_with("wlan") || iface.starts_with("wlp") {
        "\u{f1eb}"
    } else {
        "\u{f108}"
    }
}
