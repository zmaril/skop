use std::time::Duration;
use eframe::egui;
use serde::{Serialize, Deserialize};
use crate::widgets::command_widget::{CommandExecutor, CommandSpec, CommandWidget, ExecutionMode, CommandOutputRenderer, CommandControlBar};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessSortBy {
    CPU,
    Memory,
    PID,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ProcessMonitorWidget {
    pub id: usize,
    pub version: i32,
    pub refresh_interval_secs: u64,
    pub max_processes: usize,
    pub sort_by: ProcessSortBy,
    #[serde(skip, default = "default_executor")]
    pub executor: CommandExecutor,
    #[serde(skip, default)]
    pub config_unsaved: bool,
}

fn default_executor() -> CommandExecutor {
    CommandExecutor::new()
}

impl crate::widgets::Widget for ProcessMonitorWidget {
    fn widget_type_name(&self) -> &'static str {
        "process_monitor"
    }
    
    fn widget_id(&self) -> usize {
        self.id
    }
    
    fn widget_version(&self) -> i32 {
        self.version
    }
    
    fn increment_version(&mut self) {
        self.version += 1;
        self.config_unsaved = false;
    }
    
    fn set_database(&mut self, database: Option<std::sync::Arc<crate::database::investigation_db::InvestigationDB>>) {
        let widget_id = self.id as i32;
        let widget_version = self.version;
        self.executor.set_database(database, widget_id, widget_version);
    }
    
    fn config_changed(&self) -> bool {
        self.config_unsaved
    }
    
    
    fn needs_restart(&self) -> bool {
        // Process monitor needs restart for execution-affecting changes (sort, max processes, interval)
        self.config_unsaved
    }
    
    fn start(&self) {
        self.start_command();
    }
    
    fn stop(&self) {
        self.stop_command();
    }
    
    fn render(&mut self, ctx: &egui::Context, idx: usize) -> (bool, bool) {
        let mut open = true;
        let mut refresh_clicked = false;
        
        egui::Window::new("Process Monitor")
            .id(egui::Id::new(format!("process_monitor_{}", self.id)))
            .open(&mut open)
            .default_pos([150.0 + (idx as f32 * 50.0), 150.0 + (idx as f32 * 50.0)])
            .default_size([900.0, 600.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    refresh_clicked = self.render_controls(ui);
                    
                    ui.separator();
                    ui.label("Sort by:");
                    let old_sort = self.sort_by.clone();
                    ui.selectable_value(&mut self.sort_by, ProcessSortBy::CPU, "CPU");
                    ui.selectable_value(&mut self.sort_by, ProcessSortBy::Memory, "Memory");
                    ui.selectable_value(&mut self.sort_by, ProcessSortBy::PID, "PID");
                    if old_sort != self.sort_by {
                        self.config_unsaved = true;
                        // Restart with new sort if running
                        if self.executor.is_running() {
                            self.stop();
                            self.start();
                        }
                    }
                    
                    ui.separator();
                    ui.label("Max:");
                    let old_max = self.max_processes;
                    ui.add(egui::DragValue::new(&mut self.max_processes).range(5..=100));
                    if old_max != self.max_processes {
                        self.config_unsaved = true;
                        if self.executor.is_running() {
                            self.stop();
                            self.start();
                        }
                    }
                    
                    ui.separator();
                    ui.label("Interval:");
                    let old_interval = self.refresh_interval_secs;
                    ui.add(egui::DragValue::new(&mut self.refresh_interval_secs).range(1..=60).suffix("s"));
                    if old_interval != self.refresh_interval_secs {
                        self.config_unsaved = true;
                        if self.executor.is_running() {
                            self.stop();
                            self.start();
                        }
                    }
                });
                
                ui.separator();
                self.render_output(ui);
            });
        
        (open, refresh_clicked)
    }
    
    fn refresh(&self) {
        self.stop();
        self.start();
    }
    
    fn restore_widget_data(&mut self, data: Vec<String>) {
        self.executor.load_historical_output(data);
    }
}

impl CommandWidget for ProcessMonitorWidget {
    fn build_command(&self) -> CommandSpec {
        // Use ps aux which jc supports
        CommandSpec::new("ps")
            .arg("aux")
    }
    
    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
    
    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Periodic(Duration::from_secs(self.refresh_interval_secs))
    }
}

impl CommandOutputRenderer for ProcessMonitorWidget {
    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
    
    fn render_output(&self, ui: &mut eframe::egui::Ui) {
        use eframe::egui;
        use serde_json::Value;
        
        let raw_output = CommandWidget::executor(self).output.lock().unwrap();
        
        if raw_output.is_empty() {
            ui.label("No data available");
            return;
        }
        
        let raw_text = raw_output.join("\n");
        
        // Process through jc --ps to get JSON
        match std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("echo '{}' | jc --ps -q 2>/dev/null", raw_text.replace("'", "'\\''")))
            .output()
        {
            Ok(output) if !output.stdout.is_empty() => {
                // Parse JSON and render as egui table
                match serde_json::from_slice::<Value>(&output.stdout) {
                    Ok(json) => {
                        if let Some(processes) = json.as_array() {
                            // Render as egui table
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                    egui::Grid::new("process_table")
                                        .num_columns(4)
                                        .spacing([20.0, 4.0])
                                        .striped(true)
                                        .show(ui, |ui| {
                                            // Header
                                            ui.strong("PID");
                                            ui.strong("COMMAND"); 
                                            ui.strong("CPU%");
                                            ui.strong("MEMORY");
                                            ui.end_row();
                                            
                                            // Process rows - sort by the selected field
                                            let mut process_list: Vec<&Value> = processes.iter().collect();
                                            match self.sort_by {
                                                ProcessSortBy::CPU => {
                                                    process_list.sort_by(|a, b| {
                                                        let a_cpu = a.get("cpu_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                                        let b_cpu = b.get("cpu_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                                        b_cpu.partial_cmp(&a_cpu).unwrap_or(std::cmp::Ordering::Equal)
                                                    });
                                                }
                                                ProcessSortBy::Memory => {
                                                    process_list.sort_by(|a, b| {
                                                        let a_mem = a.get("mem_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                                        let b_mem = b.get("mem_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                                        b_mem.partial_cmp(&a_mem).unwrap_or(std::cmp::Ordering::Equal)
                                                    });
                                                }
                                                ProcessSortBy::PID => {
                                                    process_list.sort_by(|a, b| {
                                                        let a_pid = a.get("pid").and_then(|v| v.as_i64()).unwrap_or(0);
                                                        let b_pid = b.get("pid").and_then(|v| v.as_i64()).unwrap_or(0);
                                                        a_pid.cmp(&b_pid)
                                                    });
                                                }
                                            }
                                            
                                            for process in process_list.iter().take(self.max_processes) {
                                                let pid = process.get("pid")
                                                    .and_then(|v| v.as_i64())
                                                    .map(|v| v.to_string())
                                                    .unwrap_or_else(|| "N/A".to_string());
                                                
                                                let command = process.get("command")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("N/A")
                                                    .split_whitespace()
                                                    .next()
                                                    .unwrap_or("N/A");
                                                
                                                let cpu = process.get("cpu_percent")
                                                    .and_then(|v| v.as_f64())
                                                    .map(|v| format!("{:.1}%", v))
                                                    .unwrap_or_else(|| "N/A".to_string());
                                                
                                                let memory = process.get("mem_percent")
                                                    .and_then(|v| v.as_f64())
                                                    .map(|v| format!("{:.1}%", v))
                                                    .unwrap_or_else(|| "N/A".to_string());
                                                
                                                ui.monospace(&pid);
                                                ui.monospace(command);
                                                ui.monospace(&cpu);
                                                ui.monospace(memory);
                                                ui.end_row();
                                            }
                                        });
                                });
                        } else {
                            // JSON doesn't have expected structure - fail
                            ui.label("❌ jc failed to parse ps output - invalid JSON structure");
                            ui.separator();
                            ui.label("Raw ps output for debugging:");
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    for line in raw_text.lines() {
                                        ui.label(egui::RichText::new(line).monospace().size(10.0));
                                    }
                                });
                        }
                    }
                    Err(e) => {
                        // JSON parsing failed - fail
                        ui.label(format!("❌ JSON parsing failed: {}", e));
                        ui.separator();
                        ui.label("Raw jc output for debugging:");
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                let jc_output = String::from_utf8_lossy(&output.stdout);
                                for line in jc_output.lines() {
                                    ui.label(egui::RichText::new(line).monospace().size(10.0));
                                }
                            });
                    }
                }
            }
            _ => {
                // jc command failed - fail
                ui.label("❌ jc --ps command failed or not supported on this platform");
                ui.separator();
                ui.label("Raw ps output for debugging:");
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for line in raw_text.lines() {
                            ui.label(egui::RichText::new(line).monospace().size(10.0));
                        }
                    });
            }
        }
    }
}

impl CommandControlBar for ProcessMonitorWidget {}


impl ProcessMonitorWidget {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            version: 0,
            refresh_interval_secs: 5,
            max_processes: 20,
            sort_by: ProcessSortBy::CPU,
            executor: CommandExecutor::new(),
            config_unsaved: false,
        }
    }
}