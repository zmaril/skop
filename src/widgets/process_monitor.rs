use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use eframe::egui;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct Process {
    pub user: String,
    pub pid: i64,
    pub cpu_percent: f64,
    pub mem_percent: f64,
    pub command: String,
    pub state: String,
}

#[derive(Clone)]
pub struct ProcessMonitorWidget {
    pub id: usize,
    pub processes: Arc<Mutex<Vec<Process>>>,
    pub is_running: Arc<Mutex<bool>>,
    pub refresh_interval_ms: u64,
    pub sort_by: ProcessSortBy,
    pub filter_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProcessSortBy {
    CPU,
    Memory,
    PID,
    Name,
}

impl ProcessMonitorWidget {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            processes: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            refresh_interval_ms: 1000, // 1 second refresh
            sort_by: ProcessSortBy::CPU,
            filter_text: String::new(),
        }
    }
    
    pub fn execute(&self) {
        let processes = self.processes.clone();
        let is_running = self.is_running.clone();
        let refresh_interval_ms = self.refresh_interval_ms;
        
        *is_running.lock().unwrap() = true;
        
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                while *is_running.lock().unwrap() {
                    match get_processes().await {
                        Ok(proc_list) => {
                            *processes.lock().unwrap() = proc_list;
                        }
                        Err(e) => {
                            eprintln!("Error getting processes: {}", e);
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(refresh_interval_ms)).await;
                }
            });
        });
    }
    
    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
    }
    
    pub fn render(&mut self, ctx: &egui::Context, idx: usize) -> (bool, bool) {
        let is_running = *self.is_running.lock().unwrap();
        let mut open = true;
        let mut refresh_clicked = false;
        
        egui::Window::new("Process Monitor")
            .id(egui::Id::new(format!("process_widget_{}", self.id)))
            .open(&mut open)
            .default_pos([250.0 + (idx as f32 * 50.0), 100.0 + (idx as f32 * 50.0)])
            .default_size([800.0, 500.0])
            .resizable(true)
            .show(ctx, |ui| {
                // Control bar
                ui.horizontal(|ui| {
                    if is_running {
                        ui.spinner();
                        ui.label("Monitoring...");
                        if ui.button("Stop").clicked() {
                            self.stop();
                        }
                    } else {
                        ui.label("Stopped");
                        if ui.button("Start").clicked() {
                            refresh_clicked = true;
                        }
                    }
                    
                    ui.separator();
                    
                    // Sort options
                    ui.label("Sort by:");
                    if ui.selectable_label(self.sort_by == ProcessSortBy::CPU, "CPU").clicked() {
                        self.sort_by = ProcessSortBy::CPU;
                    }
                    if ui.selectable_label(self.sort_by == ProcessSortBy::Memory, "Memory").clicked() {
                        self.sort_by = ProcessSortBy::Memory;
                    }
                    if ui.selectable_label(self.sort_by == ProcessSortBy::PID, "PID").clicked() {
                        self.sort_by = ProcessSortBy::PID;
                    }
                    if ui.selectable_label(self.sort_by == ProcessSortBy::Name, "Name").clicked() {
                        self.sort_by = ProcessSortBy::Name;
                    }
                    
                    ui.separator();
                    
                    // Filter
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.filter_text);
                });
                
                ui.separator();
                
                // Process table
                let mut processes = self.processes.lock().unwrap().clone();
                
                // Apply filter
                if !self.filter_text.is_empty() {
                    processes.retain(|p| 
                        p.command.to_lowercase().contains(&self.filter_text.to_lowercase()) ||
                        p.user.to_lowercase().contains(&self.filter_text.to_lowercase())
                    );
                }
                
                // Sort processes
                match self.sort_by {
                    ProcessSortBy::CPU => processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap()),
                    ProcessSortBy::Memory => processes.sort_by(|a, b| b.mem_percent.partial_cmp(&a.mem_percent).unwrap()),
                    ProcessSortBy::PID => processes.sort_by(|a, b| a.pid.cmp(&b.pid)),
                    ProcessSortBy::Name => processes.sort_by(|a, b| a.command.cmp(&b.command)),
                }
                
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        use egui_extras::{TableBuilder, Column};
                        
                        TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::initial(60.0).resizable(true))  // PID
                            .column(Column::initial(80.0).resizable(true))  // User
                            .column(Column::initial(60.0).resizable(true))  // CPU %
                            .column(Column::initial(60.0).resizable(true))  // Mem %
                            .column(Column::initial(50.0).resizable(true))  // State
                            .column(Column::remainder())                    // Command
                            .header(20.0, |mut header| {
                                header.col(|ui| { ui.strong("PID"); });
                                header.col(|ui| { ui.strong("User"); });
                                header.col(|ui| { ui.strong("CPU %"); });
                                header.col(|ui| { ui.strong("Mem %"); });
                                header.col(|ui| { ui.strong("State"); });
                                header.col(|ui| { ui.strong("Command"); });
                            })
                            .body(|mut body| {
                                for process in processes.iter().take(100) { // Limit to 100 for performance
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| {
                                            ui.label(process.pid.to_string());
                                        });
                                        row.col(|ui| {
                                            ui.label(&process.user);
                                        });
                                        row.col(|ui| {
                                            let color = if process.cpu_percent > 50.0 {
                                                egui::Color32::RED
                                            } else if process.cpu_percent > 20.0 {
                                                egui::Color32::YELLOW
                                            } else {
                                                egui::Color32::GREEN
                                            };
                                            ui.colored_label(color, format!("{:.1}", process.cpu_percent));
                                        });
                                        row.col(|ui| {
                                            let color = if process.mem_percent > 10.0 {
                                                egui::Color32::RED
                                            } else if process.mem_percent > 5.0 {
                                                egui::Color32::YELLOW
                                            } else {
                                                egui::Color32::GREEN
                                            };
                                            ui.colored_label(color, format!("{:.1}", process.mem_percent));
                                        });
                                        row.col(|ui| {
                                            ui.label(&process.state);
                                        });
                                        row.col(|ui| {
                                            ui.label(&process.command);
                                        });
                                    });
                                }
                            });
                    });
            });
        
        (open, refresh_clicked)
    }
}

async fn get_processes() -> Result<Vec<Process>, Box<dyn std::error::Error>> {
    use tokio::process::Command;
    
    let output = Command::new("sh")
        .arg("-c")
        .arg("ps aux | jc --ps")
        .output()
        .await?;
    
    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: Vec<Value> = serde_json::from_str(&json_str)?;
    
    let mut processes = Vec::new();
    for item in json {
        if let Value::Object(map) = item {
            let process = Process {
                user: map.get("user").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                pid: map.get("pid").and_then(|v| v.as_i64()).unwrap_or(0),
                cpu_percent: map.get("cpu_percent").and_then(|v| v.as_f64()).unwrap_or(0.0),
                mem_percent: map.get("mem_percent").and_then(|v| v.as_f64()).unwrap_or(0.0),
                command: map.get("command").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                state: map.get("stat").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            };
            processes.push(process);
        }
    }
    
    Ok(processes)
}