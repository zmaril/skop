use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};

// Core execution modes
#[derive(Debug, Clone)]
pub enum ExecutionMode {
    OneShot,                    // Run once and exit
    Continuous,                 // Run continuously (like vmstat)
    Periodic(Duration),         // Run periodically with interval
}

// Command builder struct
#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

impl CommandSpec {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }
    
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }
    
    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
}

// Core command executor that all widgets will use  
#[derive(Clone, Default)]
pub struct CommandExecutor {
    pub output: Arc<Mutex<Vec<String>>>,
    pub is_running: Arc<Mutex<bool>>,
    pub database: Option<Arc<crate::database::investigation_db::InvestigationDB>>,
    pub widget_id: Option<i32>,
    pub widget_version: Option<i32>,
    pub max_lines: usize,  // Limit output buffer size
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            output: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            database: None,
            widget_id: None,
            widget_version: None,
            max_lines: 1000,
        }
    }
    
    pub fn with_max_lines(mut self, max: usize) -> Self {
        self.max_lines = max;
        self
    }
    
    pub fn set_database(&mut self, database: Option<Arc<crate::database::investigation_db::InvestigationDB>>, widget_id: i32, widget_version: i32) {
        self.database = database;
        self.widget_id = Some(widget_id);
        self.widget_version = Some(widget_version);
    }
    
    pub fn clear_output(&self) {
        self.output.lock().unwrap().clear();
    }
    
    pub fn load_historical_output(&self, lines: Vec<String>) {
        let mut output = self.output.lock().unwrap();
        output.clear(); // Clear any existing output
        output.extend(lines); // Add all historical lines
        
        // Respect max_lines limit
        if output.len() > self.max_lines {
            let excess = output.len() - self.max_lines;
            output.drain(0..excess);
        }
    }
    
    pub fn add_output(&self, line: String, line_number: i32) {
        // Add to output buffer for UI
        {
            let mut output = self.output.lock().unwrap();
            output.push(line.clone());
            // Keep buffer size limited
            if output.len() > self.max_lines {
                let excess = output.len() - self.max_lines;
                output.drain(0..excess);
            }
        }
        
        // Log to database if available
        if let (Some(db), Some(widget_id), Some(widget_version)) = 
            (&self.database, &self.widget_id, &self.widget_version) {
            let db_clone = db.clone();
            let widget_id = *widget_id;
            let widget_version = *widget_version;
            let line_clone = line.clone();
            
            tokio::spawn(async move {
                if let Err(e) = db_clone.record_raw_data(widget_id, widget_version, &line_clone, line_number).await {
                    eprintln!("Failed to record raw data: {}", e);
                }
            });
        }
    }
    
    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
    }
    
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }
    
    // Execute command once
    pub fn run_once(&self, spec: CommandSpec) {
        if self.is_running() {
            return;
        }
        
        let executor = self.clone();
        *self.is_running.lock().unwrap() = true;
        
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                executor.execute_command(spec, false).await;
            });
        });
    }
    
    // Execute command continuously (like vmstat 2)
    pub fn run_continuous(&self, spec: CommandSpec) {
        if self.is_running() {
            return;
        }
        
        let executor = self.clone();
        *self.is_running.lock().unwrap() = true;
        
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                executor.execute_command(spec, true).await;
            });
        });
    }
    
    // Execute command periodically
    pub fn run_periodic(&self, spec: CommandSpec, interval: Duration) {
        if self.is_running() {
            return;
        }
        
        let executor = self.clone();
        *self.is_running.lock().unwrap() = true;
        
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                while executor.is_running() {
                    // Clear output for each periodic run
                    executor.clear_output();
                    
                    executor.execute_command(spec.clone(), false).await;
                    
                    // Wait for interval
                    tokio::time::sleep(interval).await;
                }
            });
        });
    }
    
    async fn execute_command(&self, spec: CommandSpec, continuous: bool) {
        let mut cmd = Command::new(&spec.program);
        for arg in &spec.args {
            cmd.arg(arg);
        }
        
        match cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn() {
            Ok(mut child) => {
                let stdout = child.stdout.take().unwrap();
                let mut reader = BufReader::new(stdout).lines();
                let mut line_number = 1i32;
                
                while self.is_running() {
                    match reader.next_line().await {
                        Ok(Some(line)) => {
                            self.add_output(line, line_number);
                            line_number += 1;
                        }
                        Ok(None) => {
                            // Process ended
                            if !continuous {
                                self.add_output("Command completed".to_string(), line_number);
                            }
                            break;
                        }
                        Err(e) => {
                            self.add_output(format!("Error reading output: {}", e), line_number);
                            break;
                        }
                    }
                }
                
                let _ = child.kill().await;
            }
            Err(e) => {
                self.add_output(format!("Failed to execute command: {}", e), 0);
            }
        }
        
        *self.is_running.lock().unwrap() = false;
    }
}

// Main trait that command widgets implement
pub trait CommandWidget: crate::widgets::Widget {
    // Required: build the command to execute
    fn build_command(&self) -> CommandSpec;
    
    // Required: get the executor
    fn executor(&self) -> &CommandExecutor;
    fn executor_mut(&mut self) -> &mut CommandExecutor;
    
    // Optional: execution mode
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::OneShot
    }
    
    // Provided: standard start implementation
    fn start_command(&self) {
        let spec = self.build_command();
        
        match self.execution_mode() {
            ExecutionMode::OneShot => {
                self.executor().run_once(spec);
            }
            ExecutionMode::Continuous => {
                self.executor().run_continuous(spec);
            }
            ExecutionMode::Periodic(interval) => {
                self.executor().run_periodic(spec, interval);
            }
        }
    }
    
    // Provided: standard stop
    fn stop_command(&self) {
        self.executor().stop();
    }
    
    
    // Provided: handle config changes with immediate versioning and optional restart
    // Note: Database save will be handled by the specific widget implementation
    fn handle_config_change(&mut self, database: Option<std::sync::Arc<crate::database::investigation_db::InvestigationDB>>) {
        let needs_restart = self.needs_restart();
        
        // Stop if restart needed
        if needs_restart {
            self.stop();
        }
        
        // Increment version (automatically marks config as saved)
        self.increment_version();
        
        // Update database connection with new version
        if let Some(ref db) = database {
            let widget_id = self.widget_id() as i32;
            let widget_version = self.widget_version();
            self.executor_mut().set_database(Some(db.clone()), widget_id, widget_version);
        }
        
        // Restart if needed
        if needs_restart {
            self.start();
        }
    }
}

// Trait for widgets with configurable refresh intervals
pub trait RefreshableWidget {
    fn refresh_interval(&self) -> Duration;
    fn set_refresh_interval(&mut self, interval: Duration);
}

// Trait for widgets that filter output
pub trait FilterableOutput {
    fn filter_pattern(&self) -> &str;
    fn set_filter_pattern(&mut self, pattern: String);
    fn matches_filter(&self, line: &str) -> bool {
        if self.filter_pattern().is_empty() {
            true
        } else {
            line.to_lowercase().contains(&self.filter_pattern().to_lowercase())
        }
    }
}

// UI rendering traits
pub trait CommandOutputRenderer {
    fn executor(&self) -> &CommandExecutor;
    
    fn render_output(&self, ui: &mut eframe::egui::Ui) {
        use eframe::egui;
        
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let output = self.executor().output.lock().unwrap();
                for line in output.iter() {
                    ui.label(egui::RichText::new(line).monospace().size(12.0));
                }
            });
    }
}

pub trait CommandControlBar: CommandWidget {
    fn render_controls(&mut self, ui: &mut eframe::egui::Ui) -> bool {
        let mut refresh_clicked = false;
        let is_running = self.executor().is_running();
        
        if is_running {
            ui.spinner();
            ui.label("Running...");
            if ui.button("Stop").clicked() {
                self.stop_command();
            }
        } else {
            ui.label("Stopped");
            if ui.button("Start").clicked() {
                self.start_command();
                refresh_clicked = true;
            }
        }
        
        if ui.button("Clear").clicked() {
            self.executor().clear_output();
        }
        
        refresh_clicked
    }
}