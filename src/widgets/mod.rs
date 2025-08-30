pub mod ssh_command;
pub mod cpu_monitor;
pub mod system_info;
pub mod process_monitor;
pub mod network_monitor;
pub mod about;

pub use ssh_command::SSHCommandWidget;
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
    fn start(&self) {} // Default no-op implementation  
    fn stop(&self) {} // Default no-op implementation
    fn render(&mut self, ctx: &egui::Context, idx: usize) -> (bool, bool);
    fn refresh(&self) { self.stop(); self.start(); }
}

// Self-contained widget creation functions - each widget handles its own configuration
impl WidgetType {
    pub fn new_ssh_command(id: usize) -> Self {
        WidgetType::SSHCommand(SSHCommandWidget::new_with_config(id))
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
    SSHCommand(SSHCommandWidget),
    CPUMonitor(CPUMonitorWidget),
    SystemInfo(SystemInfoWidget),
    ProcessMonitor(ProcessMonitorWidget),
    NetworkMonitor(NetworkMonitorWidget),
    About(AboutWidget),
}

