pub mod command_widget;
pub mod raw_command;
pub mod cpu_monitor;
pub mod system_info;
pub mod process_monitor;
pub mod network_monitor;
pub mod about;

pub use raw_command::RawCommandWidget;
pub use cpu_monitor::CPUMonitorWidget;
pub use system_info::SystemInfoWidget;
pub use process_monitor::ProcessMonitorWidget;
pub use network_monitor::NetworkMonitorWidget;
pub use about::AboutWidget;

use serde::{Serialize, Deserialize};
use enum_dispatch::enum_dispatch;
use eframe::egui;

#[enum_dispatch]
pub trait Widget {
    fn widget_type_name(&self) -> &'static str;
    fn widget_id(&self) -> usize;
    fn widget_version(&self) -> i32;
    fn increment_version(&mut self);
    fn set_database(&mut self, database: Option<std::sync::Arc<crate::database::investigation_db::InvestigationDB>>);
    fn start(&self) {} 
    fn stop(&self) {} 
    fn render(&mut self, ctx: &egui::Context, idx: usize) -> (bool, bool);
    fn refresh(&self) { self.stop(); self.start(); }
    fn config_changed(&self) -> bool { false }
    fn needs_restart(&self) -> bool { false }
    
    // Restore historical data to widget - default no-op for widgets without data
    fn restore_widget_data(&mut self, _data: Vec<String>) {}
}

// Self-contained widget creation functions - each widget handles its own configuration
impl WidgetType {
    pub fn new_raw_command(id: usize) -> Self {
        WidgetType::RawCommand(RawCommandWidget::new_with_config(id))
    }
    
    pub fn new_cpu_monitor(id: usize) -> Self {
        WidgetType::CPUMonitor(CPUMonitorWidget::new(id))
    }
    
    pub fn new_system_info(id: usize) -> Self {
        WidgetType::SystemInfo(SystemInfoWidget::new_with_config(id))
    }
    
    pub fn new_process_monitor(id: usize) -> Self {
        WidgetType::ProcessMonitor(ProcessMonitorWidget::new(id))
    }
    
    pub fn new_network_monitor(id: usize) -> Self {
        WidgetType::NetworkMonitor(NetworkMonitorWidget::new(id))
    }
    
    pub fn new_about(id: usize) -> Self {
        WidgetType::About(AboutWidget::new(id))
    }
}

#[enum_dispatch(Widget)]
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WidgetType {
    RawCommand(RawCommandWidget),
    CPUMonitor(CPUMonitorWidget),
    SystemInfo(SystemInfoWidget),
    ProcessMonitor(ProcessMonitorWidget),
    NetworkMonitor(NetworkMonitorWidget),
    About(AboutWidget),
}

