use eframe::egui;
use crate::{AppMode, Skop};
use crate::widgets::{WidgetType, Widget};
use crate::investigation::{Investigation, COLORS, find_color_name};
use crate::database::investigation_db::Host;

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
                    
                    // Reload investigations to reflect any changes made in workspace
                    if let Some(ref main_db) = self.main_db {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        match rt.block_on(Investigation::load_all(main_db)) {
                            Ok(investigations) => {
                                self.investigations = investigations;
                                println!("Reloaded {} investigations when returning to home", self.investigations.len());
                            }
                            Err(e) => {
                                eprintln!("ERROR: Failed to reload investigations: {}", e);
                            }
                        }
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
        
        // Extract data needed for UI to avoid borrowing conflicts
        let investigation_data = self.current_investigation.as_ref().map(|inv| {
            (inv.name.clone(), inv.description.clone(), inv.color.clone())
        });
        
        // Track if we need to update investigation after UI
        let mut should_update_investigation = false;
        let mut new_name = String::new();
        let mut new_description = String::new();
        let mut new_color = [0.0, 0.0, 0.0];
        
        // Left Sidebar - Widget Producer Menu
        egui::SidePanel::left("widget_menu")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                if let Some((name, description, color)) = &investigation_data {
                    // Create a muted version for background
                    let bg_color = egui::Color32::from_rgb(
                        ((color[0] * 0.2 + 0.8) * 255.0) as u8,
                        ((color[1] * 0.2 + 0.8) * 255.0) as u8,
                        ((color[2] * 0.2 + 0.8) * 255.0) as u8,
                    );
                    
                    let response = ui.allocate_response(
                        egui::vec2(ui.available_width(), 35.0),
                        egui::Sense::hover()
                    );
                    
                    ui.painter().rect_filled(response.rect, 4.0, bg_color);
                    
                    // Draw the text on top with proper padding
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(response.rect.shrink2(egui::vec2(10.0, 5.0))), |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(name);
                        });
                    });
                    
                    ui.add_space(5.0);
                    
                    // Investigation editing controls using egui state management
                    ui.collapsing("Edit Investigation", |ui| {
                        // Use egui's persistent state for editing values
                        let mut edit_name = ui.ctx().data_mut(|d| d.get_temp::<String>(egui::Id::new("edit_inv_name"))).unwrap_or_else(|| name.clone());
                        let mut edit_desc = ui.ctx().data_mut(|d| d.get_temp::<String>(egui::Id::new("edit_inv_desc"))).unwrap_or_else(|| description.clone());
                        let mut edit_color = ui.ctx().data_mut(|d| d.get_temp::<[f32; 3]>(egui::Id::new("edit_inv_color"))).unwrap_or(*color);
                        
                        ui.label("Name:");
                        let name_changed = ui.text_edit_singleline(&mut edit_name).changed();
                        if name_changed {
                            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("edit_inv_name"), edit_name.clone()));
                        }
                        
                        ui.add_space(5.0);
                        
                        ui.label("Description:");
                        let desc_changed = ui.text_edit_multiline(&mut edit_desc).changed();
                        if desc_changed {
                            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("edit_inv_desc"), edit_desc.clone()));
                        }
                        
                        ui.add_space(5.0);
                        
                        ui.label("Color:");
                        
                        // Find current color name
                        let current_color_name = find_color_name(edit_color).unwrap_or("Unknown");
                        let mut selected_color_name = ui.ctx().data_mut(|d| 
                            d.get_temp::<String>(egui::Id::new("edit_inv_color_name"))
                        ).unwrap_or_else(|| current_color_name.to_string());
                        
                        let mut color_changed = false;
                        
                        egui::ComboBox::from_label("")
                            .selected_text(&selected_color_name)
                            .show_ui(ui, |ui| {
                                for (color_name, color_rgb) in COLORS {
                                    let was_selected = ui.selectable_value(&mut selected_color_name, color_name.to_string(), *color_name).clicked();
                                    if was_selected {
                                        edit_color = *color_rgb;
                                        color_changed = true;
                                    }
                                }
                            });
                        
                        if color_changed {
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(egui::Id::new("edit_inv_color"), edit_color);
                                d.insert_temp(egui::Id::new("edit_inv_color_name"), selected_color_name);
                            });
                        }
                        
                        // Show color preview
                        let color_preview = egui::Color32::from_rgb(
                            (edit_color[0] * 255.0) as u8,
                            (edit_color[1] * 255.0) as u8,
                            (edit_color[2] * 255.0) as u8,
                        );
                        ui.horizontal(|ui| {
                            ui.label("Preview:");
                            let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2.0, color_preview);
                        });
                        
                        ui.add_space(10.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button("Save Changes").clicked() {
                                should_update_investigation = true;
                                new_name = edit_name.clone();
                                new_description = edit_desc.clone();
                                new_color = edit_color;
                                
                                // Clear the temp data after saving
                                ui.ctx().data_mut(|d| {
                                    d.remove::<String>(egui::Id::new("edit_inv_name"));
                                    d.remove::<String>(egui::Id::new("edit_inv_desc"));
                                    d.remove::<[f32; 3]>(egui::Id::new("edit_inv_color"));
                                    d.remove::<String>(egui::Id::new("edit_inv_color_name"));
                                });
                            }
                            
                            if ui.button("Reset").clicked() {
                                // Clear temp data to reset to original values
                                ui.ctx().data_mut(|d| {
                                    d.remove::<String>(egui::Id::new("edit_inv_name"));
                                    d.remove::<String>(egui::Id::new("edit_inv_desc"));
                                    d.remove::<[f32; 3]>(egui::Id::new("edit_inv_color"));
                                    d.remove::<String>(egui::Id::new("edit_inv_color_name"));
                                });
                            }
                        });
                    });
                    
                    ui.separator();
                }
                
                // Host management section
                ui.collapsing("Host Configuration", |ui| {
                    ui.label(format!("Configured Hosts: {}", self.hosts.len()));
                    
                    // List existing hosts
                    egui::ScrollArea::vertical()
                        .max_height(100.0)
                        .show(ui, |ui| {
                            for host in &self.hosts {
                                ui.horizontal(|ui| {
                                    if host.is_localhost {
                                        ui.label("🏠");
                                    } else {
                                        ui.label("🖥️");
                                    }
                                    ui.label(&host.name);
                                    if !host.is_localhost {
                                        ui.label(format!("({})", host.ssh_alias));
                                    }
                                });
                            }
                        });
                    
                    ui.separator();
                    
                    // Add new host section
                    ui.collapsing("Add New Host", |ui| {
                        let mut new_host_name = ui.ctx().data_mut(|d| 
                            d.get_temp::<String>(egui::Id::new("new_host_name"))
                        ).unwrap_or_default();
                        let mut new_ssh_alias = ui.ctx().data_mut(|d| 
                            d.get_temp::<String>(egui::Id::new("new_ssh_alias"))
                        ).unwrap_or_default();
                        let mut new_host_description = ui.ctx().data_mut(|d| 
                            d.get_temp::<String>(egui::Id::new("new_host_description"))
                        ).unwrap_or_default();
                        
                        ui.label("Display Name:");
                        if ui.text_edit_singleline(&mut new_host_name).changed() {
                            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("new_host_name"), new_host_name.clone()));
                        }
                        
                        ui.label("SSH Alias:");
                        ui.small("Examples: 'myserver', 'user@hostname', 'user@192.168.1.100'");
                        if ui.text_edit_singleline(&mut new_ssh_alias).changed() {
                            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("new_ssh_alias"), new_ssh_alias.clone()));
                        }
                        
                        ui.label("Description:");
                        if ui.text_edit_multiline(&mut new_host_description).changed() {
                            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("new_host_description"), new_host_description.clone()));
                        }
                        
                        ui.horizontal(|ui| {
                            if ui.button("Add Host").clicked() {
                                if !new_host_name.trim().is_empty() && !new_ssh_alias.trim().is_empty() {
                                    // Add host to database
                                    if let Some(ref current_investigation) = self.current_investigation {
                                        let rt = tokio::runtime::Runtime::new().unwrap();
                                        match rt.block_on(async {
                                            let db = current_investigation.open().await?;
                                            db.add_host(&new_host_name, &new_ssh_alias, &new_host_description).await
                                        }) {
                                            Ok(host_id) => {
                                                println!("Added host '{}' with ID {}", new_host_name, host_id);
                                                
                                                // Add to local list
                                                self.hosts.push(Host {
                                                    id: Some(host_id),
                                                    name: new_host_name.clone(),
                                                    ssh_alias: new_ssh_alias.clone(),
                                                    description: new_host_description.clone(),
                                                    is_localhost: new_ssh_alias == "localhost" || new_ssh_alias == "127.0.0.1",
                                                });
                                                
                                                // Update all existing widgets with the new host list
                                                for widget in &mut self.widgets {
                                                    widget.set_available_hosts(self.hosts.clone());
                                                }
                                                
                                                // Clear form
                                                ui.ctx().data_mut(|d| {
                                                    d.remove::<String>(egui::Id::new("new_host_name"));
                                                    d.remove::<String>(egui::Id::new("new_ssh_alias"));
                                                    d.remove::<String>(egui::Id::new("new_host_description"));
                                                });
                                            }
                                            Err(e) => {
                                                eprintln!("ERROR: Failed to add host: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                            
                            if ui.button("Clear").clicked() {
                                ui.ctx().data_mut(|d| {
                                    d.remove::<String>(egui::Id::new("new_host_name"));
                                    d.remove::<String>(egui::Id::new("new_ssh_alias"));
                                    d.remove::<String>(egui::Id::new("new_host_description"));
                                });
                            }
                        });
                    });
                });
                
                ui.separator();
                
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
                
                ui.label("Commands:");
                ui.vertical(|ui| {
                    if ui.button("Command").clicked() {
                        self.add_widget(WidgetType::new_raw_command(self.next_widget_id));
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
        
        // Handle investigation updates after UI to avoid borrowing conflicts
        if should_update_investigation {
            if let Some(ref mut investigation) = self.current_investigation {
                investigation.name = new_name.clone();
                investigation.description = new_description.clone();
                investigation.color = new_color;
                
                // Update investigation metadata in database
                let rt = tokio::runtime::Runtime::new().unwrap();
                match rt.block_on(investigation.update_metadata()) {
                    Ok(()) => {
                        println!("Successfully updated investigation metadata");
                    }
                    Err(e) => {
                        eprintln!("ERROR: Failed to update investigation metadata: {}", e);
                        eprintln!("Investigation: {}", investigation.name);
                        eprintln!("Database path: {:?}", investigation.file_path);
                    }
                }
                
                // Update last accessed time in main database
                if let Some(ref main_db) = self.main_db {
                    match rt.block_on(investigation.update_last_accessed(main_db)) {
                        Ok(()) => {
                            println!("Successfully updated last accessed time");
                        }
                        Err(e) => {
                            eprintln!("ERROR: Failed to update last accessed time: {}", e);
                            eprintln!("Investigation ID: {:?}", investigation.id);
                        }
                    }
                }
            }
        }
        
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