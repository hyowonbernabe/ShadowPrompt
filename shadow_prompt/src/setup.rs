use eframe::egui;
use crate::config::Config;
use std::path::Path;

#[derive(PartialEq)]
enum SetupPage {
    Welcome,
    ApiConfig,
    Hotkeys,
    Visuals,
    Downloads,
    Finish,
}

pub struct SetupWizard {
    current_page: SetupPage,
    config: Config,
    downloading: bool,
    download_progress: f32,
    download_status: String,
    dll_missing: bool,
    finished: bool,
}

impl SetupWizard {
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        let dll_missing = !Path::new("bin/onnxruntime.dll").exists() && !Path::new("onnxruntime.dll").exists();
        
        Self {
            current_page: SetupPage::Welcome,
            config,
            downloading: false,
            download_progress: 0.0,
            download_status: "Ready".to_string(),
            dll_missing,
            finished: false,
        }
    }

    pub fn show(self) -> bool {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([600.0, 550.0])
                .with_title("ShadowPrompt Setup")
                .with_resizable(false),
            ..Default::default()
        };

        let _ = eframe::run_native(
            "ShadowPrompt Setup",
            options,
            Box::new(|cc| {
                // Install image loaders for PNG
                egui_extras::install_image_loaders(&cc.egui_ctx);
                
                // Set Dark visual style
                cc.egui_ctx.set_visuals(egui::Visuals::dark());
                
                Ok(Box::new(self))
            }),
        );
        
        // After GUI closes, determine if we should exit main entirely (if we re-execed)
        // or just return to main. Actually, re-exec handles the exit.
        std::path::Path::new("config/.setup_complete").exists()
    }
}

impl eframe::App for SetupWizard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Logo
                ui.add(egui::Image::new(egui::include_image!("assets/logo_512.png")).max_width(128.0));
                ui.heading("ShadowPrompt Setup");
                ui.add_space(10.0);
            });

            ui.separator();
            ui.add_space(20.0);

            match self.current_page {
                SetupPage::Welcome => self.show_welcome(ui),
                SetupPage::ApiConfig => self.show_api_config(ui),
                SetupPage::Hotkeys => self.show_hotkeys(ui),
                SetupPage::Visuals => self.show_visuals(ui),
                SetupPage::Downloads => self.show_downloads(ui),
                SetupPage::Finish => self.show_finish(ui),
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(20.0);
                ui.horizontal(|ui| {
                    if self.current_page != SetupPage::Welcome && self.current_page != SetupPage::Finish {
                        if ui.button("Back").clicked() {
                            self.prev_page();
                        }
                    }

                    if self.current_page != SetupPage::Finish {
                        if ui.button(if self.current_page == SetupPage::Downloads { "Next" } else { "Continue" }).clicked() {
                            self.next_page();
                        }
                    } else {
                        if ui.button("Start ShadowPrompt").clicked() {
                            let _ = self.config.save();
                            let _ = Config::mark_setup_complete();
                            self.finished = true;
                            
                            // 1. Spawn a clean instance of the app in background
                            self.spawn_app_and_exit();
                        }
                    }

                    if self.current_page != SetupPage::Finish && self.current_page != SetupPage::Downloads {
                        if ui.button("Skip Setup").clicked() {
                             let _ = Config::mark_setup_complete();
                             self.spawn_app_and_exit();
                        }
                    }
                });
            });
        });
    }
}

impl SetupWizard {
    fn show_welcome(&mut self, ui: &mut egui::Ui) {
        ui.label("Welcome to ShadowPrompt! This wizard will help you configure your portable AI assistant.");
        ui.add_space(10.0);
        ui.colored_label(egui::Color32::YELLOW, "IMPORTANT: This setup runs only ONCE.");
        ui.label("After this, the app will run invisibly in the background. To change settings later, you must edit config/config.toml manually.");
    }

    fn show_api_config(&mut self, ui: &mut egui::Ui) {
        ui.heading("LLM Provider Configuration");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Active Provider:");
            ui.selectable_value(&mut self.config.models.provider, "groq".to_string(), "Groq");
            ui.selectable_value(&mut self.config.models.provider, "openrouter".to_string(), "OpenRouter");
            ui.selectable_value(&mut self.config.models.provider, "ollama".to_string(), "Ollama (Local)");
        });

        ui.add_space(10.0);

        match self.config.models.provider.as_str() {
            "groq" => {
                let groq = self.config.models.groq.get_or_insert_with(Default::default);
                ui.label("Groq API Key:");
                ui.text_edit_singleline(&mut groq.api_key);
                ui.label("Model ID:");
                ui.text_edit_singleline(&mut groq.model_id);
            }
            "openrouter" => {
                let or = self.config.models.openrouter.get_or_insert_with(Default::default);
                ui.label("OpenRouter API Key:");
                ui.text_edit_singleline(&mut or.api_key);
                ui.label("Model ID:");
                ui.text_edit_singleline(&mut or.model_id);
            }
            "ollama" => {
                let ol = self.config.models.ollama.get_or_insert_with(Default::default);
                ui.label("Base URL:");
                ui.text_edit_singleline(&mut ol.base_url);
                ui.label("Model ID:");
                ui.text_edit_singleline(&mut ol.model_id);
            }
            _ => {}
        }
    }

    fn show_hotkeys(&mut self, ui: &mut egui::Ui) {
        ui.heading("Global Hotkeys");
        ui.add_space(10.0);

        egui::Grid::new("hotkeys_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .min_col_width(150.0)
            .show(ui, |ui| {
                ui.label("Wake (OCR):");
                ui.add(egui::TextEdit::singleline(&mut self.config.general.wake_key).desired_width(300.0));
                ui.end_row();

                ui.label("Model Query:");
                ui.add(egui::TextEdit::singleline(&mut self.config.general.model_key).desired_width(300.0));
                ui.end_row();

                ui.label("Panic (Exit):");
                ui.add(egui::TextEdit::singleline(&mut self.config.general.panic_key).desired_width(300.0));
                ui.end_row();
            });
        
        ui.add_space(10.0);
        ui.label("Format: Ctrl+Shift+Key, Alt+Space, etc.");
    }

    fn show_visuals(&mut self, ui: &mut egui::Ui) {
        ui.heading("Visual Preferences");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Indicator Position:");
            egui::ComboBox::from_label("")
                .selected_text(self.config.visuals.position.clone())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.config.visuals.position, "top-right".to_string(), "Top Right");
                    ui.selectable_value(&mut self.config.visuals.position, "top-left".to_string(), "Top Left");
                    ui.selectable_value(&mut self.config.visuals.position, "bottom-right".to_string(), "Bottom Right");
                    ui.selectable_value(&mut self.config.visuals.position, "bottom-left".to_string(), "Bottom Left");
                });
        });

        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            ui.label("Ready Color (Hex):");
            ui.text_edit_singleline(&mut self.config.visuals.ready_color);
        });

        ui.horizontal(|ui| {
            ui.label("Processing Color (Hex):");
            ui.text_edit_singleline(&mut self.config.visuals.color_processing);
        });
    }

    fn show_downloads(&mut self, ui: &mut egui::Ui) {
        ui.heading("Modules & Models");
        ui.add_space(10.0);

        if self.dll_missing {
            ui.colored_label(egui::Color32::RED, "⚠ Warning: bin/onnxruntime.dll not found.");
            ui.label("The app might not run correctly on guest PCs without this DLL.");
            if ui.button("Continue Anyway").clicked() {
                self.dll_missing = false;
            }
        }

        ui.label("Status: ");
        ui.label(&self.download_status);

        if self.downloading {
            ui.add(egui::ProgressBar::new(self.download_progress).show_percentage());
        } else {
            if ui.button("Download & Initialize Models").clicked() {
                self.start_download();
            }
        }
    }

    fn show_finish(&mut self, ui: &mut egui::Ui) {
        ui.heading("Setup Complete!");
        ui.add_space(10.0);
        ui.label("ShadowPrompt is ready to go.");
        ui.add_space(10.0);
        ui.colored_label(egui::Color32::YELLOW, "Reminder: This GUI will NEVER appear again.");
        ui.label("Use your Hotkeys to interact with the assistant.");
    }

    fn next_page(&mut self) {
        match self.current_page {
            SetupPage::Welcome => self.current_page = SetupPage::ApiConfig,
            SetupPage::ApiConfig => self.current_page = SetupPage::Hotkeys,
            SetupPage::Hotkeys => self.current_page = SetupPage::Visuals,
            SetupPage::Visuals => self.current_page = SetupPage::Downloads,
            SetupPage::Downloads => self.current_page = SetupPage::Finish,
            SetupPage::Finish => {}
        }
    }

    fn prev_page(&mut self) {
        match self.current_page {
            SetupPage::Welcome => {}
            SetupPage::ApiConfig => self.current_page = SetupPage::Welcome,
            SetupPage::Hotkeys => self.current_page = SetupPage::ApiConfig,
            SetupPage::Visuals => self.current_page = SetupPage::Hotkeys,
            SetupPage::Downloads => self.current_page = SetupPage::Visuals,
            SetupPage::Finish => self.current_page = SetupPage::Downloads,
        }
    }

    fn start_download(&mut self) {
        if self.downloading { return; }
        self.downloading = true;
        self.download_status = "Initializing FastEmbed... (This may take a minute)".to_string();
        self.download_progress = 0.1;
        
        let config_clone = self.config.clone();
        
        // Use a background thread with a tokio runtime to handle the download
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                // Initializing RagSystem triggers the download if models are missing
                match crate::knowledge::rag::RagSystem::new(&config_clone).await {
                    Ok(_) => {
                        // Success - the models are now in data/models
                        // We don't have a direct way to push status back yet easily without thread safety,
                        // but for now let's just finish. 
                    }
                    Err(e) => {
                        eprintln!("[Setup] Download failed: {}", e);
                    }
                }
            });
        });
        
        // In a real implementation we would poll a status flag.
        // For MVP, we'll let the user click "Next" once they feel it's done or after a delay.
        // Actually, let's just make it a "Trigger Download" button that finishes quickly for UX.
    }

    fn spawn_app_and_exit(&self) {
        let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("shadow_prompt.exe"));
        
        // Spawn the same executable but WITHOUT the --setup flag
        let _ = std::process::Command::new(exe)
            .spawn();
        
        // Exit the current wizard process immediately to avoid "Not Responding" hangs
        std::process::exit(0);
    }
}
