use eframe::egui;
use serde::{Serialize, Deserialize};
use crate::widgets::command_widget::{CommandExecutor, CommandSpec, CommandWidget, ExecutionMode, CommandOutputRenderer, CommandControlBar};

#[derive(Clone, Serialize, Deserialize)]
pub struct RawCommandWidget {
    pub id: usize,
    pub version: i32,
    pub command: String,
    pub needs_config: bool,
    #[serde(skip, default = "default_executor")]
    pub executor: CommandExecutor,
    #[serde(skip, default)]
    pub config_unsaved: bool,
}

fn default_executor() -> CommandExecutor {
    CommandExecutor::new()
}

impl crate::widgets::Widget for RawCommandWidget {
    fn widget_type_name(&self) -> &'static str {
        "raw_command"
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
    
    fn restore_widget_data(&mut self, data: Vec<String>) {
        if !data.is_empty() {
            self.executor.load_historical_output(data);
            // If we have historical data, the widget was previously configured
            self.needs_config = false;
        }
    }
    
    fn set_available_hosts(&mut self, hosts: Vec<crate::database::investigation_db::Host>) {
        self.executor.set_available_hosts(hosts);
    }
    
    fn config_changed(&self) -> bool {
        self.config_unsaved
    }
    
    
    fn needs_restart(&self) -> bool {
        // Raw command needs restart when command changes
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
        
        if self.needs_config {
            // Configuration mode
            egui::Window::new("Raw Command Configuration")
                .id(egui::Id::new(format!("raw_config_{}", self.id)))
                .open(&mut open)
                .default_pos([400.0 + (idx as f32 * 30.0), 200.0 + (idx as f32 * 30.0)])
                .default_size([400.0, 200.0])
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label("Configure Command:");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Command:");
                        let old_command = self.command.clone();
                        ui.text_edit_singleline(&mut self.command);
                        if old_command != self.command {
                            self.config_unsaved = true;
                        }
                    });
                    
                    ui.separator();
                    
                    if ui.button("Execute Command").clicked() && !self.command.is_empty() {
                        self.needs_config = false;
                        self.start();
                    }
                });
        } else {
            // Execution mode  
            egui::Window::new(format!("Raw Command: {}", self.command))
                .id(egui::Id::new(format!("raw_widget_{}", self.id)))
                .open(&mut open)
                .default_pos([250.0 + (idx as f32 * 50.0), 100.0 + (idx as f32 * 50.0)])
                .default_size([600.0, 400.0])
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        refresh_clicked = self.render_controls(ui);
                        
                        ui.separator();
                        ui.label("Command:");
                        let old_command = self.command.clone();
                        ui.text_edit_singleline(&mut self.command);
                        if old_command != self.command {
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
impl CommandWidget for RawCommandWidget {
    fn build_command(&self) -> CommandSpec {
        // Use shell to execute the raw command
        CommandSpec::new("sh")
            .arg("-c")
            .arg(&self.command)
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
impl CommandOutputRenderer for RawCommandWidget {
    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
}

impl CommandControlBar for RawCommandWidget {}

impl RawCommandWidget {
    pub fn new(id: usize, command: String) -> Self {
        Self {
            id,
            version: 0,
            command,
            needs_config: false,
            executor: CommandExecutor::new(),
            config_unsaved: false,
        }
    }
    
    pub fn new_with_config(id: usize) -> Self {
        Self {
            id,
            version: 0,
            command: String::new(),
            needs_config: true,
            executor: CommandExecutor::new(),
            config_unsaved: false,
        }
    }
}