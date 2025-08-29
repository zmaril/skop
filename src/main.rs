use eframe::egui;

mod widgets;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "skop",
        options,
        Box::new(|cc| Ok(Box::new(Skop::new(cc)))),
    )
}

use widgets::{WidgetType, SSHCommandWidget, CPUMonitorWidget, SystemInfoWidget};

struct Skop {
    // Widget system
    widgets: Vec<WidgetType>,
    next_widget_id: usize,
    
    // Configuration dialogs
    show_ssh_config: bool,
    config_hostname: String,
    config_command: String,
}

impl Skop {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Disable egui debug mode to hide widget ID warnings
        cc.egui_ctx.set_debug_on_hover(false);
        
        // Set default font size
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional));
        style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::new(14.0, egui::FontFamily::Monospace));
        cc.egui_ctx.set_style(style);
        
        Self {
            widgets: vec![],
            next_widget_id: 0,
            
            show_ssh_config: false,
            config_hostname: String::from("localhost"),
            config_command: String::from(""),
        }
    }
    
    fn create_ssh_widget(&mut self, hostname: String, command: String) {
        let widget = SSHCommandWidget::new(self.next_widget_id, hostname, command);
        self.widgets.push(WidgetType::SSHCommand(widget));
        // Auto-execute the newly created SSH widget
        if let Some(WidgetType::SSHCommand(ssh_widget)) = self.widgets.last() {
            ssh_widget.execute();
        }
        self.next_widget_id += 1;
    }
    
    fn create_cpu_monitor(&mut self) {
        let widget = CPUMonitorWidget::new(self.next_widget_id);
        self.widgets.push(WidgetType::CPUMonitor(widget));
        // Auto-start the CPU monitor
        if let Some(WidgetType::CPUMonitor(cpu_widget)) = self.widgets.last() {
            cpu_widget.execute();
        }
        self.next_widget_id += 1;
    }
    
    fn create_system_info(&mut self, info_type: String) {
        let widget = SystemInfoWidget::new(self.next_widget_id, info_type);
        self.widgets.push(WidgetType::SystemInfo(widget));
        self.next_widget_id += 1;
    }
}


impl eframe::App for Skop {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint for live updates
        ctx.request_repaint();
        
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| {
                    if ui.button("Clear All Widgets").clicked() {
                        self.widgets.clear();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    ui.label("Skop - Widget-based Command Runner");
                    ui.separator();
                    ui.label("Use the sidebar to create widgets");
                });
            });
        });
        
        // Left Sidebar - Widget Producer Menu
        egui::SidePanel::left("widget_menu")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Widget Menu");
                ui.separator();
                
                ui.label("System Monitoring:");
                ui.vertical(|ui| {
                    if ui.button("CPU Monitor").clicked() {
                        self.create_cpu_monitor();
                    }
                    if ui.button("System Info").clicked() {
                        self.create_system_info("hardware".to_string());
                    }
                    if ui.button("Activity Monitor").clicked() {
                        self.create_system_info("activity".to_string());
                    }
                });
                
                ui.separator();
                
                ui.label("SSH Commands:");
                ui.vertical(|ui| {
                    if ui.button("Custom SSH").clicked() {
                        self.show_ssh_config = true;
                    }
                    if ui.button("File List").clicked() {
                        self.create_ssh_widget("localhost".to_string(), "ls -la".to_string());
                    }
                    if ui.button("Working Dir").clicked() {
                        self.create_ssh_widget("localhost".to_string(), "pwd".to_string());
                    }
                    if ui.button("Who Am I").clicked() {
                        self.create_ssh_widget("localhost".to_string(), "whoami".to_string());
                    }
                    if ui.button("Disk Usage").clicked() {
                        self.create_ssh_widget("localhost".to_string(), "df -h".to_string());
                    }
                    if ui.button("Ping Test").clicked() {
                        self.create_ssh_widget("localhost".to_string(), "ping -c 3 google.com".to_string());
                    }
                });
                
                ui.separator();
                
                ui.label(format!("Active Widgets: {}", self.widgets.len()));
            });
        
        // SSH Configuration Dialog
        let mut create_ssh_widget = false;
        let mut cancel_ssh_config = false;
        
        if self.show_ssh_config {
            egui::Window::new("SSH Configuration")
                .open(&mut self.show_ssh_config)
                .default_size([400.0, 200.0])
                .show(ctx, |ui| {
                    ui.label("Configure SSH Command:");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Host:");
                        ui.text_edit_singleline(&mut self.config_hostname);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Command:");
                        ui.text_edit_singleline(&mut self.config_command);
                    });
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("Execute Command").clicked() && !self.config_hostname.is_empty() && !self.config_command.is_empty() {
                            create_ssh_widget = true;
                        }
                        
                        if ui.button("Cancel").clicked() {
                            cancel_ssh_config = true;
                        }
                    });
                });
        }
        
        // Handle SSH config actions
        if create_ssh_widget {
            self.create_ssh_widget(self.config_hostname.clone(), self.config_command.clone());
            self.show_ssh_config = false;
            self.config_hostname = "localhost".to_string();
            self.config_command.clear();
        }
        
        if cancel_ssh_config {
            self.show_ssh_config = false;
        }
        
        // Render all widgets
        let mut widgets_to_remove = vec![];
        let mut cpu_refresh_requests = vec![];
        let mut info_refresh_requests = vec![];
        
        for (idx, widget) in self.widgets.iter_mut().enumerate() {
            let should_remove = match widget {
                WidgetType::SSHCommand(ssh_widget) => {
                    !ssh_widget.render(ctx, idx)
                },
                WidgetType::CPUMonitor(cpu_widget) => {
                    let (open, refresh_clicked) = cpu_widget.render(ctx, idx);
                    if refresh_clicked {
                        cpu_refresh_requests.push(cpu_widget.clone());
                    }
                    !open
                },
                WidgetType::SystemInfo(info_widget) => {
                    let (open, refresh_clicked) = info_widget.render(ctx, idx);
                    if refresh_clicked {
                        info_refresh_requests.push(info_widget.clone());
                    }
                    !open
                },
            };
            
            if should_remove {
                widgets_to_remove.push(idx);
            }
        }
        
        // Handle refresh requests
        for cpu_widget in cpu_refresh_requests {
            cpu_widget.execute();
        }
        
        for info_widget in info_refresh_requests {
            info_widget.execute();
        }
        
        // Remove closed widgets
        for idx in widgets_to_remove.iter().rev() {
            self.widgets.remove(*idx);
        }
        
        // Central panel (background)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Skop - Widget Command Runner");
            ui.label("Execute commands and monitor system resources with widgets.");
            
            ui.add_space(20.0);
            
            if self.widgets.is_empty() {
                ui.label("No active widgets. Use the widget menu on the left to create widgets.");
                ui.add_space(10.0);
                ui.label("Available widget types:");
                ui.label("• SSH Commands - Run commands locally or remotely");
                ui.label("• CPU Monitor - View CPU usage and system load");
                ui.label("• System Info - Display hardware information");
                ui.label("• Activity Monitor - Show top processes");
            } else {
                ui.label(format!("Active widgets: {}", self.widgets.len()));
            }
        });
    }
}