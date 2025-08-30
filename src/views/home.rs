use eframe::egui;
use crate::{AppMode, Skop};
use crate::investigation::Investigation;

impl Skop {
    pub fn render_home(&mut self, ctx: &egui::Context) {
        // Select a new quote when entering the home screen
        if self.home_quote_index == 0 {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
            self.home_quote_index = (hasher.finish() as usize % 3) + 1; // 1, 2, or 3
        }
        
        // Delete confirmation dialog
        let mut delete_investigation = false;
        let mut archive_investigation = false;
        
        if self.show_delete_confirmation {
            egui::Window::new("Confirm Action")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    if let Some(idx) = self.investigation_to_delete {
                        if idx < self.investigations.len() {
                            let investigation = &self.investigations[idx];
                            ui.label(format!("What would you like to do with '{}'?", investigation.name));
                            ui.separator();
                            ui.label("Archive: Hide from view but keep data");
                            ui.label("Delete: Permanently remove all data");
                            
                            ui.add_space(10.0);
                            
                            ui.horizontal(|ui| {
                                if ui.button("Archive").clicked() {
                                    archive_investigation = true;
                                    self.show_delete_confirmation = false;
                                }
                                
                                if ui.button("Delete Forever").clicked() {
                                    delete_investigation = true;
                                    self.show_delete_confirmation = false;
                                }
                                
                                if ui.button("Cancel").clicked() {
                                    self.show_delete_confirmation = false;
                                    self.investigation_to_delete = None;
                                }
                            });
                        }
                    }
                });
        }
        
        // Handle actions
        if delete_investigation || archive_investigation {
            if let Some(delete_idx) = self.investigation_to_delete.take() {
                if delete_idx < self.investigations.len() {
                    let investigation = self.investigations.remove(delete_idx);
                    if let Some(ref db) = self.main_db {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        if delete_investigation {
                            let _ = rt.block_on(investigation.delete(db));
                        } else if archive_investigation {
                            let _ = rt.block_on(investigation.archive(db));
                        }
                    }
                }
            }
        }
        
        // Handle investigation selection - store the clicked investigation
        let mut selected_investigation: Option<Investigation> = None;
        
        // Left panel - Investigations list
        egui::SidePanel::left("investigations_panel")
            .default_width(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Investigations");
                ui.separator();
                
                if self.investigations.is_empty() {
                    ui.add_space(20.0);
                    ui.label("No investigations yet");
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (idx, investigation) in self.investigations.iter().enumerate() {
                            let response = ui.allocate_response(
                                egui::vec2(ui.available_width(), 80.0),
                                egui::Sense::click()
                            );
                            
                            // Color row based on investigation color with hover effect
                            let bg_color = if response.hovered() {
                                egui::Color32::from_rgb(
                                    ((investigation.color[0] * 0.4 + 0.6) * 255.0) as u8,
                                    ((investigation.color[1] * 0.4 + 0.6) * 255.0) as u8,
                                    ((investigation.color[2] * 0.4 + 0.6) * 255.0) as u8,
                                )
                            } else {
                                egui::Color32::from_rgb(
                                    ((investigation.color[0] * 0.2 + 0.8) * 255.0) as u8,
                                    ((investigation.color[1] * 0.2 + 0.8) * 255.0) as u8,
                                    ((investigation.color[2] * 0.2 + 0.8) * 255.0) as u8,
                                )
                            };
                            
                            ui.painter().rect_filled(response.rect, 4.0, bg_color);
                            ui.painter().rect_stroke(response.rect, 4.0, ui.style().visuals.window_stroke(), egui::StrokeKind::Inside);
                            
                            // Click to open investigation
                            if response.clicked() {
                                selected_investigation = Some(investigation.clone());
                            }
                            
                            // Content within the rect
                            let inner_rect = response.rect.shrink(10.0);
                            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(&investigation.name).size(14.0).strong());
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("🗑").clicked() {
                                                self.investigation_to_delete = Some(idx);
                                                self.show_delete_confirmation = true;
                                            }
                                        });
                                    });
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new(format!("Last accessed: {}", 
                                        Investigation::format_timestamp(investigation.last_accessed)))
                                        .size(11.0)
                                        .color(ui.style().visuals.weak_text_color()));
                                });
                            });
                            
                            ui.add_space(5.0);
                        }
                    });
                }
            });
        
        // Handle investigation selection outside the borrow
        if let Some(investigation) = selected_investigation {
            self.current_investigation = Some(investigation.clone());
            
            // Clear existing widgets
            self.widgets.clear();
            
            // Load saved widgets from database
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(self.load_widgets_from_db(&investigation)) {
                eprintln!("Failed to load widgets: {}", e);
            }
            
            self.mode = AppMode::InvestigationWorkspace;
            
            if let Some(ref db) = self.main_db {
                let _ = rt.block_on(investigation.update_last_accessed(db));
            }
        }
        
        // Central panel - Title and new investigation
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(150.0);
                
                // Main title with version - centered
                ui.heading(egui::RichText::new("skop").size(64.0).strong());
                ui.label(egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).size(20.0).color(ui.style().visuals.weak_text_color()));
                
                ui.add_space(20.0);
                
                // Display selected quote
                let quotes = [
                    "have you tried turning it off and on again",
                    "turn your head and cough", 
                    "have you checked dns yet?"
                ];
                let quote_index = if self.home_quote_index > 0 { self.home_quote_index - 1 } else { 0 };
                ui.label(egui::RichText::new(format!("\"{}\"", quotes[quote_index])).size(16.0).italics().color(ui.style().visuals.weak_text_color()));
                
                ui.add_space(60.0);
                
                // Button column - all buttons same size and centered
                ui.vertical_centered(|ui| {
                    if ui.add_sized([280.0, 50.0], egui::Button::new(egui::RichText::new("New Investigation").size(18.0))).clicked() {
                        println!("New Investigation button clicked");
                        let mut investigation = Investigation::new_with_random_name();
                        println!("Created investigation: {}", investigation.name);
                        
                        if let Some(ref db) = self.main_db {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            match rt.block_on(investigation.create(db)) {
                                Ok(_) => {
                                    println!("Investigation created successfully");
                                    self.investigations.push(investigation.clone());
                                    self.current_investigation = Some(investigation);
                                    // Clear widgets for new investigation
                                    self.widgets.clear();
                                    self.mode = AppMode::InvestigationWorkspace;
                                }
                                Err(e) => println!("Failed to create investigation: {}", e),
                            }
                        } else {
                            println!("No database available");
                        }
                    }
                    
                    ui.add_space(10.0);
                    
                    if ui.add_sized([280.0, 50.0], egui::Button::new(egui::RichText::new("Settings").size(18.0))).clicked() {
                        self.mode = AppMode::Settings;
                    }
                    
                    ui.add_space(10.0);
                    
                    if ui.add_sized([280.0, 50.0], egui::Button::new(egui::RichText::new("About").size(18.0))).clicked() {
                        self.mode = AppMode::About;
                    }
                    
                    ui.add_space(10.0);
                    
                    if ui.add_sized([280.0, 50.0], egui::Button::new(egui::RichText::new("Help").size(18.0))).clicked() {
                        self.mode = AppMode::Help;
                    }
                    
                    ui.add_space(10.0);
                    
                    if ui.add_sized([280.0, 50.0], egui::Button::new(egui::RichText::new("Quit").size(18.0))).clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });
    }
}