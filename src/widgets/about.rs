use eframe::egui;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AboutWidget {
    pub id: usize,
    pub version: i32,
}

impl crate::widgets::Widget for AboutWidget {
    fn widget_type_name(&self) -> &'static str {
        "about"
    }
    
    fn widget_id(&self) -> usize {
        self.id
    }
    
    fn widget_version(&self) -> i32 {
        self.version
    }
    
    fn increment_version(&mut self) {
        self.version += 1;
    }
    
    fn set_database(&mut self, _database: Option<std::sync::Arc<crate::database::investigation_db::InvestigationDB>>) {
        // About widget doesn't capture data
    }
    
    fn needs_restart(&self) -> bool {
        // About widget doesn't execute commands, never needs restart
        false
    }
    
    fn render(&mut self, ctx: &egui::Context, idx: usize) -> (bool, bool) {
        let mut open = true;
        
        egui::Window::new("About Skop")
            .id(egui::Id::new(format!("about_widget_{}", self.id)))
            .open(&mut open)
            .default_pos([400.0 + (idx as f32 * 30.0), 200.0 + (idx as f32 * 30.0)])
            .default_size([600.0, 500.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Skop - System Knowledge Operations Platform");
                ui.separator();
                
                ui.label("Skop makes investigating and debugging UNIX systems hurt less.");
                ui.add_space(10.0);
                
                ui.heading("Philosophy");
                ui.label("After working ten years as a system administrator, I grew tired of:");
                ui.label("• Losing significant results in terminal backscroll");
                ui.label("• Difficulty synchronizing outputs across time");
                ui.label("• Poor collaboration on investigative experiments");
                ui.label("• Command line tools that afford nothing to beginners");
                ui.label("• Missing the physical feedback of systems (CPU whir, disk sounds)");
                
                ui.add_space(10.0);
                ui.heading("Current Features");
                ui.label("• Real-time CPU monitoring with audio sonification");
                ui.label("• Process monitoring with color-coded usage indicators");
                ui.label("• Network connection monitoring");
                ui.label("• SSH command execution widgets");
                ui.label("• JSON parsing via jc for robust command output handling");
                ui.label("• Auto-executing widgets for immediate feedback");
                
                ui.add_space(10.0);
                ui.heading("Vision");
                ui.label("Built around an infinite canvas approach where you can:");
                ui.label("• Record everything happening on your system");
                ui.label("• Replay investigations for detailed analysis");
                ui.label("• Hear your system through CPU tones and activity sounds");
                ui.label("• Easily graph and pivot data without external tools");
                ui.label("• Collaborate in real-time on system investigations");
                ui.label("• Export results for sharing outside of Skop");
                
                ui.add_space(15.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Built with Rust, eframe/egui, and Kira audio engine");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.hyperlink_to("github.com/zmaril/skop", "https://github.com/zmaril/skop");
                    });
                });
            });
        
        (open, false)
    }
}

impl AboutWidget {
    pub fn new(id: usize) -> Self {
        Self { 
            id,
            version: 0,
        }
    }
    
}

