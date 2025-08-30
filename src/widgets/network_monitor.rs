use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use eframe::egui;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct NetworkConnection {
    pub protocol: String,
    pub local_address: String,
    pub local_port: i64,
    pub foreign_address: String,
    pub foreign_port: i64,
    pub state: String,
}

#[derive(Clone)]
pub struct NetworkMonitorWidget {
    pub id: usize,
    pub connections: Arc<Mutex<Vec<NetworkConnection>>>,
    pub is_running: Arc<Mutex<bool>>,
    pub refresh_interval_ms: u64,
    pub filter_text: String,
    pub show_established_only: bool,
}

impl NetworkMonitorWidget {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            connections: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            refresh_interval_ms: 2000, // 2 second refresh
            filter_text: String::new(),
            show_established_only: false,
        }
    }
    
    pub fn execute(&self) {
        let connections = self.connections.clone();
        let is_running = self.is_running.clone();
        let refresh_interval_ms = self.refresh_interval_ms;
        
        *is_running.lock().unwrap() = true;
        
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                while *is_running.lock().unwrap() {
                    match get_network_connections().await {
                        Ok(conn_list) => {
                            *connections.lock().unwrap() = conn_list;
                        }
                        Err(e) => {
                            eprintln!("Error getting network connections: {}", e);
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
        
        egui::Window::new("Network Connections")
            .id(egui::Id::new(format!("network_widget_{}", self.id)))
            .open(&mut open)
            .default_pos([300.0 + (idx as f32 * 50.0), 150.0 + (idx as f32 * 50.0)])
            .default_size([700.0, 400.0])
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
                    
                    ui.checkbox(&mut self.show_established_only, "Established Only");
                    
                    ui.separator();
                    
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut self.filter_text);
                });
                
                ui.separator();
                
                // Connection table
                let mut connections = self.connections.lock().unwrap().clone();
                
                // Apply filters
                if self.show_established_only {
                    connections.retain(|c| c.state == "ESTABLISHED");
                }
                
                if !self.filter_text.is_empty() {
                    connections.retain(|c| 
                        c.foreign_address.contains(&self.filter_text) ||
                        c.local_address.contains(&self.filter_text) ||
                        c.protocol.contains(&self.filter_text) ||
                        c.foreign_port.to_string().contains(&self.filter_text) ||
                        c.local_port.to_string().contains(&self.filter_text)
                    );
                }
                
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        use egui_extras::{TableBuilder, Column};
                        
                        TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::initial(50.0).resizable(true))   // Proto
                            .column(Column::initial(150.0).resizable(true))  // Local
                            .column(Column::initial(150.0).resizable(true))  // Foreign
                            .column(Column::initial(100.0).resizable(true))  // State
                            .header(20.0, |mut header| {
                                header.col(|ui| { ui.strong("Proto"); });
                                header.col(|ui| { ui.strong("Local Address"); });
                                header.col(|ui| { ui.strong("Foreign Address"); });
                                header.col(|ui| { ui.strong("State"); });
                            })
                            .body(|mut body| {
                                for conn in connections.iter().take(200) {
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| {
                                            ui.label(&conn.protocol);
                                        });
                                        row.col(|ui| {
                                            ui.label(format!("{}:{}", conn.local_address, conn.local_port));
                                        });
                                        row.col(|ui| {
                                            let label = if conn.foreign_address == "*" {
                                                "*:*".to_string()
                                            } else {
                                                format!("{}:{}", conn.foreign_address, conn.foreign_port)
                                            };
                                            ui.label(label);
                                        });
                                        row.col(|ui| {
                                            let color = match conn.state.as_str() {
                                                "ESTABLISHED" => egui::Color32::GREEN,
                                                "LISTEN" => egui::Color32::LIGHT_BLUE,
                                                "TIME_WAIT" => egui::Color32::YELLOW,
                                                "CLOSE_WAIT" => egui::Color32::from_rgb(255, 165, 0),
                                                _ => egui::Color32::GRAY,
                                            };
                                            ui.colored_label(color, &conn.state);
                                        });
                                    });
                                }
                            });
                    });
                
                ui.separator();
                ui.label(format!("Total connections: {}", connections.len()));
            });
        
        (open, refresh_clicked)
    }
}

async fn get_network_connections() -> Result<Vec<NetworkConnection>, Box<dyn std::error::Error>> {
    use tokio::process::Command;
    
    let output = Command::new("sh")
        .arg("-c")
        .arg("netstat -an | jc --netstat")
        .output()
        .await?;
    
    let json_str = String::from_utf8_lossy(&output.stdout);
    let json: Vec<Value> = serde_json::from_str(&json_str)?;
    
    let mut connections = Vec::new();
    for item in json {
        if let Value::Object(map) = item {
            // Only process network connections (not unix sockets)
            if let Some(kind) = map.get("kind").and_then(|v| v.as_str()) {
                if kind != "network" {
                    continue;
                }
            }
            
            let connection = NetworkConnection {
                protocol: map.get("proto").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                local_address: map.get("local_address").and_then(|v| v.as_str()).unwrap_or("*").to_string(),
                local_port: map.get("local_port_num").and_then(|v| v.as_i64()).unwrap_or(0),
                foreign_address: map.get("foreign_address").and_then(|v| v.as_str()).unwrap_or("*").to_string(),
                foreign_port: map.get("foreign_port_num").and_then(|v| v.as_i64()).unwrap_or(0),
                state: map.get("state").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            };
            connections.push(connection);
        }
    }
    
    Ok(connections)
}