use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use eframe::egui;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SystemInfoWidget {
    pub id: usize,
    pub info_type: String,
    pub auto_scroll: bool,
    pub needs_config: bool,
    #[serde(skip, default = "default_output")]
    pub output: Arc<Mutex<Vec<String>>>,
    #[serde(skip, default = "default_running")]
    pub is_running: Arc<Mutex<bool>>,
}

fn default_output() -> Arc<Mutex<Vec<String>>> {
    Arc::new(Mutex::new(Vec::new()))
}

fn default_running() -> Arc<Mutex<bool>> {
    Arc::new(Mutex::new(false))
}

impl crate::widgets::Widget for SystemInfoWidget {
    fn widget_type_name(&self) -> &'static str {
        "system_info"
    }
    
    fn widget_id(&self) -> usize {
        self.id
    }
    
    
    fn start(&self) {
        let output = self.output.clone();
        let is_running = self.is_running.clone();
        let info_type = self.info_type.clone();
        
        *is_running.lock().unwrap() = true;
        
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let command = match info_type.as_str() {
                    "hardware" => "system_profiler SPHardwareDataType | grep -E '(Processor|Cores|Threads|Memory)'",
                    "activity" => "top -l 1 -o cpu -n 10",
                    _ => "uname -a && sw_vers",
                };
                match crate::widgets::ssh_command::run_local_command(command, output.clone()).await {
                    Ok(_) => {
                        output.lock().unwrap().push("System info completed".to_string());
                    }
                    Err(e) => {
                        output.lock().unwrap().push(format!("Error: {}", e));
                    }
                }
                *is_running.lock().unwrap() = false;
            });
        });
    }
    
    fn render(&mut self, ctx: &egui::Context, idx: usize) -> (bool, bool) {
        let mut open = true;
        let mut refresh_clicked = false;
        let mut cancel_clicked = false;
        
        if self.needs_config {
            // Configuration mode
            egui::Window::new("System Info Configuration")
                .id(egui::Id::new(format!("sysinfo_config_{}", self.id)))
                .open(&mut open)
                .default_pos([400.0 + (idx as f32 * 30.0), 200.0 + (idx as f32 * 30.0)])
                .default_size([300.0, 150.0])
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label("Choose system information type:");
                    ui.separator();
                    
                    ui.vertical(|ui| {
                        if ui.button("Hardware Info").clicked() {
                            self.info_type = "hardware".to_string();
                            self.needs_config = false;
                            refresh_clicked = true;
                        }
                        
                        if ui.button("Activity Monitor").clicked() {
                            self.info_type = "activity".to_string();
                            self.needs_config = false;
                            refresh_clicked = true;
                        }
                        
                        if ui.button("System Overview").clicked() {
                            self.info_type = "overview".to_string();
                            self.needs_config = false;
                            refresh_clicked = true;
                        }
                        
                        ui.separator();
                        
                        if ui.button("Cancel").clicked() {
                            cancel_clicked = true;
                        }
                    });
                });
        } else {
            // Execution mode - use the existing render logic
            let is_running = *self.is_running.lock().unwrap();
            
            let title = match self.info_type.as_str() {
                "hardware" => "System Info",
                "activity" => "Activity Monitor",
                _ => "System",
            };
            
            egui::Window::new(title)
                .id(egui::Id::new(format!("info_widget_{}", self.id)))
                .open(&mut open)
                .default_pos([250.0 + (idx as f32 * 50.0), 100.0 + (idx as f32 * 50.0)])
                .default_size([600.0, 400.0])
                .resizable(true)
                .show(ctx, |ui| {
                    if is_running {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Loading...");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                            });
                        });
                        ui.separator();
                    } else {
                        let output = self.output.lock().unwrap();
                        let has_output = !output.is_empty();
                        drop(output);
                        
                        if has_output {
                            ui.horizontal(|ui| {
                                ui.label("Completed");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                                    if ui.button("Refresh").clicked() {
                                        refresh_clicked = true;
                                    }
                                });
                            });
                            ui.separator();
                        } else {
                            ui.horizontal(|ui| {
                                ui.label("Ready");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                                    if ui.button("Execute").clicked() {
                                        refresh_clicked = true;
                                    }
                                });
                            });
                            ui.separator();
                        }
                    }
                    
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(self.auto_scroll)
                        .id_salt(format!("info_scroll_{}", self.id))
                        .show(ui, |ui| {
                            let output = self.output.lock().unwrap();
                            if output.is_empty() && !is_running {
                                ui.label("Click 'Execute' to get system information");
                            } else {
                                for line in output.iter() {
                                    ui.label(egui::RichText::new(line).monospace().size(14.0));
                                }
                            }
                        });
                });
        }
        
        if cancel_clicked {
            open = false;
        }
        
        (open, refresh_clicked)
    }
}

impl SystemInfoWidget {
    pub fn new(id: usize, info_type: String) -> Self {
        Self {
            id,
            output: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            auto_scroll: true,
            info_type,
            needs_config: false,
        }
    }
    
    pub fn new_with_config(id: usize) -> Self {
        Self {
            id,
            output: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            auto_scroll: true,
            info_type: String::new(), // Will be set through configuration
            needs_config: true,
        }
    }
    
    
    
}