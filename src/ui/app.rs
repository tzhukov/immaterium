use crate::config::Config;
use crate::core::Session;
use crate::shell::{OutputLine, ShellExecutor};
use egui::{CentralPanel, Context, ScrollArea, TopBottomPanel, ViewportCommand};
use tokio::sync::mpsc;

pub struct ImmateriumApp {
    config: Config,
    command_input: String,
    session: Session,
    shell_executor: ShellExecutor,
    runtime: tokio::runtime::Runtime,
    output_buffer: Vec<String>,
    is_executing: bool,
    output_receiver: Option<mpsc::UnboundedReceiver<String>>,
}

impl ImmateriumApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        // Customize egui style
        let mut style = (*cc.egui_ctx.style()).clone();
        
        // Set fonts
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(config.appearance.font_size, egui::FontFamily::Monospace),
        );
        
        cc.egui_ctx.set_style(style);

        let working_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
        let session = Session::new("default".to_string(), working_dir.clone());
        
        let shell_executor = ShellExecutor::new(config.general.default_shell.clone())
            .expect("Failed to create shell executor");

        let runtime = tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime");

        Self {
            config,
            command_input: String::new(),
            session,
            shell_executor,
            runtime,
            output_buffer: Vec::new(),
            is_executing: false,
            output_receiver: None,
        }
    }

    fn execute_command(&mut self, ctx: &Context) {
        if self.command_input.trim().is_empty() || self.is_executing {
            return;
        }

        let command = self.command_input.trim().to_string();
        self.command_input.clear();
        self.is_executing = true;
        self.output_buffer.clear();

        tracing::info!("Executing command: {}", command);

        let (output_tx, output_rx) = mpsc::unbounded_channel();
        self.output_receiver = Some(output_rx);

        let ctx_clone = ctx.clone();
        
        // Create a clone of the executor for the async task
        let executor = ShellExecutor::new(self.config.general.default_shell.clone())
            .expect("Failed to create shell executor clone");

        self.runtime.spawn(async move {
            match executor.execute(command.clone()).await {
                Ok(mut rx) => {
                    while let Some(line) = rx.recv().await {
                        match line {
                            OutputLine::Stdout(s) | OutputLine::Stderr(s) => {
                                let _ = output_tx.send(s);
                                ctx_clone.request_repaint();
                            }
                            OutputLine::Exit(code) => {
                                tracing::info!("Command exited with code: {}", code);
                                let _ = output_tx.send(format!("\n[Exit code: {}]\n", code));
                                ctx_clone.request_repaint();
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to execute command: {}", e);
                    let _ = output_tx.send(format!("Error: {}\n", e));
                    ctx_clone.request_repaint();
                }
            }
        });
    }
}

impl eframe::App for ImmateriumApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Poll output receiver for new lines
        let mut should_clear_receiver = false;
        if let Some(rx) = &mut self.output_receiver {
            while let Ok(line) = rx.try_recv() {
                self.output_buffer.push(line);
                // Check if this is an exit message to stop executing
                if self.output_buffer.last().map(|s| s.contains("[Exit code:")).unwrap_or(false) {
                    self.is_executing = false;
                    should_clear_receiver = true;
                }
            }
        }
        if should_clear_receiver {
            self.output_receiver = None;
        }
        
        // Top menu bar
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Session").clicked() {
                        tracing::info!("New session clicked");
                        ui.close_menu();
                    }
                    if ui.button("Open Session...").clicked() {
                        tracing::info!("Open session clicked");
                        ui.close_menu();
                    }
                    if ui.button("Save Session").clicked() {
                        tracing::info!("Save session clicked");
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Settings").clicked() {
                        tracing::info!("Settings clicked");
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Copy").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Paste").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Split Horizontal").clicked() {
                        tracing::info!("Split horizontal clicked");
                        ui.close_menu();
                    }
                    if ui.button("Split Vertical").clicked() {
                        tracing::info!("Split vertical clicked");
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Zoom In").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Zoom Out").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("AI", |ui| {
                    if ui.button("Suggest Command").clicked() {
                        tracing::info!("AI suggest clicked");
                        ui.close_menu();
                    }
                    if ui.button("Explain Last Command").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    
                    ui.label("Provider:");
                    for (name, provider) in &self.config.ai.providers {
                        if provider.enabled {
                            let is_selected = name == &self.config.ai.default_provider;
                            if ui.selectable_label(is_selected, name).clicked() {
                                tracing::info!("Switched AI provider to: {}", name);
                                ui.close_menu();
                            }
                        }
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Documentation").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Keyboard Shortcuts").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("About").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        // Main terminal area
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("🚀 Immaterium Terminal");
            ui.separator();
            
            // Command input area
            ui.horizontal(|ui| {
                ui.label("$");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.command_input)
                        .desired_width(f32::INFINITY)
                        .hint_text("Enter a command...")
                        .font(egui::TextStyle::Monospace),
                );
                
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.execute_command(ctx);
                    response.request_focus();
                }
                
                if self.is_executing {
                    ui.spinner();
                }
            });
            
            ui.separator();
            
            // Output area
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.set_min_height(400.0);
                    
                    if self.output_buffer.is_empty() && !self.is_executing {
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label("✨ Ready to execute commands!");
                            ui.add_space(20.0);
                            ui.label("Type a command above and press Enter");
                            ui.add_space(10.0);
                            ui.label(format!("Working directory: {}", 
                                self.session.working_directory.display()));
                        });
                    } else {
                        // Display output
                        for line in &self.output_buffer {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(line)
                                        .font(egui::FontId::monospace(self.config.appearance.font_size))
                                )
                            );
                        }
                    }
                });
        });

        // Status bar
        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Ready");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                });
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Save window state
        if let Ok(config_json) = serde_json::to_string(&self.config) {
            storage.set_string("config", config_json);
        }
    }
}
