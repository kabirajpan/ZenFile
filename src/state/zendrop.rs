use std::path::{Path, PathBuf};
use std::net::{UdpSocket, TcpListener, TcpStream};
use std::io::{Read, Write, BufWriter};
use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use super::types::{FileManagerState, ZendropDevice, WifiNetwork, TransferEntry, TransferStatus, ZendropSendMode};

impl FileManagerState {
    pub fn refresh_zendrop(&mut self) {
        if self.zendrop_scanning.load(Ordering::SeqCst) {
            return;
        }
        self.zendrop_scanning.store(true, Ordering::SeqCst);
        self.load_paired_devices();

        let scan_results_arc = self.zendrop_scan_results.clone();
        let redraw_arc = self.pending_refresh.clone();

        std::thread::spawn(move || {
            let devices = scan_zendrop_devices();
            let networks = scan_wifi_networks();
            let name = query_network_name();

            *scan_results_arc.lock().unwrap() = Some((devices, networks, name));
            redraw_arc.store(true, Ordering::SeqCst);
        });
        self.last_zendrop_scan = Some(std::time::Instant::now());
    }

    pub fn forget_paired_device(&mut self, ip: String) {
        self.zendrop_paired.retain(|(_, paired_ip, _)| paired_ip != &ip);
        self.save_paired_devices();
    }

    pub fn disconnect_from_wifi(&mut self, ssid: String) {
        self.wifi_connecting = true;
        self.wifi_connect_error = None;
        
        let ssid_clone = ssid.clone();
        let result_arc = self.wifi_connection_result.clone();
        
        std::thread::spawn(move || {
            let mut cmd = std::process::Command::new("nmcli");
            cmd.args(&["connection", "down", "id", &ssid_clone]);
            
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        *result_arc.lock().unwrap() = Some(Ok(String::new()));
                    } else {
                        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
                        let err = if err.is_empty() {
                            String::from_utf8_lossy(&output.stdout).trim().to_string()
                        } else {
                            err
                        };
                        *result_arc.lock().unwrap() = Some(Err(if err.is_empty() { "Disconnect failed".to_string() } else { err }));
                    }
                }
                Err(e) => {
                    *result_arc.lock().unwrap() = Some(Err(e.to_string()));
                }
            }
        });
    }

    pub fn connect_to_wifi(&mut self, ssid: String, password: Option<String>) {
        self.wifi_connecting = true;
        self.wifi_connect_error = None;
        
        let ssid_clone = ssid.clone();
        let pass_clone = password;
        let result_arc = self.wifi_connection_result.clone();
        
        std::thread::spawn(move || {
            let mut cmd = std::process::Command::new("nmcli");
            cmd.args(&["dev", "wifi", "connect", &ssid_clone]);
            if let Some(ref pass) = pass_clone {
                if !pass.is_empty() {
                    cmd.args(&["password", pass]);
                }
            }
            
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        *result_arc.lock().unwrap() = Some(Ok(ssid_clone));
                    } else {
                        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
                        let err = if err.is_empty() {
                            String::from_utf8_lossy(&output.stdout).trim().to_string()
                        } else {
                            err
                        };
                        *result_arc.lock().unwrap() = Some(Err(if err.is_empty() { "Connection failed".to_string() } else { err }));
                    }
                }
                Err(e) => {
                    *result_arc.lock().unwrap() = Some(Err(e.to_string()));
                }
            }
        });
    }

    pub fn start_zendrop_send(&mut self, device: ZendropDevice) {
        self.zendrop_open = true;

        let mut files_to_send: Vec<PathBuf> = Vec::new();
        let is_clipboard = self.zendrop_send_mode == ZendropSendMode::Clipboard;
        if is_clipboard {
            if let Some(clip_text) = read_from_clipboard() {
                let temp_file = std::env::temp_dir().join("Clipboard_Content.txt");
                if std::fs::write(&temp_file, clip_text).is_ok() {
                    files_to_send.push(temp_file);
                }
            }
            if files_to_send.is_empty() {
                self.zendrop_send_status = "Clipboard is empty or could not be read".to_string();
                return;
            }
        } else if self.selected_paths.is_empty() {
            files_to_send.push(self.current_dir.clone());
        } else {
            files_to_send = self.selected_paths.iter().cloned().collect();
        }

        let mut flat_files: Vec<(String, PathBuf)> = Vec::new();
        for root in &files_to_send {
            let base = root.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| root.clone());
            let mut stack = vec![root.clone()];
            while let Some(current) = stack.pop() {
                if current.is_file() {
                    let rel = current.strip_prefix(&base)
                        .unwrap_or(&current)
                        .to_string_lossy()
                        .into_owned();
                    flat_files.push((rel, current));
                } else if current.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&current) {
                        for entry in entries.flatten() {
                            stack.push(entry.path());
                        }
                    }
                }
            }
        }

        if flat_files.is_empty() {
            return;
        }

        let ip = device.ip.clone();
        let progress_arc = self.zendrop_progress.clone();
        progress_arc.store(0, Ordering::SeqCst);
        let status_msg_clone = self.zendrop_status_msg.clone();

        let mut total_bytes = 0_u64;
        for (_, path) in &flat_files {
            total_bytes += std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        }

        let entries: Vec<TransferEntry> = flat_files.iter().map(|(rel, path)| {
            let filename = Path::new(rel)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| rel.clone());
            TransferEntry {
                filename,
                total_bytes: std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
                sent_bytes: Arc::new(std::sync::atomic::AtomicU64::new(0)),
                status: Arc::new(Mutex::new(TransferStatus::Sending)),
                speed_mbps: Arc::new(Mutex::new(0.0)),
                started_at: std::time::Instant::now(),
                cancel_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            }
        }).collect();

        let entry_refs: Vec<(
            Arc<std::sync::atomic::AtomicU64>,
            Arc<Mutex<TransferStatus>>,
            Arc<Mutex<f32>>,
            Arc<std::sync::atomic::AtomicBool>,
        )> = entries.iter().map(|e| (
            e.sent_bytes.clone(),
            e.status.clone(),
            e.speed_mbps.clone(),
            e.cancel_flag.clone(),
        )).collect();

        let mut new_transfers = entries;
        new_transfers.extend(self.zendrop_transfers.drain(..));
        new_transfers.truncate(20);
        self.zendrop_transfers = new_transfers;

        std::thread::spawn(move || {
            let mut sent_bytes_total = 0_u64;

            let stream = match TcpStream::connect((ip.as_str(), 8888)) {
                Ok(s) => s,
                Err(e) => {
                    for (_, status_arc, _, _) in &entry_refs {
                        *status_arc.lock().unwrap() = TransferStatus::Failed;
                    }
                    *status_msg_clone.lock().unwrap() = Some(format!("Connection failed: {}", e));
                    progress_arc.store(100, Ordering::SeqCst);
                    return;
                }
            };
            let _ = stream.set_nodelay(true);
            let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(60)));

            let mut writer = BufWriter::with_capacity(256 * 1024, stream);
            let mut aborted = false;

            if writer.write_all(&[0x5A, 0x44, 0x52, 0x50]).is_err() {
                for (_, status_arc, _, _) in &entry_refs {
                    *status_arc.lock().unwrap() = TransferStatus::Failed;
                }
                *status_msg_clone.lock().unwrap() = Some("Failed to send handshake".to_string());
                progress_arc.store(100, Ordering::SeqCst);
                return;
            }

            for (idx, (rel_name, path)) in flat_files.iter().enumerate() {
                let (sent_arc, status_arc, speed_arc, cancel_arc) = &entry_refs[idx];
                let file_len = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

                if cancel_arc.load(Ordering::SeqCst) {
                    *status_arc.lock().unwrap() = TransferStatus::Failed;
                    aborted = true;
                    break;
                }

                *status_msg_clone.lock().unwrap() = Some(format!("Sending ({}/{}): {}", idx + 1, flat_files.len(), rel_name));

                let mut file = match std::fs::File::open(path) {
                    Ok(f) => f,
                    Err(_) => {
                        *status_arc.lock().unwrap() = TransferStatus::Failed;
                        continue;
                    }
                };

                let name_bytes = rel_name.as_bytes();
                if name_bytes.len() > 65535 { continue; }
                let name_len = name_bytes.len() as u16;

                let mut header = Vec::with_capacity(1 + 2 + name_bytes.len() + 8);
                header.push(0x01);
                header.extend_from_slice(&name_len.to_be_bytes());
                header.extend_from_slice(name_bytes);
                header.extend_from_slice(&file_len.to_be_bytes());
                if writer.write_all(&header).is_err() {
                    *status_arc.lock().unwrap() = TransferStatus::Failed;
                    aborted = true; break;
                }

                let mut buffer = vec![0u8; 256 * 1024];
                let mut file_sent = 0_u64;
                let file_start = std::time::Instant::now();

                while file_sent < file_len {
                    if cancel_arc.load(Ordering::Relaxed) {
                        *status_arc.lock().unwrap() = TransferStatus::Failed;
                        aborted = true; break;
                    }
                    let n = match file.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(_) => { aborted = true; break; },
                    };
                    if writer.write_all(&buffer[..n]).is_err() { aborted = true; break; }
                    file_sent += n as u64;
                    sent_bytes_total += n as u64;
                    sent_arc.store(file_sent, Ordering::Relaxed);
                    if total_bytes > 0 {
                        let pct = ((sent_bytes_total * 100) / total_bytes).min(99) as u32;
                        progress_arc.store(pct, Ordering::SeqCst);
                    }
                }

                if !aborted {
                    if writer.flush().is_err() {
                        aborted = true;
                    }
                }

                if aborted {
                    *status_arc.lock().unwrap() = TransferStatus::Failed;
                    break;
                }

                let elapsed = file_start.elapsed().as_secs_f32().max(0.001);
                let mb = file_sent as f32 / 1_048_576.0;
                *speed_arc.lock().unwrap() = mb / elapsed;
                sent_arc.store(file_len, Ordering::Relaxed);
                *status_arc.lock().unwrap() = TransferStatus::Done;
            }

            if !aborted {
                let _ = writer.write_all(&[0x00]);
                let _ = writer.flush();
                progress_arc.store(100, Ordering::SeqCst);
                *status_msg_clone.lock().unwrap() = Some("Transfer complete!".to_string());
            } else {
                let _ = writer.flush();
                progress_arc.store(100, Ordering::SeqCst);
                if *status_msg_clone.lock().unwrap() == None {
                    *status_msg_clone.lock().unwrap() = Some("Transfer aborted".to_string());
                }
            }
        });
    }

    pub fn load_paired_devices(&mut self) {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("zenthra");
        path.push("paired_devices.txt");
        self.zendrop_paired.clear();
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() { continue; }
                if let Some((rest, token)) = line.rsplit_once('=') {
                    if let Some((name, ip)) = rest.split_once('=') {
                        self.zendrop_paired.push((name.to_string(), ip.to_string(), token.to_string()));
                    } else {
                        self.zendrop_paired.push((rest.to_string(), token.to_string(), String::new()));
                    }
                }
            }
        }
    }

    pub fn save_paired_devices(&self) {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("zenthra");
        let _ = std::fs::create_dir_all(&path);
        path.push("paired_devices.txt");
        let mut content = String::new();
        for (name, ip, token) in &self.zendrop_paired {
            content.push_str(&format!("{}={}={}\n", name, ip, token));
        }
        let _ = std::fs::write(path, content);
    }

    pub fn load_device_name(&mut self) {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("zenthra");
        path.push("device_name.txt");
        if let Ok(content) = std::fs::read_to_string(path) {
            let name = content.trim().to_string();
            if !name.is_empty() {
                *self.zendrop_device_name.lock().unwrap() = name;
            }
        }
    }

    pub fn save_device_name(&self, new_name: &str) {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("zenthra");
        let _ = std::fs::create_dir_all(&path);
        path.push("device_name.txt");
        let _ = std::fs::write(path, new_name);
    }

    pub fn pair_device(&mut self, target_ip: String) {
        self.zendrop_pairing_active = true;
        self.zendrop_status_msg = Arc::new(Mutex::new(Some("Sending pairing request...".to_string())));
        
        let ip = target_ip.clone();
        let status_arc = self.zendrop_status_msg.clone();
        let pending_refresh = self.pending_refresh.clone();
        let zendrop_device_name_clone = self.zendrop_device_name.clone();

        std::thread::spawn(move || {
            let local_name = zendrop_device_name_clone.lock().unwrap().clone();
            let local_ip = get_desktop_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());

            let mut stream = match TcpStream::connect((ip.as_str(), 8888)) {
                Ok(s) => s,
                Err(e) => {
                    *status_arc.lock().unwrap() = Some(format!("Connection failed: {}", e));
                    return;
                }
            };

            let _ = stream.set_nodelay(true);
            let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(10)));
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(35)));

            let name_bytes = local_name.as_bytes();
            let lip_bytes = local_ip.as_bytes();

            let mut packet = Vec::new();
            packet.extend_from_slice(&0xFFFF_u16.to_be_bytes());
            packet.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
            packet.extend_from_slice(name_bytes);
            packet.extend_from_slice(&(lip_bytes.len() as u16).to_be_bytes());
            packet.extend_from_slice(lip_bytes);

            if stream.write_all(&packet).is_err() || stream.flush().is_err() {
                *status_arc.lock().unwrap() = Some("Failed to send pairing request".to_string());
                return;
            }

            *status_arc.lock().unwrap() = Some("Request sent. Awaiting acceptance on mobile...".to_string());

            let mut resp = [0u8; 1];
            if stream.read_exact(&mut resp).is_err() {
                *status_arc.lock().unwrap() = Some("Pairing request timed out or declined".to_string());
                return;
            }

            if resp[0] == 0x01 {
                *status_arc.lock().unwrap() = Some("Pairing successful!".to_string());
                let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
                path.push("zenthra");
                let _ = std::fs::create_dir_all(&path);
                path.push("paired_devices.txt");
                if let Ok(mut file) = std::fs::OpenOptions::new().append(true).create(true).open(path) {
                    let _ = writeln!(file, "Mobile-Phone={}", ip);
                }
                pending_refresh.store(true, Ordering::SeqCst);
            } else {
                *status_arc.lock().unwrap() = Some("Pairing declined by mobile user".to_string());
            }
        });

        self.load_paired_devices();
        self.zendrop_pairing_active = false;
    }

    pub fn init_background_services(&self) {
        let scanned_devices_arc = self.zendrop_scanned.clone();
        let zendrop_device_name_clone = self.zendrop_device_name.clone();
        std::thread::spawn(move || {
            let mut hasher = DefaultHasher::new();
            std::time::Instant::now().hash(&mut hasher);
            let my_instance_id = hasher.finish() as u32;

            let socket = match UdpSocket::bind("0.0.0.0:8889") {
                Ok(s) => s,
                Err(_) => return,
            };
            let _ = socket.set_broadcast(true);
            let _ = socket.set_nonblocking(true);

            let mut buffer = [0u8; 1024];
            let mut last_broadcast = std::time::Instant::now() - std::time::Duration::from_secs(5);

            loop {
                let local_ip = get_desktop_local_ip();

                {
                    if let Ok(mut scanned) = scanned_devices_arc.lock() {
                        scanned.retain(|(_, _, last_seen)| last_seen.elapsed().as_secs() < 6);
                    }
                }

                if let Ok((n, addr)) = socket.recv_from(&mut buffer) {
                    let msg = String::from_utf8_lossy(&buffer[..n]);
                    if msg.starts_with("ZenDropBeacon::") {
                        let parts: Vec<&str> = msg.split("::").collect();
                        if parts.len() == 3 {
                            let name = parts[1].to_string();
                            let beacon_inst_id: u32 = parts[2].parse().unwrap_or(0);
                            let ip = addr.ip().to_string();
                            
                            let is_self = beacon_inst_id == my_instance_id;

                            if !is_self {
                                if let Ok(mut list) = scanned_devices_arc.lock() {
                                    if let Some(pos) = list.iter().position(|(_, item_ip, _)| item_ip == &ip) {
                                        list[pos] = (name, ip, std::time::Instant::now());
                                    } else {
                                        list.push((name, ip, std::time::Instant::now()));
                                    }
                                }
                            }
                        }
                    }
                }

                if last_broadcast.elapsed().as_secs() >= 2 {
                    let local_name = zendrop_device_name_clone.lock().unwrap().clone();
                    let beacon_msg = format!("ZenDropBeacon::{}::{}", local_name, my_instance_id);
                    let _ = socket.send_to(beacon_msg.as_bytes(), "255.255.255.255:8889");
                    
                    if let Some(ref lip) = local_ip {
                        let parts: Vec<&str> = lip.split('.').collect();
                        if parts.len() == 4 {
                            let sub_broadcast = format!("{}.{}.{}.255", parts[0], parts[1], parts[2]);
                            let _ = socket.send_to(beacon_msg.as_bytes(), &(sub_broadcast.as_str(), 8889));
                        }
                    }
                    
                    last_broadcast = std::time::Instant::now();
                }

                std::thread::sleep(std::time::Duration::from_millis(150));
            }
        });

        let download_dir = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap_or_default()))
            .join("ZenDrop");

        let pending_refresh_clone = self.pending_refresh.clone();
        let zendrop_pair_request_clone = self.zendrop_pair_request.clone();
        let zendrop_pair_result_clone = self.zendrop_pair_result.clone();
        std::thread::spawn(move || {
            let listener = match TcpListener::bind("0.0.0.0:8888") {
                Ok(l) => l,
                Err(_) => return,
            };

            for stream in listener.incoming() {
                let stream = match stream {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let download_dir = download_dir.clone();
                let pending_refresh_clone = pending_refresh_clone.clone();
                let zendrop_pair_request_clone = zendrop_pair_request_clone.clone();
                let zendrop_pair_result_clone = zendrop_pair_result_clone.clone();

                std::thread::spawn(move || {
                    use std::io::{BufReader, BufWriter};

                    let mut reader = BufReader::with_capacity(256 * 1024, stream);
                    let mut first_two = [0; 2];
                    if reader.read_exact(&mut first_two).is_err() { return; }

                    if first_two == [0xFF, 0xFF] {
                        let mut name_len_buf = [0; 2];
                        if reader.read_exact(&mut name_len_buf).is_err() { return; }
                        let name_len = u16::from_be_bytes(name_len_buf) as usize;

                        let mut name_buf = vec![0; name_len];
                        if reader.read_exact(&mut name_buf).is_err() { return; }
                        let req_name = String::from_utf8_lossy(&name_buf).into_owned();

                        let mut ip_len_buf = [0; 2];
                        if reader.read_exact(&mut ip_len_buf).is_err() { return; }
                        let ip_len = u16::from_be_bytes(ip_len_buf) as usize;

                        let mut ip_buf = vec![0; ip_len];
                        if reader.read_exact(&mut ip_buf).is_err() { return; }
                        let req_ip = String::from_utf8_lossy(&ip_buf).into_owned();

                        let peer_ip = reader.get_ref().peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|_| req_ip.clone());
                        let actual_ip = if req_ip == "127.0.0.1" && peer_ip != "127.0.0.1" {
                            peer_ip
                        } else {
                            req_ip
                        };

                        zendrop_pair_result_clone.store(0, Ordering::SeqCst);
                        {
                            let mut req = zendrop_pair_request_clone.lock().unwrap();
                            *req = Some((req_name.clone(), actual_ip.clone()));
                        }
                        pending_refresh_clone.store(true, Ordering::SeqCst);

                        let start_wait = std::time::Instant::now();
                        let mut result = 0;
                        loop {
                            if start_wait.elapsed().as_secs() > 30 {
                                break;
                            }
                            let val = zendrop_pair_result_clone.load(Ordering::SeqCst);
                            if val != 0 {
                                result = val;
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(200));
                        }

                        let writer_stream = reader.get_mut();
                        if result == 1 {
                            let _ = writer_stream.write_all(&[0x01]);
                            let _ = writer_stream.flush();

                            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
                            path.push("zenthra");
                            let _ = std::fs::create_dir_all(&path);
                            path.push("paired_devices.txt");
                            if let Ok(mut file) = std::fs::OpenOptions::new().append(true).create(true).open(path) {
                                let _ = writeln!(file, "{}={}", req_name, actual_ip);
                            }
                        } else {
                            let _ = writer_stream.write_all(&[0x02]);
                            let _ = writer_stream.flush();
                        }

                        {
                            let mut req = zendrop_pair_request_clone.lock().unwrap();
                            *req = None;
                        }
                        zendrop_pair_result_clone.store(0, Ordering::SeqCst);
                        pending_refresh_clone.store(true, Ordering::SeqCst);

                    } else if first_two == [0x5A, 0x44] {
                        let mut next_two = [0; 2];
                        if reader.read_exact(&mut next_two).is_err() { return; }
                        if next_two != [0x52, 0x50] { return; }

                        loop {
                            let mut frame_type = [0; 1];
                            if reader.read_exact(&mut frame_type).is_err() { break; }

                            if frame_type[0] == 0x00 {
                                break;
                            } else if frame_type[0] == 0x01 {
                                let mut name_len_buf = [0; 2];
                                if reader.read_exact(&mut name_len_buf).is_err() { break; }
                                let name_len = u16::from_be_bytes(name_len_buf) as usize;

                                let mut name_buf = vec![0; name_len];
                                if reader.read_exact(&mut name_buf).is_err() { break; }
                                let rel_name = String::from_utf8_lossy(&name_buf).into_owned();

                                let mut file_len_buf = [0; 8];
                                if reader.read_exact(&mut file_len_buf).is_err() { break; }
                                let file_len = u64::from_be_bytes(file_len_buf);

                                let target_path = download_dir.join(rel_name);
                                if let Some(parent) = target_path.parent() {
                                    let _ = std::fs::create_dir_all(parent);
                                }

                                let file = match std::fs::File::create(&target_path) {
                                    Ok(f) => f,
                                    Err(_) => break,
                                };
                                let mut file_writer = BufWriter::with_capacity(256 * 1024, file);

                                let mut limit = (&mut reader).take(file_len);
                                if std::io::copy(&mut limit, &mut file_writer).is_err() { break; }
                                if file_writer.flush().is_err() { break; }
                            } else {
                                break;
                            }
                        }

                        pending_refresh_clone.store(true, Ordering::SeqCst);
                    }
                });
            }
        });
    }
}

fn get_desktop_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}

pub fn scan_zendrop_devices() -> Vec<ZendropDevice> {
    let mut devices = Vec::new();

    if let Some(gw_ip) = get_default_gateway() {
        devices.push(ZendropDevice {
            ip: gw_ip,
            mac: "Hotspot".to_string(),
            hostname: "Mobile App Gateway".to_string(),
            iface: "wlp2s0".to_string(),
        });
    }

    if let Ok(content) = std::fs::read_to_string("/proc/net/arp") {
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let ip = parts[0].to_string();
                let mac = parts[3].to_string();
                let iface = parts[5].to_string();
                
                if mac == "00:00:00:00:00:00" {
                    continue;
                }

                if devices.iter().any(|d| d.ip == ip) {
                    continue;
                }

                let hostname = lookup_hostname(&ip);
                
                devices.push(ZendropDevice { ip, mac, hostname, iface });
            }
        }
    }
    devices
}

fn lookup_hostname(ip: &str) -> String {
    if let Ok(hosts) = std::fs::read_to_string("/etc/hosts") {
        for line in hosts.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.first() == Some(&ip) {
                if let Some(name) = parts.get(1) {
                    return name.to_string();
                }
            }
        }
    }
    ip.to_string()
}

fn get_default_gateway() -> Option<String> {
    let content = std::fs::read_to_string("/proc/net/route").ok()?;
    for line in content.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let dest = parts[1];
            let gw_hex = parts[2];
            if dest == "00000000" && gw_hex != "00000000" {
                if let Ok(val) = u32::from_str_radix(gw_hex, 16) {
                    let bytes = val.to_ne_bytes();
                    return Some(format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3]));
                }
            }
        }
    }
    None
}

pub fn query_network_name() -> String {
    let output = std::process::Command::new("nmcli")
        .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            if line.starts_with("yes:") {
                return line.trim_start_matches("yes:").to_string();
            }
        }
    }
    "Local Network".to_string()
}

pub fn scan_wifi_networks() -> Vec<WifiNetwork> {
    let mut networks_map: HashMap<String, WifiNetwork> = HashMap::new();

    let output = std::process::Command::new("nmcli")
        .args(&["-t", "-f", "ssid,signal,security,active", "dev", "wifi"])
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 4 {
                let ssid = parts[0].to_string();
                if ssid.is_empty() { continue; }
                let signal: u8 = parts[1].parse().unwrap_or(0);
                let security = parts[2].to_string();
                let active = parts[3] == "yes";

                let entry = networks_map.entry(ssid.clone()).or_insert(WifiNetwork {
                    ssid: ssid.clone(),
                    signal,
                    security: security.clone(),
                    active,
                });

                if active {
                    *entry = WifiNetwork {
                        ssid,
                        signal,
                        security,
                        active: true,
                    };
                } else if signal > entry.signal {
                    entry.signal = signal;
                }
            }
        }
    }

    let mut result: Vec<WifiNetwork> = networks_map.into_values().collect();
    result.sort_by(|a, b| b.signal.cmp(&a.signal));
    result
}

fn read_from_clipboard() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(&["-NoProfile", "-Command", "Get-Clipboard"])
            .output()
            .ok()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("pbpaste").output().ok()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("wl-paste").output() {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
        if let Ok(output) = std::process::Command::new("xclip")
            .args(&["-o", "-selection", "clipboard"])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
        if let Ok(output) = std::process::Command::new("xsel")
            .args(&["-o", "-b"])
            .output()
        {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !s.is_empty() {
                    return Some(s);
                }
            }
        }
    }
    None
}
