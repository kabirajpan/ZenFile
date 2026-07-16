use crate::state::FileManagerState;
use zenthra::{Ui, Align, FontWeight, Id};

pub fn draw_dashboard(ui: &mut Ui, state: &mut FileManagerState, width: f32) {
    let colors = state.colors();

    // Redraw every 2 seconds to update CPU/RAM loads dynamically when the dashboard is open
    ui.request_redraw_after(std::time::Duration::from_millis(2000));

    let list_container_id = Id::from_u64(888888889);
    
    // Track dashboard container with the same ID as the list container for coordinate mapping
    let mut list_container = ui.container()
        .id(list_container_id)
        .width(width)
        .fill_y()
        .column();

    if state.theme == crate::theme::ThemeMode::Glassmorphism {
        list_container = list_container
            .bg(colors.bg_panel.with_alpha(0.25))
            .backdrop_filter(zenthra::BackdropFilter::new().blur(12.0, zenthra::style::blur::Type::Glassmorphism));
    } else {
        list_container = list_container.bg(colors.bg_panel);
    }

    list_container.show(|ui| {
        ui.container()
            .fill()
            .column()
            .scroll_y(true)
            .padding(24.0, 24.0, 24.0, 24.0)
            .show(|ui| {
                // Header
                ui.container().column().gap(4.0).show(|ui| {
                    ui.text("System Dashboard")
                        .size(20.0)
                        .weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Overview of storage drives, resource usage, and system directories")
                        .size(11.5)
                        .color(colors.text_muted)
                        .show();
                });

                ui.spacing(24.0);

                // --- 1. STORAGE & DEVICES ---
                ui.text("STORAGE & DEVICES")
                    .size(9.5)
                    .weight(FontWeight::Bold)
                    .color(colors.text_muted)
                    .show();
                ui.spacing(12.0);

                ui.container().row().wrap(zenthra::Wrap::Wrap).gap(16.0).fill_x().show(|ui| {
                    let drives = state.detect_drives();
                    for (idx, drive) in drives.into_iter().enumerate() {
                        let space_info = crate::state::query_disk_space(&drive.path);
                        let (total_str, _used_str, free_str, percent) = if let Some(space) = space_info {
                            let total_gb = space.total as f64 / 1024.0 / 1024.0 / 1024.0;
                            let free_gb = space.free as f64 / 1024.0 / 1024.0 / 1024.0;
                            let used_gb = space.used as f64 / 1024.0 / 1024.0 / 1024.0;
                            let pct = (space.used as f32 / space.total as f32).clamp(0.0, 1.0);
                            (
                                format!("{:.1} GB", total_gb),
                                format!("{:.1} GB", used_gb),
                                format!("{:.1} GB", free_gb),
                                pct,
                            )
                        } else {
                            ("Unknown".to_string(), "0 GB".to_string(), "0 GB".to_string(), 0.0)
                        };

                        let drive_card_id = Id::from_u64(888889000 + idx as u64);

                        // Subtle hover feedback matching theme - read BEFORE container borrow
                        let is_card_hov = ui.interaction_state.get(&drive_card_id).copied().unwrap_or(0.0) > 0.5;
                        let is_selected = state.dashboard_selection == crate::state::DashboardSelection::Drive(drive.path.clone());

                        let mut card_container = ui.container()
                            .id(drive_card_id)
                            .width(300.0)
                            .padding(16.0, 16.0, 16.0, 16.0)
                            .radius_all(8.0)
                            .column()
                            .gap(10.0);

                        if is_selected {
                            card_container = card_container
                                .bg(colors.bg_active)
                                .border(colors.accent, 1.5);
                        } else if is_card_hov {
                            card_container = card_container
                                .bg(colors.highlight)
                                .border(colors.border, 1.0);
                        } else {
                            card_container = card_container
                                .bg(colors.bg_panel.with_alpha(0.3))
                                .border(colors.border, 1.0);
                        }

                        let resp = card_container.show(|ui| {
                            // Disk Icon + Name
                            ui.container().row().gap(12.0).valign(Align::Center).show(|ui| {
                                ui.text("\u{f0a0}").size(22.0).color(colors.accent).show();
                                ui.container().column().gap(2.0).show(|ui| {
                                    ui.text(&drive.name)
                                        .size(12.5)
                                        .weight(FontWeight::Bold)
                                        .color(colors.text_primary)
                                        .show();
                                    ui.text(&drive.path.to_string_lossy())
                                        .size(10.0)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            });

                            ui.spacing(2.0);

                            // Storage Progress Bar
                            ui.container()
                                .fill_x()
                                .height(6.0)
                                .bg(colors.border)
                                .radius_all(3.0)
                                .show(|ui| {
                                    let filled_w = percent * ui.available_width;
                                    ui.container()
                                        .width(filled_w)
                                        .fill_y()
                                        .bg(colors.accent)
                                        .radius_all(3.0)
                                        .show(|_| {});
                                });

                            // Storage Text details
                            ui.container().row().fill_x().show(|ui| {
                                ui.text(&format!("{} free of {}", free_str, total_str))
                                    .size(10.0)
                                    .color(colors.text_muted)
                                    .show();
                                let pct_text = format!("{:.0}%", percent * 100.0);
                                ui.container().halign(Align::Right).show(|ui| {
                                    ui.text(&pct_text)
                                        .size(10.0)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            });
                        });

                        // Hover cache
                        let h = resp.hovered;
                        ui.interaction_state.insert(drive_card_id, if h { 1.0 } else { 0.0 });

                        // Click — single selects, double navigates
                        let now = std::time::Instant::now();
                        let is_double = if let (Some(last_time), Some(last_idx)) = (state.last_click_time, state.last_clicked_idx) {
                            let elapsed = now.duration_since(last_time).as_millis();
                            last_idx == 1000 + idx && elapsed >= 80 && elapsed < 400
                        } else {
                            false
                        };

                        if resp.clicked {
                            if is_double {
                                state.last_click_time = None;
                                state.last_clicked_idx = None;
                                state.active_screen = crate::state::ActiveScreen::Browser;
                                state.change_dir(drive.path.clone());
                            } else {
                                state.last_click_time = Some(now);
                                state.last_clicked_idx = Some(1000 + idx);
                                state.dashboard_selection = crate::state::DashboardSelection::Drive(drive.path.clone());
                            }
                            ui.request_redraw();
                        }
                    }
                });

                ui.spacing(24.0);

                // --- 2. SYSTEM PERFORMANCE ---
                ui.text("SYSTEM PERFORMANCE")
                    .size(9.5)
                    .weight(FontWeight::Bold)
                    .color(colors.text_muted)
                    .show();
                ui.spacing(12.0);

                ui.container().row().wrap(zenthra::Wrap::Wrap).gap(16.0).fill_x().show(|ui| {
                    // CPU Card
                    let is_cpu_sel = state.dashboard_selection == crate::state::DashboardSelection::Cpu;
                    let mut cpu_card = ui.container()
                        .width(300.0)
                        .padding(16.0, 16.0, 16.0, 16.0)
                        .radius_all(8.0)
                        .column()
                        .gap(10.0);
                    if is_cpu_sel {
                        cpu_card = cpu_card.bg(colors.bg_active).border(colors.accent, 1.5);
                    } else {
                        cpu_card = cpu_card.bg(colors.bg_panel.with_alpha(0.3)).border(colors.border, 1.0);
                    }
                    let cpu_resp = cpu_card.show(|ui| {
                            let cpu_usage = crate::state::query_cpu_usage();
                            ui.container().row().fill_x().show(|ui| {
                                ui.text("CPU Load").size(12.5).weight(FontWeight::Bold).color(colors.text_primary).show();
                                ui.container().halign(Align::Right).show(|ui| {
                                    ui.text(&format!("{:.1}%", cpu_usage))
                                        .size(12.0)
                                        .weight(FontWeight::Bold)
                                        .color(colors.accent)
                                        .show();
                                });
                            });

                            ui.container()
                                .fill_x()
                                .height(6.0)
                                .bg(colors.border)
                                .radius_all(3.0)
                                .show(|ui| {
                                    let filled_w = (cpu_usage / 100.0) * ui.available_width;
                                    ui.container()
                                        .width(filled_w)
                                        .fill_y()
                                        .bg(colors.accent)
                                        .radius_all(3.0)
                                        .show(|_| {});
                                });

                            ui.text("Load average per core")
                                .size(10.0)
                                .color(colors.text_muted)
                                .show();
                    });
                    if cpu_resp.clicked {
                        state.dashboard_selection = crate::state::DashboardSelection::Cpu;
                        ui.request_redraw();
                    }

                    // RAM Card
                    let is_ram_sel = state.dashboard_selection == crate::state::DashboardSelection::Ram;
                    let mut ram_card = ui.container()
                        .width(300.0)
                        .padding(16.0, 16.0, 16.0, 16.0)
                        .radius_all(8.0)
                        .column()
                        .gap(10.0);
                    if is_ram_sel {
                        ram_card = ram_card.bg(colors.bg_active).border(colors.accent, 1.5);
                    } else {
                        ram_card = ram_card.bg(colors.bg_panel.with_alpha(0.3)).border(colors.border, 1.0);
                    }
                    let ram_resp = ram_card.show(|ui| {
                            let ram_info = crate::state::query_ram_usage().unwrap_or((4 * 1024 * 1024 * 1024, 16 * 1024 * 1024 * 1024));
                            let used_gb = ram_info.0 as f64 / 1024.0 / 1024.0 / 1024.0;
                            let total_gb = ram_info.1 as f64 / 1024.0 / 1024.0 / 1024.0;
                            let pct = (ram_info.0 as f32 / ram_info.1 as f32).clamp(0.0, 1.0);

                            ui.container().row().fill_x().show(|ui| {
                                ui.text("Memory Usage").size(12.5).weight(FontWeight::Bold).color(colors.text_primary).show();
                                ui.container().halign(Align::Right).show(|ui| {
                                    ui.text(&format!("{:.1}%", pct * 100.0))
                                        .size(12.0)
                                        .weight(FontWeight::Bold)
                                        .color(colors.accent)
                                        .show();
                                });
                            });

                            ui.container()
                                .fill_x()
                                .height(6.0)
                                .bg(colors.border)
                                .radius_all(3.0)
                                .show(|ui| {
                                    let filled_w = pct * ui.available_width;
                                    ui.container()
                                        .width(filled_w)
                                        .fill_y()
                                        .bg(colors.accent)
                                        .radius_all(3.0)
                                        .show(|_| {});
                                });

                            ui.text(&format!("{:.1} GB used of {:.1} GB", used_gb, total_gb))
                                .size(10.0)
                                .color(colors.text_muted)
                                .show();
                    });
                    if ram_resp.clicked {
                        state.dashboard_selection = crate::state::DashboardSelection::Ram;
                        ui.request_redraw();
                    }
                });

                ui.spacing(16.0);

                // OS / Host Specifications
                ui.container()
                    .fill_x()
                    .padding(16.0, 16.0, 16.0, 16.0)
                    .radius_all(8.0)
                    .border(colors.border, 1.0)
                    .bg(colors.bg_panel.with_alpha(0.3))
                    .column()
                    .gap(10.0)
                    .show(|ui| {
                        ui.text("System Specifications")
                            .size(12.5)
                            .weight(FontWeight::Bold)
                            .color(colors.text_primary)
                            .show();

                        ui.container().row().fill_x().show(|ui| {
                            ui.text("Operating System").size(11.0).color(colors.text_muted).show();
                            ui.container().halign(Align::Right).show(|ui| {
                                #[cfg(target_os = "linux")]
                                let os = "Linux / Pop!_OS";
                                #[cfg(target_os = "macos")]
                                let os = "macOS";
                                #[cfg(target_os = "windows")]
                                let os = "Windows";
                                ui.text(os).size(11.0).weight(FontWeight::Bold).color(colors.text_primary).show();
                            });
                        });

                        ui.container().row().fill_x().show(|ui| {
                            ui.text("Hostname").size(11.0).color(colors.text_muted).show();
                            ui.container().halign(Align::Right).show(|ui| {
                                let hostname = std::fs::read_to_string("/proc/sys/kernel/hostname")
                                    .unwrap_or_else(|_| "localhost".to_string())
                                    .trim()
                                    .to_string();
                                ui.text(&hostname).size(11.0).weight(FontWeight::Bold).color(colors.text_primary).show();
                            });
                        });
                    });

                ui.spacing(24.0);

                // --- 3. QUICK DIRECTORY ACCESS ---
                ui.text("QUICK DIRECTORY ACCESS")
                    .size(9.5)
                    .weight(FontWeight::Bold)
                    .color(colors.text_muted)
                    .show();
                ui.spacing(12.0);

                ui.container().row().wrap(zenthra::Wrap::Wrap).gap(12.0).fill_x().show(|ui| {
                    let shortcuts = vec![
                        ("\u{f015}", "Home", dirs::home_dir()),
                        ("\u{f019}", "Downloads", dirs::download_dir()),
                        ("\u{f15c}", "Documents", dirs::document_dir()),
                        ("\u{f108}", "Desktop", dirs::desktop_dir()),
                        ("\u{f03e}", "Pictures", dirs::picture_dir()),
                        ("\u{f001}", "Music", dirs::audio_dir()),
                        ("\u{f008}", "Videos", dirs::video_dir()),
                    ];

                    for (idx, (icon, name, path_opt)) in shortcuts.into_iter().enumerate() {
                        if let Some(path) = path_opt {
                            let card_id = Id::from_u64(888889200 + idx as u64);

                            // Read hover state BEFORE container borrow
                            let is_hov = ui.interaction_state.get(&card_id).copied().unwrap_or(0.0) > 0.5;

                            let mut card_container = ui.container()
                                .id(card_id)
                                .width(119.0)
                                .height(72.0)
                                .padding(12.0, 12.0, 12.0, 12.0)
                                .radius_all(6.0)
                                .border(colors.border, 1.0)
                                .column()
                                .gap(6.0)
                                .halign(Align::Center)
                                .valign(Align::Center);

                            if is_hov {
                                card_container = card_container.bg(colors.highlight);
                            } else {
                                card_container = card_container.bg(colors.bg_panel.with_alpha(0.3));
                            }

                            let resp = card_container.show(|ui| {
                                ui.text(icon).size(18.0).color(colors.accent).show();
                                ui.text(name).size(10.5).color(colors.text_primary).show();
                            });

                            let h = resp.hovered;
                            ui.interaction_state.insert(card_id, if h { 1.0 } else { 0.0 });

                            if resp.clicked {
                                state.dashboard_selection = crate::state::DashboardSelection::Directory(path.clone());
                                ui.request_redraw();
                            }
                        }
                    }
                });
            });
    });
}

pub fn draw_dashboard_sidebar(ui: &mut Ui, state: &mut FileManagerState) {
    let colors = state.colors();

    let mut sidebar = ui.container()
        .width(state.details_width)
        .fill_y()
        .column()
        .scroll_y(true)
        .padding(16.0, 16.0, 16.0, 16.0);

    if state.theme == crate::theme::ThemeMode::Glassmorphism {
        sidebar = sidebar
            .bg(colors.bg_sidebar.with_alpha(0.25))
            .backdrop_filter(zenthra::BackdropFilter::new().blur(12.0, zenthra::style::blur::Type::Glassmorphism));
    } else {
        sidebar = sidebar.bg(colors.bg_sidebar);
    }

    let selection = state.dashboard_selection.clone();

    sidebar.show(|ui| {
        match selection {
            // ──────────────────────────────────────────────────────────────
            // Drive detail
            // ──────────────────────────────────────────────────────────────
            crate::state::DashboardSelection::Drive(ref path) => {
                ui.text("\u{f0a0}  DRIVE DETAILS")
                    .size(9.5).weight(FontWeight::Bold).color(colors.text_muted).show();
                ui.spacing(14.0);

                let drive_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());

                ui.text(&drive_name)
                    .size(16.0).weight(FontWeight::Bold).color(colors.text_primary).show();
                ui.text(&path.to_string_lossy())
                    .size(10.0).color(colors.text_muted).show();

                ui.spacing(16.0);
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                ui.spacing(14.0);

                if let Some(space) = crate::state::query_disk_space(path) {
                    let total_gb = space.total as f64 / 1024.0 / 1024.0 / 1024.0;
                    let used_gb  = space.used  as f64 / 1024.0 / 1024.0 / 1024.0;
                    let free_gb  = space.free  as f64 / 1024.0 / 1024.0 / 1024.0;
                    let pct = (space.used as f32 / space.total as f32).clamp(0.0, 1.0);

                    // Big progress arc-style bar
                    ui.container()
                        .fill_x().height(8.0).bg(colors.border).radius_all(4.0)
                        .show(|ui| {
                            let w = pct * ui.available_width;
                            ui.container().width(w).fill_y().bg(colors.accent).radius_all(4.0).show(|_| {});
                        });
                    ui.spacing(8.0);

                    let rows = [
                        ("Total",      format!("{:.2} GB", total_gb)),
                        ("Used",       format!("{:.2} GB ({:.0}%)", used_gb, pct * 100.0)),
                        ("Free",       format!("{:.2} GB", free_gb)),
                    ];
                    for (label, val) in &rows {
                        ui.container().column().gap(2.0).fill_x().show(|ui| {
                            ui.text(*label).size(10.0).color(colors.text_muted).show();
                            ui.text(val).size(12.0).weight(FontWeight::Bold).color(colors.text_primary).show();
                        });
                        ui.spacing(10.0);
                    }
                } else {
                    ui.text("Could not read disk info").size(10.5).color(colors.text_muted).show();
                }

                ui.spacing(16.0);
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                ui.spacing(12.0);

                // Open button
                let open_id = Id::from_u64(888889800);
                let open_resp = ui.container()
                    .id(open_id)
                    .fill_x()
                    .height(34.0)
                    .bg(colors.accent)
                    .radius_all(6.0)
                    .halign(Align::Center)
                    .valign(Align::Center)
                    .show(|ui| {
                        ui.text("Open Drive").size(12.0).weight(FontWeight::Bold)
                            .color(colors.bg_base).show();
                    });
                if open_resp.clicked {
                    state.active_screen = crate::state::ActiveScreen::Browser;
                    state.change_dir(path.clone());
                    ui.request_redraw();
                }
            }

            // ──────────────────────────────────────────────────────────────
            // CPU detail
            // ──────────────────────────────────────────────────────────────
            crate::state::DashboardSelection::Cpu => {
                ui.text("\u{f109}  CPU DETAILS")
                    .size(9.5).weight(FontWeight::Bold).color(colors.text_muted).show();
                ui.spacing(14.0);

                let cpu = crate::state::query_cpu_usage();
                ui.container().row().fill_x().show(|ui| {
                    ui.text("CPU Load").size(14.0).weight(FontWeight::Bold).color(colors.text_primary).show();
                    ui.container().halign(Align::Right).show(|ui| {
                        ui.text(&format!("{:.1}%", cpu))
                            .size(14.0).weight(FontWeight::Bold).color(colors.accent).show();
                    });
                });
                ui.spacing(8.0);
                ui.container().fill_x().height(8.0).bg(colors.border).radius_all(4.0).show(|ui| {
                    let w = (cpu / 100.0) * ui.available_width;
                    ui.container().width(w).fill_y().bg(colors.accent).radius_all(4.0).show(|_| {});
                });

                ui.spacing(16.0);
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                ui.spacing(14.0);

                // Read /proc/cpuinfo for model name + cores
                let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
                let model = cpuinfo.lines()
                    .find(|l| l.starts_with("model name"))
                    .and_then(|l| l.splitn(2, ':').nth(1))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "Unknown CPU".to_string());
                let cores = cpuinfo.lines()
                    .filter(|l| l.starts_with("processor"))
                    .count();

                let rows = [
                    ("Model", model.clone()),
                    ("Logical Cores", format!("{}", cores)),
                    ("Load avg (1m)", {
                        std::fs::read_to_string("/proc/loadavg")
                            .ok()
                            .and_then(|s| s.split_whitespace().next().map(String::from))
                            .unwrap_or_else(|| "—".to_string())
                    }),
                ];
                for (label, val) in &rows {
                    ui.container().column().gap(2.0).fill_x().show(|ui| {
                        ui.text(*label).size(10.0).color(colors.text_muted).show();
                        ui.text(val).size(11.5).weight(FontWeight::Bold).color(colors.text_primary).show();
                    });
                    ui.spacing(10.0);
                }
            }

            // ──────────────────────────────────────────────────────────────
            // RAM detail
            // ──────────────────────────────────────────────────────────────
            crate::state::DashboardSelection::Ram => {
                ui.text("\u{f538}  MEMORY DETAILS")
                    .size(9.5).weight(FontWeight::Bold).color(colors.text_muted).show();
                ui.spacing(14.0);

                let ram = crate::state::query_ram_usage()
                    .unwrap_or((4 * 1024 * 1024 * 1024, 16 * 1024 * 1024 * 1024));
                let used_gb  = ram.0 as f64 / 1024.0 / 1024.0 / 1024.0;
                let total_gb = ram.1 as f64 / 1024.0 / 1024.0 / 1024.0;
                let free_gb  = total_gb - used_gb;
                let pct = (ram.0 as f32 / ram.1 as f32).clamp(0.0, 1.0);

                ui.container().row().fill_x().show(|ui| {
                    ui.text("Memory Usage").size(14.0).weight(FontWeight::Bold).color(colors.text_primary).show();
                    ui.container().halign(Align::Right).show(|ui| {
                        ui.text(&format!("{:.1}%", pct * 100.0))
                            .size(14.0).weight(FontWeight::Bold).color(colors.accent).show();
                    });
                });
                ui.spacing(8.0);
                ui.container().fill_x().height(8.0).bg(colors.border).radius_all(4.0).show(|ui| {
                    let w = pct * ui.available_width;
                    ui.container().width(w).fill_y().bg(colors.accent).radius_all(4.0).show(|_| {});
                });

                ui.spacing(16.0);
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                ui.spacing(14.0);

                // Parse /proc/meminfo for extra detail
                let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
                let get_field = |key: &str| -> String {
                    meminfo.lines()
                        .find(|l| l.starts_with(key))
                        .and_then(|l| l.split_whitespace().nth(1))
                        .and_then(|v| v.parse::<u64>().ok())
                        .map(|kb| format!("{:.2} GB", kb as f64 / 1024.0 / 1024.0))
                        .unwrap_or_else(|| "—".to_string())
                };

                let rows = [
                    ("Total",     format!("{:.2} GB", total_gb)),
                    ("Used",      format!("{:.2} GB", used_gb)),
                    ("Free",      format!("{:.2} GB", free_gb)),
                    ("Cached",    get_field("Cached:")),
                    ("Swap Total",get_field("SwapTotal:")),
                    ("Swap Free", get_field("SwapFree:")),
                ];
                for (label, val) in &rows {
                    ui.container().column().gap(2.0).fill_x().show(|ui| {
                        ui.text(*label).size(10.0).color(colors.text_muted).show();
                        ui.text(val).size(11.5).weight(FontWeight::Bold).color(colors.text_primary).show();
                    });
                    ui.spacing(10.0);
                }
            }

            // ──────────────────────────────────────────────────────────────
            // Directory shortcut detail
            // ──────────────────────────────────────────────────────────────
            crate::state::DashboardSelection::Directory(ref path) => {
                ui.text("\u{f07b}  FOLDER DETAILS")
                    .size(9.5).weight(FontWeight::Bold).color(colors.text_muted).show();
                ui.spacing(14.0);

                let dir_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.to_string_lossy().to_string());

                ui.text(&dir_name)
                    .size(16.0).weight(FontWeight::Bold).color(colors.text_primary).show();
                ui.text(&path.to_string_lossy())
                    .size(10.0).color(colors.text_muted).show();

                ui.spacing(16.0);
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                ui.spacing(14.0);

                // Count items in directory
                let item_count = std::fs::read_dir(path)
                    .map(|rd| rd.count())
                    .unwrap_or(0);
                let exists = path.exists();
                let readable = std::fs::read_dir(path).is_ok();

                let rows = [
                    ("Path",      path.to_string_lossy().to_string()),
                    ("Items",     format!("{}", item_count)),
                    ("Exists",    if exists { "Yes".to_string() } else { "No".to_string() }),
                    ("Readable",  if readable { "Yes".to_string() } else { "Permission Denied".to_string() }),
                ];
                for (label, val) in &rows {
                    ui.container().column().gap(2.0).fill_x().show(|ui| {
                        ui.text(*label).size(10.0).color(colors.text_muted).show();
                        ui.text(val).size(11.5).weight(FontWeight::Bold).color(colors.text_primary).show();
                    });
                    ui.spacing(10.0);
                }

                ui.spacing(6.0);
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                ui.spacing(12.0);

                // Open button
                let open_id = Id::from_u64(888889801);
                let open_resp = ui.container()
                    .id(open_id)
                    .fill_x()
                    .height(34.0)
                    .bg(colors.accent)
                    .radius_all(6.0)
                    .halign(Align::Center)
                    .valign(Align::Center)
                    .show(|ui| {
                        ui.text("Open Folder").size(12.0).weight(FontWeight::Bold)
                            .color(colors.bg_base).show();
                    });
                if open_resp.clicked {
                    state.active_screen = crate::state::ActiveScreen::Browser;
                    state.change_dir(path.clone());
                    ui.request_redraw();
                }
            }

            // ──────────────────────────────────────────────────────────────
            // Nothing selected — show compact system overview
            // ──────────────────────────────────────────────────────────────
            crate::state::DashboardSelection::None => {
                ui.text("OVERVIEW")
                    .size(9.5).weight(FontWeight::Bold).color(colors.text_muted).show();
                ui.spacing(8.0);
                ui.text("Click any card to see\ndetailed information.")
                    .size(11.0).color(colors.text_muted).show();
                ui.spacing(16.0);
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                ui.spacing(14.0);

                // Live mini-gauges
                let cpu = crate::state::query_cpu_usage();
                let ram = crate::state::query_ram_usage().unwrap_or((4 * 1024 * 1024 * 1024, 16 * 1024 * 1024 * 1024));
                let ram_pct = (ram.0 as f32 / ram.1 as f32).clamp(0.0, 1.0);

                for (label, pct) in &[("CPU", cpu / 100.0), ("RAM", ram_pct)] {
                    ui.container().column().gap(5.0).fill_x().show(|ui| {
                        ui.container().row().fill_x().show(|ui| {
                            ui.text(*label).size(11.0).weight(FontWeight::Bold).color(colors.text_primary).show();
                            ui.container().halign(Align::Right).show(|ui| {
                                ui.text(&format!("{:.0}%", pct * 100.0))
                                    .size(11.0).weight(FontWeight::Bold).color(colors.accent).show();
                            });
                        });
                        ui.container().fill_x().height(5.0).bg(colors.border).radius_all(3.0).show(|ui| {
                            let w = pct * ui.available_width;
                            ui.container().width(w).fill_y().bg(colors.accent).radius_all(3.0).show(|_| {});
                        });
                    });
                    ui.spacing(12.0);
                }
            }
        }
    });
}


