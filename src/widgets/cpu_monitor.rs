use eframe::egui;
use serde::{Serialize, Deserialize};
use crate::widgets::command_widget::{CommandExecutor, CommandSpec, CommandWidget, ExecutionMode, CommandOutputRenderer, CommandControlBar};

#[derive(Clone, Serialize, Deserialize)]
pub struct CPUMonitorWidget {
    pub id: usize,
    pub version: i32,
    pub interval_seconds: u64,
    #[serde(skip, default = "default_executor")]
    pub executor: CommandExecutor,
    #[serde(skip, default)]
    pub config_unsaved: bool,
    #[serde(skip, default)]
    pub database: Option<std::sync::Arc<crate::database::investigation_db::InvestigationDB>>,
}

fn default_executor() -> CommandExecutor {
    CommandExecutor::new()
}

impl crate::widgets::Widget for CPUMonitorWidget {
    fn widget_type_name(&self) -> &'static str {
        "cpu_monitor"
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
        self.executor.set_database(database.clone(), widget_id, widget_version);
        self.database = database;
    }
    
    fn config_changed(&self) -> bool {
        self.config_unsaved
    }
    
    fn needs_restart(&self) -> bool {
        // CPU monitor needs restart for any config change (interval affects command execution)
        self.config_unsaved
    }
    
    fn restore_widget_data(&mut self, data: Vec<String>) {
        self.executor.load_historical_output(data);
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
        
        egui::Window::new(format!("CPU Monitor (vmstat {}s)", self.interval_seconds))
            .id(egui::Id::new(format!("cpu_monitor_{}", self.id)))
            .open(&mut open)
            .default_pos([100.0 + (idx as f32 * 50.0), 100.0 + (idx as f32 * 50.0)])
            .default_size([800.0, 600.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    refresh_clicked = self.render_controls(ui);
                    
                    ui.separator();
                    ui.label("Interval:");
                    let old_interval = self.interval_seconds;
                    ui.add(egui::DragValue::new(&mut self.interval_seconds).range(1..=60).suffix("s"));
                    if old_interval != self.interval_seconds {
                        // Handle config change immediately
                        self.handle_config_change(self.database.clone());
                        
                        // Save to database if available
                        if let Some(ref db) = self.database {
                            let widget = crate::widgets::WidgetType::CPUMonitor(self.clone());
                            let db_clone = db.clone();
                            tokio::spawn(async move {
                                if let Err(e) = db_clone.save_widget_instance(&widget).await {
                                    eprintln!("Failed to save CPU monitor config change: {}", e);
                                }
                            });
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
}

// Implement the CommandWidget trait
impl CommandWidget for CPUMonitorWidget {
    fn build_command(&self) -> CommandSpec {
        CommandSpec::new("vmstat")
            .arg(self.interval_seconds.to_string())
    }
    
    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
    
    fn executor_mut(&mut self) -> &mut CommandExecutor {
        &mut self.executor
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Continuous
    }
}

// Implement UI traits
impl CommandOutputRenderer for CPUMonitorWidget {
    fn executor(&self) -> &CommandExecutor {
        &self.executor
    }
}

impl CommandControlBar for CPUMonitorWidget {}

impl CPUMonitorWidget {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            version: 0,  // Starting at 0 as requested
            interval_seconds: 2,
            executor: CommandExecutor::new(),
            config_unsaved: false,
            database: None,
        }
    }
}