use eframe::egui;

mod widgets;
mod database;
mod investigation;
mod views;

fn main() -> eframe::Result {
    // Initialize database on startup
    if let Err(e) = database::ensure_skop_dir() {
        eprintln!("Failed to create skop directory: {}", e);
        std::process::exit(1);
    }
    
    // Initialize database in tokio runtime
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        if let Err(e) = database::main_db::MainDB::new().await {
            eprintln!("Failed to initialize main database: {}", e);
            std::process::exit(1);
        }
    });
    
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

use widgets::WidgetType;
use investigation::Investigation;
use database::main_db::MainDB;

#[derive(PartialEq)]
pub enum AppMode {
    Home,
    InvestigationWorkspace,
    Settings,
    About,
    Help,
}

pub struct Skop {
    // App mode
    pub mode: AppMode,
    
    // Investigation browser
    pub investigations: Vec<Investigation>,
    pub current_investigation: Option<Investigation>,
    pub main_db: Option<MainDB>,
    pub show_delete_confirmation: bool,
    pub investigation_to_delete: Option<usize>,
    pub home_quote_index: usize,
    
    // Widget system (for workspace mode)
    pub widgets: Vec<WidgetType>,
    pub next_widget_id: usize,
    
    // Configuration dialogs
    pub show_ssh_config: bool,
    pub config_hostname: String,
    pub config_command: String,
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
            mode: AppMode::Home,
            
            investigations: vec![],
            current_investigation: None,
            main_db: None,
            show_delete_confirmation: false,
            investigation_to_delete: None,
            home_quote_index: 0,
            
            widgets: vec![],
            next_widget_id: 0,
            
            show_ssh_config: false,
            config_hostname: String::from("localhost"),
            config_command: String::from(""),
        }
    }
    
    pub fn add_widget(&mut self, widget: WidgetType) {
        widget.execute(); // Auto-execute before adding
        self.widgets.push(widget);
        self.next_widget_id += 1;
    }
}


impl eframe::App for Skop {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint for live updates
        ctx.request_repaint();
        
        // Load database and investigations if not loaded
        if self.main_db.is_none() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(MainDB::new()) {
                Ok(db) => {
                    match rt.block_on(Investigation::load_all(&db)) {
                        Ok(investigations) => {
                            self.investigations = investigations;
                            println!("Loaded {} investigations", self.investigations.len());
                        }
                        Err(e) => println!("Failed to load investigations: {}", e),
                    }
                    self.main_db = Some(db);
                    println!("Database initialized successfully");
                }
                Err(e) => {
                    println!("Failed to initialize database: {}", e);
                }
            }
        }
        
        match self.mode {
            AppMode::Home => self.render_home(ctx),
            AppMode::InvestigationWorkspace => self.render_investigation_workspace(ctx),
            AppMode::Settings => self.render_settings(ctx),
            AppMode::About => self.render_about(ctx),
            AppMode::Help => self.render_help(ctx),
        }
    }
}