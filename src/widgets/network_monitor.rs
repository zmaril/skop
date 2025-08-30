use std::time::Duration;
use eframe::egui;
use serde::{Serialize, Deserialize};
use crate::widgets::command_widget::{CommandExecutor, CommandSpec, CommandWidget, ExecutionMode, CommandOutputRenderer, CommandControlBar};

#[derive(Clone, Serialize, Deserialize)]
pub struct NetworkMonitorWidget {
    pub id: usize,
    pub version: i32,
    pub refresh_interval_secs: u64,
    pub filter_text: String,
    pub show_established_only: bool,
    #[serde(skip, default = "default_executor")]
    pub executor: CommandExecutor,
    #[serde(skip, default)]
    pub config_unsaved: bool,
}

fn default_executor() -> CommandExecutor {
    CommandExecutor::new()
}

impl crate::widgets::Widget for NetworkMonitorWidget {
    fn widget_type_name(&self) -> &'static str {
        "network_monitor"
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
        // Only restart if refresh interval changed, not for filter changes
        // In the future, we might track which specific config changed
        // For now, assume any config change needs restart
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
        
        egui::Window::new("Network Connections")
            .id(egui::Id::new(format!("network_widget_{}", self.id)))
            .open(&mut open)
            .default_pos([300.0 + (idx as f32 * 50.0), 150.0 + (idx as f32 * 50.0)])
            .default_size([700.0, 400.0])
            .resizable(true)
            .show(ctx, |ui| {
                // Control bar
                ui.horizontal(|ui| {
                    refresh_clicked = self.render_controls(ui);
                    
                    ui.separator();
                    
                    let old_established = self.show_established_only;
                    ui.checkbox(&mut self.show_established_only, "Established Only");
                    if old_established != self.show_established_only {
                        self.config_unsaved = true;
                    }
                    
                    ui.separator();
                    
                    ui.label("Filter:");
                    let old_filter = self.filter_text.clone();
                    ui.text_edit_singleline(&mut self.filter_text);
                    if old_filter != self.filter_text {
                        self.config_unsaved = true;
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
                
                // Output from netstat command
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

// Implement the CommandWidget trait
impl CommandWidget for NetworkMonitorWidget {
    fn build_command(&self) -> CommandSpec {
        CommandSpec::new("netstat")
            .arg("-an")
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

// Implement UI traits
impl CommandOutputRenderer for NetworkMonitorWidget {
    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
}

impl CommandControlBar for NetworkMonitorWidget {}

impl NetworkMonitorWidget {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            version: 0,
            refresh_interval_secs: 5, // 5 second refresh
            filter_text: String::new(),
            show_established_only: false,
            executor: CommandExecutor::new(),
            config_unsaved: false,
        }
    }
}