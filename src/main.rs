use eframe::{App, Frame, egui};
use egui_extras::{Column, TableBuilder};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use sysinfo::System;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Default)]
struct IpInfo {
    #[serde(default)]
    query: String,
    #[serde(default)]
    country: String,
    #[serde(default)]
    city: String,
    #[serde(default)]
    isp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct IpDetails {
    country: String,
    city: String,
    isp: String,
}

#[derive(Clone, Debug, PartialEq)]
enum IpDetailStatus {
    Pending,
    Fetched(IpDetails),
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ConnectionRecord {
    foreign_ip: String,
    state: String,
    first_seen: String,
    last_seen: String,
    active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ProcessRecord {
    name: String,
    pid: String,
    connections: HashMap<String, ConnectionRecord>, // Key: foreign_ip
}

struct TempConn {
    foreign_ip: String,
    state: String,
    pid: String,
    process_name: String,
}

struct AppState {
    processes: Arc<Mutex<HashMap<String, ProcessRecord>>>,
    current_ip: Arc<Mutex<Option<IpInfo>>>,
    ip_cache: Arc<Mutex<HashMap<String, IpDetailStatus>>>,
    ctx: egui::Context,
}

impl AppState {
    fn new(ctx: egui::Context) -> Self {
        let processes = Arc::new(Mutex::new(load_history()));
        let current_ip = Arc::new(Mutex::new(None));
        let ip_cache = Arc::new(Mutex::new(HashMap::new()));

        let processes_clone = processes.clone();
        let current_ip_clone = current_ip.clone();
        let ctx_clone = ctx.clone();

        std::thread::spawn(move || {
            let mut last_ip: Option<IpInfo> = None;
            let mut sys = System::new_all();

            loop {
                sys.refresh_processes();

                if let Ok(new_info) = fetch_public_ip() {
                    let time = get_timestamp();
                    let conns = get_connected_ips(&sys);

                    let mut log_changed = false;
                    if let Some(old_info) = &last_ip {
                        if old_info.query != new_info.query {
                            Notification::new()
                                .summary("🌐 IP Changed!")
                                .body(&format!(
                                    "New IP: {}\nLocation: {}, {}\nISP: {}",
                                    new_info.query, new_info.city, new_info.country, new_info.isp
                                ))
                                .show()
                                .ok();
                        }
                    } else {
                        Notification::new()
                            .summary("🟢 IP Logger Started")
                            .body(&format!(
                                "Current IP: {}\nLocation: {}, {}",
                                new_info.query, new_info.city, new_info.country
                            ))
                            .show()
                            .ok();
                    }
                    last_ip = Some(new_info.clone());

                    let mut cur = current_ip_clone.lock().unwrap();
                    *cur = Some(new_info.clone());
                    drop(cur);

                    let mut procs = processes_clone.lock().unwrap();

                    for conn in conns {
                        let pid = conn.pid.clone();
                        let foreign_ip = conn.foreign_ip.clone();

                        let record = procs.entry(pid.clone()).or_insert(ProcessRecord {
                            name: conn.process_name.clone(),
                            pid: pid.clone(),
                            connections: HashMap::new(),
                        });

                        // Update process name if it was "Unknown" but now resolved
                        if record.name == "Unknown" && conn.process_name != "Unknown" {
                            record.name = conn.process_name.clone();
                        }

                        let conn_record = record.connections.entry(foreign_ip.clone()).or_insert(
                            ConnectionRecord {
                                foreign_ip: foreign_ip.clone(),
                                state: conn.state.clone(),
                                first_seen: time.clone(),
                                last_seen: time.clone(),
                                active: true,
                            },
                        );

                        if conn_record.last_seen != time
                            || conn_record.state != conn.state
                            || !conn_record.active
                        {
                            conn_record.last_seen = time.clone();
                            conn_record.state = conn.state.clone();
                            conn_record.active = true;
                            log_changed = true;
                        }
                    }

                    // Mark old connections as inactive
                    for (_, proc) in procs.iter_mut() {
                        for (_, conn) in proc.connections.iter_mut() {
                            if conn.last_seen != time && conn.active {
                                conn.active = false;
                                log_changed = true;
                            }
                        }
                    }

                    let processes_snapshot = procs.clone();
                    drop(procs);

                    if log_changed {
                        save_history(&processes_snapshot);
                    }

                    ctx_clone.request_repaint();
                } else {
                    ctx_clone.request_repaint();
                }
                std::thread::sleep(Duration::from_secs(30));
            }
        });

        AppState {
            processes,
            current_ip,
            ip_cache,
            ctx,
        }
    }

    fn fetch_ip_details(&self, ip: String) {
        let mut cache = self.ip_cache.lock().unwrap();
        if cache.contains_key(&ip) {
            return;
        }
        cache.insert(ip.clone(), IpDetailStatus::Pending);
        drop(cache);

        let cache_clone = self.ip_cache.clone();
        let ctx_clone = self.ctx.clone();

        std::thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap();

            let url = format!("http://ip-api.com/json/{}", ip);
            if let Ok(res) = client.get(&url).send()
                && let Ok(info) = res.json::<IpInfo>()
            {
                let mut c = cache_clone.lock().unwrap();
                if info.query.is_empty() {
                    c.insert(ip, IpDetailStatus::Failed);
                } else {
                    c.insert(
                        ip,
                        IpDetailStatus::Fetched(IpDetails {
                            country: info.country,
                            city: info.city,
                            isp: info.isp,
                        }),
                    );
                }
                ctx_clone.request_repaint();
                return;
            }

            let mut c = cache_clone.lock().unwrap();
            c.insert(ip, IpDetailStatus::Failed);
            ctx_clone.request_repaint();
        });
    }
}

impl App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.heading(
                    egui::RichText::new("📝 Advanced Process IP Logger")
                        .size(24.0)
                        .strong(),
                );
            });
            ui.add_space(10.0);
        });

        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading("Current Status");
                ui.separator();

                let cur_ip = self.current_ip.lock().unwrap().clone();
                if let Some(info) = cur_ip {
                    ui.label(egui::RichText::new("🟢 Active IP").color(egui::Color32::GREEN));
                    ui.label(egui::RichText::new(&info.query).size(20.0).strong());
                    ui.add_space(10.0);

                    ui.label(egui::RichText::new("🌍 Location").color(egui::Color32::LIGHT_BLUE));
                    ui.label(format!("{}, {}", info.city, info.country));
                    ui.add_space(10.0);

                    ui.label(egui::RichText::new("🏢 ISP").color(egui::Color32::GOLD));
                    ui.label(&info.isp);
                } else {
                    ui.label(egui::RichText::new("🔴 Fetching IP...").color(egui::Color32::RED));
                }

                ui.add_space(20.0);
                ui.separator();
                if ui.button("🗑 Clear Session Logs").clicked() {
                    self.processes.lock().unwrap().clear();
                    fs::remove_file("ip_history_v2.json").ok();
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Connected Processes");
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let procs = self.processes.lock().unwrap().clone();
                    let mut proc_list: Vec<_> = procs.values().collect();
                    proc_list.sort_by(|a, b| {
                        let a_active = a.connections.values().filter(|c| c.active).count();
                        let b_active = b.connections.values().filter(|c| c.active).count();
                        b_active.cmp(&a_active).then(a.name.cmp(&b.name))
                    });

                    for proc in proc_list {
                        let active_count = proc.connections.values().filter(|c| c.active).count();
                        let total_count = proc.connections.len();

                        let header_text = format!(
                            "{} (PID: {}) - {} Active / {} Total",
                            proc.name, proc.pid, active_count, total_count
                        );

                        ui.collapsing(egui::RichText::new(header_text).strong(), |ui| {
                            TableBuilder::new(ui)
                                .striped(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .column(Column::initial(130.0).resizable(true)) // IP
                                .column(Column::initial(80.0).resizable(true)) // State
                                .column(Column::initial(130.0).resizable(true)) // First Seen
                                .column(Column::initial(130.0).resizable(true)) // Last Seen
                                .column(Column::initial(50.0).resizable(true)) // Active
                                .column(Column::remainder()) // Details
                                .header(25.0, |mut header| {
                                    header.col(|ui| {
                                        ui.strong("Foreign IP");
                                    });
                                    header.col(|ui| {
                                        ui.strong("State");
                                    });
                                    header.col(|ui| {
                                        ui.strong("First Seen");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Last Seen");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Active");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Details (Click to fetch)");
                                    });
                                })
                                .body(|mut body| {
                                    let mut conns: Vec<_> = proc.connections.values().collect();
                                    conns.sort_by(|a, b| {
                                        b.active.cmp(&a.active).then(b.last_seen.cmp(&a.last_seen))
                                    });

                                    for conn in conns {
                                        body.row(30.0, |mut row| {
                                            row.col(|ui| {
                                                if ui
                                                    .add(
                                                        egui::Label::new(&conn.foreign_ip)
                                                            .sense(egui::Sense::click()),
                                                    )
                                                    .on_hover_text("Click to copy IP")
                                                    .clicked()
                                                {
                                                    ui.output_mut(|o| {
                                                        o.copied_text = conn.foreign_ip.clone()
                                                    });
                                                }
                                            });
                                            row.col(|ui| {
                                                ui.label(&conn.state);
                                            });
                                            row.col(|ui| {
                                                ui.label(&conn.first_seen);
                                            });
                                            row.col(|ui| {
                                                ui.label(&conn.last_seen);
                                            });
                                            row.col(|ui| {
                                                if conn.active {
                                                    ui.label(
                                                        egui::RichText::new("Yes")
                                                            .color(egui::Color32::GREEN),
                                                    );
                                                } else {
                                                    ui.label(
                                                        egui::RichText::new("No")
                                                            .color(egui::Color32::RED),
                                                    );
                                                }
                                            });
                                            row.col(|ui| {
                                                let cache = self.ip_cache.lock().unwrap();
                                                let status = cache.get(&conn.foreign_ip).cloned();
                                                drop(cache);

                                                match status {
                                                    Some(IpDetailStatus::Fetched(details)) => {
                                                        ui.label(format!(
                                                            "{}, {} ({})",
                                                            details.city,
                                                            details.country,
                                                            details.isp
                                                        ));
                                                    }
                                                    Some(IpDetailStatus::Pending) => {
                                                        ui.label("⏳ Fetching...");
                                                    }
                                                    Some(IpDetailStatus::Failed) => {
                                                        if ui.button("❌ Failed (Retry)").clicked()
                                                        {
                                                            self.ip_cache
                                                                .lock()
                                                                .unwrap()
                                                                .remove(&conn.foreign_ip);
                                                            self.fetch_ip_details(
                                                                conn.foreign_ip.clone(),
                                                            );
                                                        }
                                                    }
                                                    None => {
                                                        if ui.button("🌍 Get Info").clicked() {
                                                            self.fetch_ip_details(
                                                                conn.foreign_ip.clone(),
                                                            );
                                                        }
                                                    }
                                                }
                                            });
                                        });
                                    }
                                });
                        });
                        ui.add_space(5.0);
                    }
                });
        });
    }
}

fn get_timestamp() -> String {
    let now = chrono::Utc::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn fetch_public_ip() -> Result<IpInfo, ()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|_| ())?;

    let res = client
        .get("http://ip-api.com/json/")
        .send()
        .map_err(|_| ())?
        .text()
        .map_err(|_| ())?;

    let info: IpInfo = serde_json::from_str(&res).map_err(|_| ())?;
    if info.query.is_empty() {
        return Err(());
    }
    Ok(info)
}

fn get_connected_ips(sys: &System) -> Vec<TempConn> {
    let mut conns = Vec::new();
    let is_windows = cfg!(target_os = "windows");

    let output = if is_windows {
        Command::new("netstat").arg("-ano").output()
    } else {
        Command::new("netstat").arg("-tunp").output()
    };

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if is_windows {
                if parts.len() >= 5 && parts[0] == "TCP" {
                    let foreign_ip = parts[2].split(':').next().unwrap_or("").to_string();
                    let state = parts[3].to_string();
                    let pid_str = parts[4];
                    let mut proc_name = "Unknown".to_string();
                    if let Ok(pid_val) = pid_str.parse::<sysinfo::Pid>()
                        && let Some(process) = sys.process(pid_val)
                    {
                        proc_name = process.name().to_string();
                    }
                    if !foreign_ip.is_empty()
                        && foreign_ip != "0.0.0.0"
                        && foreign_ip != "[::]"
                        && foreign_ip != "127.0.0.1"
                        && foreign_ip != "*"
                    {
                        conns.push(TempConn {
                            foreign_ip,
                            state,
                            pid: pid_str.to_string(),
                            process_name: proc_name,
                        });
                    }
                }
            } else if parts.len() >= 7 && (parts[0] == "tcp" || parts[0] == "udp") {
                let mut foreign_idx = 4;
                let mut state_idx = 5;
                let mut pid_idx = 6;

                if parts[0] == "udp" {
                    foreign_idx = 4;
                    state_idx = 5;
                    pid_idx = 5;
                }

                if parts.len() > pid_idx {
                    let foreign_ip = parts[foreign_idx]
                        .split(':')
                        .next()
                        .unwrap_or("")
                        .to_string();
                    let state = if parts[0] == "tcp" {
                        parts[state_idx].to_string()
                    } else {
                        "N/A".to_string()
                    };
                    let pid_program = parts[pid_idx];
                    let pid_str = pid_program.split('/').next().unwrap_or("");
                    let mut proc_name = "Unknown".to_string();

                    if let Ok(pid_val) = pid_str.parse::<sysinfo::Pid>() {
                        if let Some(process) = sys.process(pid_val) {
                            proc_name = process.name().to_string();
                        } else if let Some(name) = pid_program.split('/').nth(1) {
                            proc_name = name.to_string();
                        }
                    } else if let Some(name) = pid_program.split('/').nth(1) {
                        proc_name = name.to_string();
                    }

                    if !foreign_ip.is_empty()
                        && foreign_ip != "0.0.0.0"
                        && foreign_ip != "[::]"
                        && foreign_ip != "127.0.0.1"
                        && foreign_ip != "*"
                    {
                        conns.push(TempConn {
                            foreign_ip,
                            state,
                            pid: pid_str.to_string(),
                            process_name: proc_name,
                        });
                    }
                }
            }
        }
    }

    let mut unique = Vec::new();
    let mut seen = HashSet::new();
    for c in conns {
        let key = format!("{}-{}", c.foreign_ip, c.pid);
        if !seen.contains(&key) {
            seen.insert(key);
            unique.push(c);
        }
    }
    unique
}

fn save_history(procs: &HashMap<String, ProcessRecord>) {
    if let Ok(json) = serde_json::to_string_pretty(procs) {
        fs::write("ip_history_v2.json", json).ok();
    }
}

fn load_history() -> HashMap<String, ProcessRecord> {
    if let Ok(content) = fs::read_to_string("ip_history_v2.json")
        && let Ok(procs) = serde_json::from_str(&content)
    {
        return procs;
    }
    HashMap::new()
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "IP Logger Advanced",
        options,
        Box::new(|cc| {
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals = egui::Visuals::dark();
            cc.egui_ctx.set_style(style);

            Box::new(AppState::new(cc.egui_ctx.clone()))
        }),
    )
}
