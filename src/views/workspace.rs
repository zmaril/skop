use eframe::egui;
use crate::{AppMode, Skop};
use crate::widgets::{WidgetType, SSHCommandWidget, CPUMonitorWidget, SystemInfoWidget, ProcessMonitorWidget, NetworkMonitorWidget, AboutWidget};

impl Skop {
    pub fn render_investigation_workspace(&mut self, ctx: &egui::Context) {
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Home").clicked() {
                    self.mode = AppMode::Home;
                    self.home_quote_index = 0; // Reset to trigger new quote selection
                }
                
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
                if let Some(ref investigation) = self.current_investigation {
                    // Create a muted version for background
                    let bg_color = egui::Color32::from_rgb(
                        ((investigation.color[0] * 0.2 + 0.8) * 255.0) as u8,
                        ((investigation.color[1] * 0.2 + 0.8) * 255.0) as u8,
                        ((investigation.color[2] * 0.2 + 0.8) * 255.0) as u8,
                    );
                    
                    let response = ui.allocate_response(
                        egui::vec2(ui.available_width(), 35.0),
                        egui::Sense::hover()
                    );
                    
                    ui.painter().rect_filled(response.rect, 4.0, bg_color);
                    
                    // Draw the text on top with proper padding
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(response.rect.shrink2(egui::vec2(10.0, 5.0))), |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(&investigation.name);
                        });
                    });
                    
                    ui.add_space(5.0);
                    ui.separator();
                }
                
                ui.label("System Monitoring:");
                ui.vertical(|ui| {
                    if ui.button("CPU Monitor").clicked() {
                        let widget = CPUMonitorWidget::new(self.next_widget_id);
                        self.add_widget(WidgetType::CPUMonitor(widget));
                    }
                    if ui.button("Process Monitor").clicked() {
                        let widget = ProcessMonitorWidget::new(self.next_widget_id);
                        self.add_widget(WidgetType::ProcessMonitor(widget));
                    }
                    if ui.button("Network Monitor").clicked() {
                        let widget = NetworkMonitorWidget::new(self.next_widget_id);
                        self.add_widget(WidgetType::NetworkMonitor(widget));
                    }
                    if ui.button("System Info").clicked() {
                        let widget = SystemInfoWidget::new(self.next_widget_id, "hardware".to_string());
                        self.add_widget(WidgetType::SystemInfo(widget));
                    }
                    if ui.button("Activity Monitor").clicked() {
                        let widget = SystemInfoWidget::new(self.next_widget_id, "activity".to_string());
                        self.add_widget(WidgetType::SystemInfo(widget));
                    }
                });
                
                ui.separator();
                
                ui.label("SSH Commands:");
                ui.vertical(|ui| {
                    if ui.button("Custom SSH").clicked() {
                        self.show_ssh_config = true;
                    }
                    if ui.button("File List").clicked() {
                        let widget = SSHCommandWidget::new(self.next_widget_id, "localhost".to_string(), "ls -la".to_string());
                        self.add_widget(WidgetType::SSHCommand(widget));
                    }
                    if ui.button("Working Dir").clicked() {
                        let widget = SSHCommandWidget::new(self.next_widget_id, "localhost".to_string(), "pwd".to_string());
                        self.add_widget(WidgetType::SSHCommand(widget));
                    }
                    if ui.button("Who Am I").clicked() {
                        let widget = SSHCommandWidget::new(self.next_widget_id, "localhost".to_string(), "whoami".to_string());
                        self.add_widget(WidgetType::SSHCommand(widget));
                    }
                    if ui.button("Disk Usage").clicked() {
                        let widget = SSHCommandWidget::new(self.next_widget_id, "localhost".to_string(), "df -h".to_string());
                        self.add_widget(WidgetType::SSHCommand(widget));
                    }
                    if ui.button("Ping Test").clicked() {
                        let widget = SSHCommandWidget::new(self.next_widget_id, "localhost".to_string(), "ping -c 3 google.com".to_string());
                        self.add_widget(WidgetType::SSHCommand(widget));
                    }
                });
                
                ui.separator();
                
                ui.label("Information:");
                ui.vertical(|ui| {
                    if ui.button("About").clicked() {
                        let widget = AboutWidget::new(self.next_widget_id);
                        self.add_widget(WidgetType::About(widget));
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
            let widget = SSHCommandWidget::new(self.next_widget_id, self.config_hostname.clone(), self.config_command.clone());
            self.add_widget(WidgetType::SSHCommand(widget));
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
        let mut process_refresh_requests = vec![];
        let mut network_refresh_requests = vec![];
        
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
                WidgetType::ProcessMonitor(proc_widget) => {
                    let (open, refresh_clicked) = proc_widget.render(ctx, idx);
                    if refresh_clicked {
                        process_refresh_requests.push(proc_widget.clone());
                    }
                    !open
                },
                WidgetType::NetworkMonitor(net_widget) => {
                    let (open, refresh_clicked) = net_widget.render(ctx, idx);
                    if refresh_clicked {
                        network_refresh_requests.push(net_widget.clone());
                    }
                    !open
                },
                WidgetType::About(about_widget) => {
                    !about_widget.render(ctx, idx)
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
        
        for proc_widget in process_refresh_requests {
            proc_widget.execute();
        }
        
        for net_widget in network_refresh_requests {
            net_widget.execute();
        }
        
        // Remove closed widgets
        for idx in widgets_to_remove.iter().rev() {
            self.widgets.remove(*idx);
        }
        
        // Central panel (background)
        egui::CentralPanel::default().show(ctx, |_ui| {
            // Empty central panel - widgets float on top
        });
    }
}