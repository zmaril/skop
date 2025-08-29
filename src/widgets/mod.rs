pub mod ssh_command;
pub mod cpu_monitor;
pub mod system_info;

use std::sync::{Arc, Mutex};

pub use ssh_command::SSHCommandWidget;
pub use cpu_monitor::CPUMonitorWidget;
pub use system_info::SystemInfoWidget;

#[derive(Clone)]
pub enum WidgetType {
    SSHCommand(SSHCommandWidget),
    CPUMonitor(CPUMonitorWidget),
    SystemInfo(SystemInfoWidget),
}