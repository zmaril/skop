use eframe::egui;
use crate::{AppMode, Skop};

impl Skop {
    pub fn render_settings(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                if ui.button("← Back").clicked() {
                    self.mode = AppMode::Home;
                    self.home_quote_index = 0; // Reset to trigger new quote selection
                }
                ui.add_space(50.0);
                ui.heading(egui::RichText::new("Settings").size(32.0));
            });
        });
    }
}