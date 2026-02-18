use std::sync::mpsc::{self, Receiver};
use eframe::egui;
use crate::config::Config;
use crate::tos_text::{TOS_TEXT, TOS_VERSION};
use crate::hotkey_recorder::{HotkeyRecorder, hotkey_field, validate_hotkeys};
use crate::color_picker::{color_picker, color_picker_compact};
use std::path::Path;
use crate::llm::LlmClient;

// --- Helper function to test provider connectivity ---
fn test_provider_sync(provider: &str, config: &Config) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        match LlmClient::test_provider(provider, config).await {
            Ok(response) => {
                if response.to_lowercase().contains("ok") || response.len() < 50 {
                    Ok("Connected successfully!".to_string())
                } else {
                    Ok(format!("Connected (response: {})", &response[..response.len().min(30)]))
                }
            }
            Err(e) => Err(e.to_string())
        }
    })
}

// --- Page Enum ---

#[derive(PartialEq, Clone, Copy)]
enum SetupPage {
    Landing,
    TermsOfService,
    LLMProvider,
    Features,
    Hotkeys,
    Visuals,
    Downloads,
    Credits,
}

impl SetupPage {
    fn index(&self) -> usize {
        match self {
            SetupPage::Landing => 1,
            SetupPage::TermsOfService => 2,
            SetupPage::LLMProvider => 3,
            SetupPage::Features => 4,
            SetupPage::Hotkeys => 5,
            SetupPage::Visuals => 6,
            SetupPage::Downloads => 7,
            SetupPage::Credits => 8,
        }
    }

    fn total() -> usize { 8 }

    fn title(&self) -> &'static str {
        match self {
            SetupPage::Landing => "Welcome",
            SetupPage::TermsOfService => "Terms of Service",
            SetupPage::LLMProvider => "LLM Provider",
            SetupPage::Features => "Features",
            SetupPage::Hotkeys => "Hotkey Configuration",
            SetupPage::Visuals => "Visual Preferences",
            SetupPage::Downloads => "Modules & Models",
            SetupPage::Credits => "Credits",
        }
    }
}

// --- Provider State ---

#[derive(Default)]
struct ProviderState {
    groq_enabled: bool,
    openrouter_enabled: bool,
    ollama_enabled: bool,
}

impl ProviderState {
    fn from_config(config: &Config) -> Self {
        Self {
            groq_enabled: config.models.groq.as_ref().map(|g| !g.api_key.is_empty()).unwrap_or(false),
            openrouter_enabled: config.models.openrouter.as_ref().map(|o| !o.api_key.is_empty()).unwrap_or(false),
            ollama_enabled: config.models.ollama.is_some(),
        }
    }

    fn has_at_least_one(&self) -> bool {
        self.groq_enabled || self.openrouter_enabled || self.ollama_enabled
    }
}

// --- Main Wizard Struct ---

pub struct SetupWizard {
    current_page: SetupPage,
    config: Config,

    // TOS
    tos_accepted: bool,

    // Provider
    provider_state: ProviderState,

    // Hotkeys
    wake_recorder: HotkeyRecorder,
    model_recorder: HotkeyRecorder,
    panic_recorder: HotkeyRecorder,
    hide_recorder: HotkeyRecorder,
    hotkey_error: Option<String>,

    // Downloads
    downloading: bool,
    download_progress: f32,
    download_status: String,
    download_rx: Option<Receiver<(f32, String)>>,
    download_success: bool,

    finished: bool,
}

impl SetupWizard {
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        let provider_state = ProviderState::from_config(&config);

        Self {
            current_page: SetupPage::Landing,
            config,
            tos_accepted: false,
            provider_state,
            wake_recorder: HotkeyRecorder::new(),
            model_recorder: HotkeyRecorder::new(),
            panic_recorder: HotkeyRecorder::new(),
            hide_recorder: HotkeyRecorder::new(),
            hotkey_error: None,
            downloading: false,
            download_progress: 0.0,
            download_status: "Ready to download.".to_string(),
            download_rx: None,
            download_success: false,
            finished: false,
        }
    }

    pub fn show(self) -> bool {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([520.0, 680.0])
                .with_min_inner_size([450.0, 500.0])
                .with_title("ShadowPrompt Setup")
                .with_resizable(true),
            ..Default::default()
        };

        let _ = eframe::run_native(
            "ShadowPrompt Setup",
            options,
            Box::new(|cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);
                cc.egui_ctx.set_visuals(egui::Visuals::dark());
                Ok(Box::new(self))
            }),
        );

        std::path::Path::new("config/.setup_complete").exists()
    }

    // --- Navigation ---

    fn can_go_next(&self) -> bool {
        match self.current_page {
            SetupPage::Landing => true,
            SetupPage::TermsOfService => self.tos_accepted,
            SetupPage::LLMProvider => self.provider_state.has_at_least_one(),
            SetupPage::Features => true,
            SetupPage::Hotkeys => self.hotkey_error.is_none(),
            SetupPage::Visuals => true,
            SetupPage::Downloads => self.download_success,
            SetupPage::Credits => true,
        }
    }

    fn next_page(&mut self) {
        // Validate before advancing
        if self.current_page == SetupPage::Hotkeys {
            if let Err(e) = validate_hotkeys(
                &self.config.general.wake_key,
                &self.config.general.model_key,
                &self.config.general.panic_key,
                Some(&self.config.visuals.hide_key),
            ) {
                self.hotkey_error = Some(e);
                return;
            }
            self.hotkey_error = None;
        }

        self.current_page = match self.current_page {
            SetupPage::Landing => SetupPage::TermsOfService,
            SetupPage::TermsOfService => SetupPage::LLMProvider,
            SetupPage::LLMProvider => SetupPage::Features,
            SetupPage::Features => SetupPage::Hotkeys,
            SetupPage::Hotkeys => SetupPage::Visuals,
            SetupPage::Visuals => SetupPage::Downloads,
            SetupPage::Downloads => SetupPage::Credits,
            SetupPage::Credits => SetupPage::Credits,
        };
    }

    fn prev_page(&mut self) {
        self.current_page = match self.current_page {
            SetupPage::Landing => SetupPage::Landing,
            SetupPage::TermsOfService => SetupPage::Landing,
            SetupPage::LLMProvider => SetupPage::TermsOfService,
            SetupPage::Features => SetupPage::LLMProvider,
            SetupPage::Hotkeys => SetupPage::Features,
            SetupPage::Visuals => SetupPage::Hotkeys,
            SetupPage::Downloads => SetupPage::Visuals,
            SetupPage::Credits => SetupPage::Downloads,
        };
    }
}

// --- eframe App Implementation ---

impl eframe::App for SetupWizard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll download status
        if let Some(rx) = &self.download_rx {
            while let Ok((prog, status)) = rx.try_recv() {
                self.download_progress = prog;
                self.download_status = status;
                if self.download_progress >= 1.0 {
                    self.downloading = false;
                    self.download_success = true;
                }
                if self.download_status.starts_with("Error") {
                    self.downloading = false;
                    self.download_success = false;
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // --- Header ---
            ui.vertical_centered(|ui| {
                ui.add(egui::Image::new(egui::include_image!("../assets/logo_512.png")).max_width(80.0));
                ui.add_space(4.0);
                ui.heading("ShadowPrompt Setup");
            });

            // --- Step Indicator ---
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let step_text = format!(
                    "Step {} of {} — {}",
                    self.current_page.index(),
                    SetupPage::total(),
                    self.current_page.title()
                );
                ui.label(egui::RichText::new(step_text).color(egui::Color32::GRAY).size(14.0));
            });

            // Progress bar
            let progress = self.current_page.index() as f32 / SetupPage::total() as f32;
            ui.add(egui::ProgressBar::new(progress).show_percentage().desired_height(16.0));

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);

            // --- Page Content ---
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    match self.current_page {
                        SetupPage::Landing => self.show_landing(ui),
                        SetupPage::TermsOfService => self.show_tos(ui),
                        SetupPage::LLMProvider => self.show_llm_provider(ui),
                        SetupPage::Features => self.show_features(ui),
                        SetupPage::Hotkeys => self.show_hotkeys(ui),
                        SetupPage::Visuals => self.show_visuals(ui),
                        SetupPage::Downloads => self.show_downloads(ui),
                        SetupPage::Credits => self.show_credits(ui),
                    }
                });

            // --- Footer Navigation ---
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                // Back button
                let show_back = self.current_page != SetupPage::Landing
                    && self.current_page != SetupPage::Credits
                    && !self.downloading;

                if show_back && ui.button("← Back").clicked() {
                    self.prev_page();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.current_page == SetupPage::Credits {
                        if ui.button("Start ShadowPrompt →").clicked() {
                            let _ = self.config.save();
                            let _ = Config::mark_setup_complete();
                            self.finished = true;
                            self.spawn_app_and_exit();
                        }
                    } else if self.current_page == SetupPage::TermsOfService && !self.tos_accepted {
                        if ui.button("I Decline").clicked() {
                            std::process::exit(0);
                        }
                        if ui.button("I Accept").clicked() {
                            self.tos_accepted = true;
                            self.config.general.tos_accepted = true;
                            self.config.general.tos_accepted_version = TOS_VERSION.to_string();
                            self.next_page();
                        }
                    } else {
                        let enabled = self.can_go_next() && !self.downloading;
                        if ui.add_enabled(enabled, egui::Button::new("Next →")).clicked() {
                            self.next_page();
                        }
                    }
                });
            });
        });

        // Request repaint for animations
        if self.downloading || self.wake_recorder.is_recording()
            || self.model_recorder.is_recording() || self.panic_recorder.is_recording()
            || self.hide_recorder.is_recording()
        {
            ctx.request_repaint();
        }
    }
}

// --- Page Implementations ---

impl SetupWizard {
    fn show_landing(&mut self, ui: &mut egui::Ui) {
        ui.label("Welcome to ShadowPrompt!");
        ui.add_space(8.0);

        ui.label("This wizard will guide you through the initial setup of your portable AI assistant.");
        ui.add_space(12.0);

        ui.colored_label(
            egui::Color32::YELLOW,
            egui::RichText::new("⚠ IMPORTANT: This setup runs only ONCE.")
                .strong()
        );
        ui.add_space(4.0);

        ui.label("After completing setup, ShadowPrompt will run invisibly in the background. There is no GUI by design.");
        ui.add_space(8.0);

        ui.label("To modify settings later, edit the ");
        ui.code("config/config.toml");
        ui.label(" file directly.");
        ui.add_space(12.0);

        ui.separator();
        ui.add_space(8.0);

        ui.label(egui::RichText::new("Portable Design").strong());
        ui.label("ShadowPrompt is designed to be fully contained. After setup, you can place the entire folder on a USB drive and run it on any Windows 10/11 computer.");
    }

    fn show_tos(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Please read and accept the Terms of Service to continue.").strong());
        ui.add_space(8.0);

        // Scrollable TOS text
        egui::ScrollArea::vertical()
            .max_height(280.0)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut TOS_TEXT.to_string())
                        .desired_width(f32::INFINITY)
                        .interactive(false)
                        .font(egui::TextStyle::Monospace)
                );
            });

        ui.add_space(8.0);

        if self.tos_accepted {
            ui.colored_label(egui::Color32::GREEN, "✓ Terms accepted");
        }
    }

    fn show_llm_provider(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Configure at least one LLM provider to continue.").strong());
        ui.add_space(4.0);
        ui.label("You can configure multiple providers. ShadowPrompt will automatically fall back to the next available provider if one fails.");
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Tip: Click 'Test Connection' to verify your API key works.").color(egui::Color32::GRAY).small());
        ui.add_space(12.0);

        // --- Groq ---
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.provider_state.groq_enabled, "");
                ui.label(egui::RichText::new("Groq").strong());
                ui.label(egui::RichText::new("(Recommended)").color(egui::Color32::GREEN).small());
            });
            ui.label("Ultra-fast inference with a generous free tier.");

            if self.provider_state.groq_enabled {
                let groq = self.config.models.groq.get_or_insert_with(Default::default);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("API Key:");
                    ui.add(egui::TextEdit::singleline(&mut groq.api_key).desired_width(250.0).password(true));
                });
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.add(egui::TextEdit::singleline(&mut groq.model_id).desired_width(200.0));
                });
                ui.add_space(4.0);
                
                // Test Connection button
                let groq_enabled = self.provider_state.groq_enabled;
                if groq_enabled {
                    if ui.button("Test Groq Connection").clicked() {
                        let config = self.config.clone();
                        std::thread::spawn(move || {
                            let result = test_provider_sync("groq", &config);
                            match &result {
                                Ok(msg) => log::info!("Groq test: {}", msg),
                                Err(e) => log::error!("Groq test failed: {}", e),
                            }
                        });
                    }
                    ui.label(egui::RichText::new("(Check logs for result)").color(egui::Color32::GRAY).small());
                }
            }
        });

        ui.add_space(8.0);

        // --- OpenRouter ---
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.provider_state.openrouter_enabled, "");
                ui.label(egui::RichText::new("OpenRouter").strong());
            });
            ui.label("Wide selection of models from various providers.");

            if self.provider_state.openrouter_enabled {
                let or = self.config.models.openrouter.get_or_insert_with(Default::default);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("API Key:");
                    ui.add(egui::TextEdit::singleline(&mut or.api_key).desired_width(250.0).password(true));
                });
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.add(egui::TextEdit::singleline(&mut or.model_id).desired_width(200.0));
                });
                ui.add_space(4.0);
                if ui.button("Test OpenRouter Connection").clicked() {
                    let config = self.config.clone();
                    std::thread::spawn(move || {
                        let result = test_provider_sync("openrouter", &config);
                        match &result {
                            Ok(msg) => log::info!("OpenRouter test: {}", msg),
                            Err(e) => log::error!("OpenRouter test failed: {}", e),
                        }
                    });
                }
                ui.label(egui::RichText::new("(Check logs for result)").color(egui::Color32::GRAY).small());
            }
        });

        ui.add_space(8.0);

        // --- Ollama ---
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.provider_state.ollama_enabled, "");
                ui.label(egui::RichText::new("Ollama").strong());
                ui.label(egui::RichText::new("⚠ Developer Only").color(egui::Color32::YELLOW).small());
            });
            ui.label("Local models. Requires Ollama server running separately.");

            if self.provider_state.ollama_enabled {
                let ol = self.config.models.ollama.get_or_insert_with(Default::default);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Base URL:");
                    ui.add(egui::TextEdit::singleline(&mut ol.base_url).desired_width(200.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.add(egui::TextEdit::singleline(&mut ol.model_id).desired_width(150.0));
                });
                ui.add_space(4.0);
                if ui.button("Test Ollama Connection").clicked() {
                    let config = self.config.clone();
                    std::thread::spawn(move || {
                        let result = test_provider_sync("ollama", &config);
                        match &result {
                            Ok(msg) => log::info!("Ollama test: {}", msg),
                            Err(e) => log::error!("Ollama test failed: {}", e),
                        }
                    });
                }
                ui.label(egui::RichText::new("(Check logs for result)").color(egui::Color32::GRAY).small());
            }
        });

        ui.add_space(12.0);

        if !self.provider_state.has_at_least_one() {
            ui.colored_label(egui::Color32::RED, "⚠ Please enable and configure at least one provider.");
        }
    }

    fn show_features(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("ShadowPrompt includes the following features:").strong());
        ui.add_space(12.0);

        // Web Search
        ui.group(|ui| {
            ui.label(egui::RichText::new("🔍 Web Search").strong());
            ui.label("LLM models can search the web, significantly improving accuracy for current information.");
            ui.add_space(8.0);
            ui.label("Search Engine:");
            ui.add_space(4.0);
            ui.radio_value(&mut self.config.search.engine, "serper".to_string(), "Serper.dev (Recommended - reliable, $0.50/1k queries)");
            ui.radio_value(&mut self.config.search.engine, "duckduckgo".to_string(), "DuckDuckGo (Free - may rate-limit)");
            
            if self.config.search.engine == "serper" {
                ui.add_space(8.0);
                let api_key = self.config.search.serper_api_key.get_or_insert_with(String::new);
                ui.horizontal(|ui| {
                    ui.label("Serper API Key:");
                    ui.add(egui::TextEdit::singleline(api_key).desired_width(250.0).password(true));
                });
                ui.label(egui::RichText::new("Get your free API key at serper.dev").color(egui::Color32::GRAY).small());
            }
        });

        ui.add_space(8.0);

        // Local RAG
        ui.group(|ui| {
            ui.label(egui::RichText::new("📚 Local RAG").strong());
            ui.label("Place documents in the knowledge/ folder. The AI can retrieve relevant context from your local files.");
            ui.colored_label(egui::Color32::YELLOW, "⚠ Too many documents may slow down responses.");
            ui.add_space(4.0);
            if ui.button("📂 View Folder").clicked() {
                let knowledge_path = crate::config::get_exe_dir().join("knowledge");
                let _ = std::fs::create_dir_all(&knowledge_path);
                let _ = open::that(&knowledge_path);
            }
        });

        ui.add_space(8.0);

        // Auto LLM Selection
        ui.group(|ui| {
            ui.label(egui::RichText::new("🔄 Auto-LLM Fallback").strong());
            ui.label("If your primary provider hits rate limits, ShadowPrompt automatically switches to the next available provider.");
            ui.label(egui::RichText::new("Priority: Groq → OpenRouter → Ollama").color(egui::Color32::GRAY).small());
        });

        ui.add_space(8.0);

        // Hallucinations Warning
        ui.group(|ui| {
            ui.label(egui::RichText::new("⚠ AI Limitations").strong().color(egui::Color32::YELLOW));
            ui.label("LLMs can produce incorrect or fabricated information (hallucinations). This is inherent to AI technology.");
            ui.label("For better accuracy, use smarter models and enable Web Search.");
        });
    }

    fn show_hotkeys(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Configure your global hotkeys.").strong());
        ui.add_space(4.0);
        ui.label("Click 'Record' and press your desired key combination within 5 seconds.");
        ui.add_space(12.0);

        // Hotkey fields
        hotkey_field(ui, "Wake (OCR):", &mut self.config.general.wake_key, &mut self.wake_recorder, "wake");
        ui.add_space(8.0);

        hotkey_field(ui, "Model Query:", &mut self.config.general.model_key, &mut self.model_recorder, "model");
        ui.add_space(8.0);

        hotkey_field(ui, "Panic (Exit):", &mut self.config.general.panic_key, &mut self.panic_recorder, "panic");
        ui.add_space(8.0);

        hotkey_field(ui, "Hide Graphics:", &mut self.config.visuals.hide_key, &mut self.hide_recorder, "hide");
        ui.add_space(12.0);

        // Validation error
        if let Some(ref error) = self.hotkey_error {
            ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        ui.label(egui::RichText::new("Hotkey Tips:").strong());
        ui.label("• Use combinations like Ctrl+Shift+Space");
        ui.label("• Avoid common shortcuts (Ctrl+C, Ctrl+V)");
        ui.label("• Each hotkey must be unique");
    }

    fn show_visuals(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Customize the visual indicators.").strong());
        ui.add_space(4.0);
        ui.label("ShadowPrompt displays small pixel indicators to show its status.");
        ui.add_space(12.0);

        // Position
        ui.horizontal(|ui| {
            ui.label("Indicator Position:");
            egui::ComboBox::from_id_salt("indicator_position")
                .selected_text(&self.config.visuals.position)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.config.visuals.position, "top-right".to_string(), "Top Right");
                    ui.selectable_value(&mut self.config.visuals.position, "top-left".to_string(), "Top Left");
                    ui.selectable_value(&mut self.config.visuals.position, "bottom-right".to_string(), "Bottom Right");
                    ui.selectable_value(&mut self.config.visuals.position, "bottom-left".to_string(), "Bottom Left");
                });
        });

        ui.add_space(12.0);

        // Status Colors
        ui.label(egui::RichText::new("Status Colors").strong());
        ui.add_space(4.0);

        color_picker(ui, "Ready:", &mut self.config.visuals.ready_color);
        ui.add_space(4.0);
        color_picker(ui, "Processing:", &mut self.config.visuals.color_processing);

        ui.add_space(16.0);

        // MCQ Colors
        ui.label(egui::RichText::new("Multiple Choice Indicator Colors").strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            color_picker_compact(ui, "A:", &mut self.config.visuals.color_mcq_a);
            ui.add_space(16.0);
            color_picker_compact(ui, "B:", &mut self.config.visuals.color_mcq_b);
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            color_picker_compact(ui, "C:", &mut self.config.visuals.color_mcq_c);
            ui.add_space(16.0);
            color_picker_compact(ui, "D:", &mut self.config.visuals.color_mcq_d);
        });

        ui.add_space(16.0);

        // True/False Colors
        ui.label(egui::RichText::new("True/False Indicator Colors").strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            color_picker_compact(ui, "True:", &mut self.config.visuals.color_true);
            ui.add_space(16.0);
            color_picker_compact(ui, "False:", &mut self.config.visuals.color_false);
        });

        ui.add_space(16.0);

        // Text Overlay Settings
        ui.label(egui::RichText::new("Text Answer Display").strong());
        ui.add_space(4.0);

        ui.checkbox(&mut self.config.visuals.text_overlay_enabled, "Show answer text at bottom-right");
        
        if self.config.visuals.text_overlay_enabled {
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label("Position:");
                egui::ComboBox::from_id_salt("text_overlay_position")
                    .selected_text(&self.config.visuals.text_overlay_position)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.visuals.text_overlay_position, "bottom-right".to_string(), "Bottom Right");
                        ui.selectable_value(&mut self.config.visuals.text_overlay_position, "top-right".to_string(), "Top Right");
                        ui.selectable_value(&mut self.config.visuals.text_overlay_position, "bottom-left".to_string(), "Bottom Left");
                        ui.selectable_value(&mut self.config.visuals.text_overlay_position, "top-left".to_string(), "Top Left");
                    });
            });

            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                ui.label("Font Size:");
                ui.add(egui::Slider::new(&mut self.config.visuals.text_overlay_font_size, 8..=48).text(""));
            });

            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                ui.label("Background Opacity:");
                ui.add(egui::Slider::new(&mut self.config.visuals.text_overlay_bg_opacity, 50..=255).text(""));
            });
            
            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                ui.label("Text Opacity:");
                ui.add(egui::Slider::new(&mut self.config.visuals.text_overlay_text_opacity, 50..=255).text(""));
            });
        }
    }

    fn show_downloads(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Required Downloads").strong());
        ui.add_space(4.0);
        ui.label("ShadowPrompt needs to download embedding models for local RAG functionality.");
        ui.add_space(12.0);

        ui.label(format!("Status: {}", self.download_status));
        ui.add_space(8.0);

        if self.downloading {
            ui.add(egui::ProgressBar::new(self.download_progress).show_percentage().animate(true));
            ui.add_space(8.0);
            ui.spinner();
        } else if self.download_success {
            ui.colored_label(egui::Color32::GREEN, "✓ Downloads complete! You may proceed.");
        } else {
            let button_label = if self.download_status.starts_with("Error") {
                "Retry Download"
            } else {
                "Download Models"
            };

            if ui.button(button_label).clicked() {
                self.start_download();
            }

            ui.add_space(8.0);
            ui.label(egui::RichText::new("This download is required to complete setup.").color(egui::Color32::GRAY).small());
        }
    }

    fn show_credits(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("Thank You!").strong().size(20.0));
            ui.add_space(8.0);
            ui.label("ShadowPrompt setup is complete.");
        });

        ui.add_space(16.0);

        // Quick Start Summary
        ui.group(|ui| {
            ui.label(egui::RichText::new("📋 Quick Start Summary").strong());
            ui.add_space(8.0);

            egui::Grid::new("hotkey_summary")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Wake (OCR):");
                    ui.code(&self.config.general.wake_key);
                    ui.end_row();

                    ui.label("Model Query:");
                    ui.code(&self.config.general.model_key);
                    ui.end_row();

            ui.label("Panic (Exit):");
                    ui.code(&self.config.general.panic_key);
                    ui.end_row();

                    ui.label("Hide Graphics:");
                    ui.code(&self.config.visuals.hide_key);
                    ui.end_row();
                });
        });

        ui.add_space(16.0);

        // Credits
        ui.separator();
        ui.add_space(8.0);

        ui.vertical_centered(|ui| {
            ui.label("Developed by");
            ui.label(egui::RichText::new("Hyowon Bernabe").strong());
            ui.add_space(4.0);
            ui.hyperlink_to("www.hyowonbernabe.me", "https://www.hyowonbernabe.me");
            ui.add_space(8.0);
            ui.hyperlink_to("GitHub Repository", "https://github.com/hyowonbernabe/ShadowPrompt");
        });
    }

    // --- Download Logic ---

    fn start_download(&mut self) {
        if self.downloading { return; }

        self.downloading = true;
        self.download_status = "Initializing...".to_string();
        self.download_progress = 0.0;
        self.download_success = false;

        let (tx, rx) = mpsc::channel();
        self.download_rx = Some(rx);

        let config_clone = self.config.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                // Step 1: Check and download onnxruntime.dll
                let dll_path = Path::new("onnxruntime.dll");
                let bin_dll_path = Path::new("bin/onnxruntime.dll");

                if !dll_path.exists() && !bin_dll_path.exists() {
                    let _ = tx.send((0.1, "Downloading ONNX Runtime DLL...".to_string()));

                    if let Err(e) = download_onnx_dll().await {
                        let _ = tx.send((0.0, format!("Error downloading DLL: {}", e)));
                        return;
                    }
                }

                // Step 2: Initialize FastEmbed models
                let _ = tx.send((0.4, "Downloading embedding models...".to_string()));

                let rag_system = crate::knowledge::rag::RagSystem::new(&config_clone).await;
                if rag_system.is_operational() {
                     let _ = tx.send((1.0, "Download complete!".to_string()));
                } else {
                     let error_msg = rag_system.get_init_error().unwrap_or("Unknown error");
                     let _ = tx.send((0.0, format!("Error: {}", error_msg)));
                }
            });
        });
    }

    fn spawn_app_and_exit(&self) {
        let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("shadow_prompt.exe"));

        let _ = std::process::Command::new(exe).spawn();

        std::process::exit(0);
    }
}

// --- Download Helper ---

async fn download_onnx_dll() -> anyhow::Result<()> {
    use std::io::Write;

    // ONNX Runtime 1.16.3 Windows x64
    let url = "https://github.com/microsoft/onnxruntime/releases/download/v1.16.3/onnxruntime-win-x64-1.16.3.zip";

    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    // Extract DLL from zip
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();

        if name.ends_with("onnxruntime.dll") {
            let mut out = std::fs::File::create("onnxruntime.dll")?;
            std::io::copy(&mut file, &mut out)?;
            out.flush()?;
            break;
        }
    }

    Ok(())
}
