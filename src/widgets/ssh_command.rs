use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use openssh::{SessionBuilder, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use eframe::egui;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SSHCommandWidget {
    pub id: usize,
    pub hostname: String,
    pub command: String,
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

impl crate::widgets::Widget for SSHCommandWidget {
    fn widget_type_name(&self) -> &'static str {
        "ssh_command"
    }
    
    fn widget_id(&self) -> usize {
        self.id
    }
    
    
    fn start(&self) {
        let output = self.output.clone();
        let is_running = self.is_running.clone();
        let hostname = self.hostname.clone();
        let command = self.command.clone();
        
        *is_running.lock().unwrap() = true;
        
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                match run_ssh_command(&hostname, &command, output.clone()).await {
                    Ok(_) => {
                        output.lock().unwrap().push(format!("Command completed"));
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
        let mut execute_clicked = false;
        let mut cancel_clicked = false;
        
        if self.needs_config {
            // Configuration mode
            egui::Window::new("SSH Configuration")
                .id(egui::Id::new(format!("ssh_config_{}", self.id)))
                .open(&mut open)
                .default_pos([400.0 + (idx as f32 * 30.0), 200.0 + (idx as f32 * 30.0)])
                .default_size([400.0, 200.0])
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label("Configure SSH Command:");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Host:");
                        ui.text_edit_singleline(&mut self.hostname);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Command:");
                        ui.text_edit_singleline(&mut self.command);
                    });
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("Execute Command").clicked() && !self.hostname.is_empty() && !self.command.is_empty() {
                            self.needs_config = false;
                            execute_clicked = true;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            cancel_clicked = true;
                        }
                    });
                });
        } else {
            // Execution mode
            let is_running = *self.is_running.lock().unwrap();
            
            egui::Window::new(format!("{}@{}", self.command, self.hostname))
                .id(egui::Id::new(format!("ssh_widget_{}", self.id)))
                .open(&mut open)
                .default_pos([250.0 + (idx as f32 * 50.0), 100.0 + (idx as f32 * 50.0)])
                .default_size([600.0, 400.0])
                .resizable(true)
                .show(ctx, |ui| {
                    if is_running {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Running...");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                            });
                        });
                        ui.separator();
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("Completed");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                            });
                        });
                        ui.separator();
                    }
                    
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .stick_to_bottom(self.auto_scroll)
                        .id_salt(format!("scroll_area_{}", self.id))
                        .show(ui, |ui| {
                            let output = self.output.lock().unwrap();
                            for line in output.iter() {
                                ui.label(egui::RichText::new(line).monospace().size(14.0));
                            }
                        });
                });
        }
        
        if execute_clicked {
            self.start();
        }
        
        if cancel_clicked {
            open = false;
        }
        
        (open, false) // SSH commands don't need refresh functionality
    }
}

impl SSHCommandWidget {
    pub fn new(id: usize, hostname: String, command: String) -> Self {
        Self {
            id,
            hostname,
            command,
            auto_scroll: true,
            needs_config: false,
            output: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn new_with_config(id: usize) -> Self {
        Self {
            id,
            hostname: "localhost".to_string(),
            command: String::new(),
            auto_scroll: true,
            needs_config: true,
            output: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
        }
    }
    
    
    
}

async fn run_ssh_command(
    hostname: &str,
    command: &str,
    output: Arc<Mutex<Vec<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if hostname == "localhost" || hostname == "127.0.0.1" {
        run_local_command(command, output).await
    } else {
        run_remote_command(hostname, command, output).await
    }
}

pub async fn run_local_command(
    command: &str,
    output: Arc<Mutex<Vec<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    output.lock().unwrap().push("Running locally...".to_string());
    output.lock().unwrap().push(format!("Executing: {}", command));
    output.lock().unwrap().push("─".repeat(40));
    
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        output.lock().unwrap().push("Error: Empty command".to_string());
        return Ok(());
    }
    
    let mut child = Command::new(parts[0])
        .args(&parts[1..])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let output_clone = output.clone();
    let stdout_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            output_clone.lock().unwrap().push(line);
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });
    
    let output_clone = output.clone();
    let stderr_task = tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            output_clone.lock().unwrap().push(format!("Error: {}", line));
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });
    
    let _ = tokio::try_join!(stdout_task, stderr_task);
    
    let status = child.wait().await?;
    output.lock().unwrap().push("─".repeat(40));
    output.lock().unwrap().push(format!("Exit status: {:?}", status));
    
    Ok(())
}

async fn run_remote_command(
    hostname: &str,
    command: &str,
    output: Arc<Mutex<Vec<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    output.lock().unwrap().push(format!("Connecting to {}...", hostname));
    
    let session = SessionBuilder::default()
        .connect(hostname)
        .await?;
    
    output.lock().unwrap().push(format!("Connected to {}", hostname));
    output.lock().unwrap().push(format!("Executing: {}", command));
    output.lock().unwrap().push("─".repeat(40));
    
    let mut child = session
        .command(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .await?;
    
    let stdout = child.stdout().take().unwrap();
    let stderr = child.stderr().take().unwrap();
    
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    tokio::select! {
        _ = async {
            while let Ok(Some(line)) = stdout_reader.next_line().await {
                output.lock().unwrap().push(line);
            }
        } => {},
        _ = async {
            while let Ok(Some(line)) = stderr_reader.next_line().await {
                output.lock().unwrap().push(format!("Error: {}", line));
            }
        } => {},
    }
    
    let status = child.wait().await?;
    output.lock().unwrap().push("─".repeat(40));
    output.lock().unwrap().push(format!("Exit status: {:?}", status));
    
    Ok(())
}