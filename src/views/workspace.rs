use eframe::egui;
use crate::{AppMode, Skop};
use crate::widgets::{WidgetType, Widget};

impl Skop {
    pub fn render_investigation_workspace(&mut self, ctx: &egui::Context) {
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Home").clicked() {
                    // Stop all widgets before leaving workspace
                    for widget in &self.widgets {
                        widget.stop();
                    }
                    
                    self.mode = AppMode::Home;
                    self.home_quote_index = 0; // Reset to trigger new quote selection
                }
                
                ui.menu_button("View", |ui| {
                    if ui.button("Clear All Widgets").clicked() {
                        // Stop all widgets before clearing
                        for widget in &self.widgets {
                            widget.stop();
                        }
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
                        self.add_widget(WidgetType::new_cpu_monitor(self.next_widget_id));
                    }
                    if ui.button("Process Monitor").clicked() {
                        self.add_widget(WidgetType::new_process_monitor(self.next_widget_id));
                    }
                    if ui.button("Network Monitor").clicked() {
                        self.add_widget(WidgetType::new_network_monitor(self.next_widget_id));
                    }
                    if ui.button("System Info").clicked() {
                        self.add_widget(WidgetType::new_system_info(self.next_widget_id));
                    }
                });
                
                ui.separator();
                
                ui.label("SSH Commands:");
                ui.vertical(|ui| {
                    if ui.button("SSH Command").clicked() {
                        self.add_widget(WidgetType::new_ssh_command(self.next_widget_id));
                    }
                });
                
                ui.separator();
                
                ui.label("Information:");
                ui.vertical(|ui| {
                    if ui.button("About").clicked() {
                        self.add_widget(WidgetType::new_about(self.next_widget_id));
                    }
                });
                
                ui.separator();
                
                ui.label(format!("Active Widgets: {}", self.widgets.len()));
            });
        
        
        // Render all widgets
        let mut widgets_to_remove = vec![];
        
        for (idx, widget) in self.widgets.iter_mut().enumerate() {
            let (open, refresh_clicked) = widget.render(ctx, idx);
            
            if refresh_clicked {
                widget.refresh();
            }
            
            if !open {
                widgets_to_remove.push(idx);
            }
        }
        
        // Remove closed widgets
        for idx in widgets_to_remove.iter().rev() {
            let widget = &self.widgets[*idx];
            
            // Stop widget activities before removal
            widget.stop();
            
            // Archive widget in database if we have an active investigation
            if let Some(ref current_investigation) = self.current_investigation {
                let rt = tokio::runtime::Runtime::new().unwrap();
                if let Err(e) = rt.block_on(async {
                    let db = current_investigation.open().await?;
                    db.archive_widget_instance(widget).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                }) {
                    eprintln!("Failed to archive widget in database: {}", e);
                }
            }
            
            self.widgets.remove(*idx);
        }
        
        // Central panel (background)
        egui::CentralPanel::default().show(ctx, |_ui| {
            // Empty central panel - widgets float on top
        });
    }
}