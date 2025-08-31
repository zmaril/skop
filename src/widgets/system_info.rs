use eframe::egui;
use serde::{Serialize, Deserialize};
use crate::widgets::command_widget::{CommandExecutor, CommandSpec, CommandWidget, ExecutionMode, CommandOutputRenderer, CommandControlBar};

#[derive(Clone, Serialize, Deserialize)]
pub struct SystemInfoWidget {
    pub id: usize,
    pub version: i32,
    pub info_type: String,
    pub needs_config: bool,
    #[serde(skip, default = "default_executor")]
    pub executor: CommandExecutor,
    #[serde(skip, default)]
    pub config_unsaved: bool,
}

fn default_executor() -> CommandExecutor {
    CommandExecutor::new()
}

impl crate::widgets::Widget for SystemInfoWidget {
    fn widget_type_name(&self) -> &'static str {
        "system_info"
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
        // System info needs restart when info type changes (different commands)
        self.config_unsaved
    }
    
    fn restore_widget_data(&mut self, data: Vec<String>) {
        self.executor.load_historical_output(data);
    }
    
    fn set_available_hosts(&mut self, hosts: Vec<crate::database::investigation_db::Host>) {
        self.executor.set_available_hosts(hosts);
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
        
        if self.needs_config {
            // Configuration mode
            egui::Window::new("System Info Configuration")
                .id(egui::Id::new(format!("system_config_{}", self.id)))
                .open(&mut open)
                .default_pos([400.0 + (idx as f32 * 30.0), 200.0 + (idx as f32 * 30.0)])
                .default_size([400.0, 200.0])
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label("Select System Information Type:");
                    ui.separator();
                    
                    let old_type = self.info_type.clone();
                    ui.radio_value(&mut self.info_type, "hardware".to_string(), "Hardware Information (system_profiler)");
                    ui.radio_value(&mut self.info_type, "activity".to_string(), "Activity Monitor (top)");
                    ui.radio_value(&mut self.info_type, "overview".to_string(), "System Overview (uname)");
                    
                    if old_type != self.info_type {
                        self.config_unsaved = true;
                    }
                    
                    ui.separator();
                    
                    if ui.button("Start Monitoring").clicked() && !self.info_type.is_empty() {
                        self.needs_config = false;
                        self.start();
                    }
                });
        } else {
            // Execution mode
            let title = match self.info_type.as_str() {
                "hardware" => "Hardware Information",
                "activity" => "Activity Monitor",
                _ => "System Overview",
            };
            
            egui::Window::new(title)
                .id(egui::Id::new(format!("system_widget_{}", self.id)))
                .open(&mut open)
                .default_pos([250.0 + (idx as f32 * 50.0), 100.0 + (idx as f32 * 50.0)])
                .default_size([600.0, 400.0])
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        refresh_clicked = self.render_controls(ui);
                        
                        ui.separator();
                        ui.label("Type:");
                        let old_type = self.info_type.clone();
                        ui.selectable_value(&mut self.info_type, "hardware".to_string(), "Hardware");
                        ui.selectable_value(&mut self.info_type, "activity".to_string(), "Activity");
                        ui.selectable_value(&mut self.info_type, "overview".to_string(), "Overview");
                        if old_type != self.info_type {
                            self.config_unsaved = true;
                            // Restart with new command if running
                            if self.executor.is_running() {
                                self.stop();
                                self.start();
                            }
                        }
                    });
                    
                    ui.separator();
                    self.render_output(ui);
                });
        }
        
        (open, refresh_clicked)
    }
    
    fn refresh(&self) {
        self.stop();
        self.start();
    }
}

// Implement the CommandWidget trait
impl CommandWidget for SystemInfoWidget {
    fn build_command(&self) -> CommandSpec {
        let command = match self.info_type.as_str() {
            "hardware" => "system_profiler SPHardwareDataType",
            "activity" => "top -l 1 -o cpu -n 10",
            _ => "uname -a && sw_vers",
        };
        
        CommandSpec::new("sh")
            .arg("-c")
            .arg(command)
    }
    
    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
    
    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::OneShot
    }
}

// Implement UI traits
impl CommandOutputRenderer for SystemInfoWidget {
    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
}

impl CommandControlBar for SystemInfoWidget {}

impl SystemInfoWidget {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            version: 0,
            info_type: "overview".to_string(),
            needs_config: false,
            executor: CommandExecutor::new(),
            config_unsaved: false,
        }
    }
    
    pub fn new_with_config(id: usize) -> Self {
        Self {
            id,
            version: 0,
            info_type: String::new(),
            needs_config: true,
            executor: CommandExecutor::new(),
            config_unsaved: false,
        }
    }
}