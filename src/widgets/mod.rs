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

#[derive(Clone)]
pub enum WidgetType {
    SSHCommand(SSHCommandWidget),
    CPUMonitor(CPUMonitorWidget),
    SystemInfo(SystemInfoWidget),
    ProcessMonitor(ProcessMonitorWidget),
    NetworkMonitor(NetworkMonitorWidget),
    About(AboutWidget),
}

impl WidgetType {
    pub fn execute(&self) {
        match self {
            WidgetType::SSHCommand(w) => w.execute(),
            WidgetType::CPUMonitor(w) => w.execute(),
            WidgetType::SystemInfo(w) => w.execute(),
            WidgetType::ProcessMonitor(w) => w.execute(),
            WidgetType::NetworkMonitor(w) => w.execute(),
            WidgetType::About(w) => w.execute(),
        }
    }
}