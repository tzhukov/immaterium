use crate::config::Config;
use crate::core::{Block, BlockManager, Session};
use crate::shell::{OutputLine, ShellExecutor};
use crate::ui::BlockWidget;
use egui::{CentralPanel, Context, ScrollArea, TopBottomPanel, ViewportCommand};
use std::path::PathBuf;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct ImmateriumApp {
    config: Config,
    command_input: String,
    session: Session,
    block_manager: BlockManager,
    runtime: tokio::runtime::Runtime,
    current_block_id: Option<Uuid>,
    output_receiver: Option<mpsc::UnboundedReceiver<OutputMessage>>,
    context_menu_block: Option<Uuid>,
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

        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let session = Session::new("default".to_string(), working_dir.clone());
        
        let runtime = tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime");

        Self {
            config,
            command_input: String::new(),
            session,
            block_manager: BlockManager::new(),
            runtime,
            current_block_id: None,
            output_receiver: None,
            context_menu_block: None,
        }
    }

    fn execute_command(&mut self, ctx: &Context) {
        if self.command_input.trim().is_empty() {
            return;
        }

        // Check if there's already a running command
        if self.current_block_id.is_some() {
            tracing::warn!("Command already running, ignoring new command");
            return;
        }

        let command = self.command_input.trim().to_string();
        self.command_input.clear();

        tracing::info!("Executing command: {}", command);

        // Create a new block
        let mut block = Block::new(command.clone(), self.session.working_directory.clone());
        block.start_execution();
        let block_id = block.id;
        self.block_manager.add_block(block);
        self.current_block_id = Some(block_id);

        let (output_tx, output_rx) = mpsc::unbounded_channel();
        self.output_receiver = Some(output_rx);

        let ctx_clone = ctx.clone();
        
        // Create executor for this command
        let executor = ShellExecutor::new(self.config.general.default_shell.clone())
            .expect("Failed to create shell executor");

        self.runtime.spawn(async move {
            match executor.execute(command.clone()).await {
                Ok(mut rx) => {
                    while let Some(line) = rx.recv().await {
                        match line {
                            OutputLine::Stdout(s) | OutputLine::Stderr(s) => {
                                let _ = output_tx.send(OutputMessage::Output(s));
                                ctx_clone.request_repaint();
                            }
                            OutputLine::Exit(code) => {
                                tracing::info!("Command exited with code: {}", code);
                                let _ = output_tx.send(OutputMessage::Exit(code));
                                ctx_clone.request_repaint();
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to execute command: {}", e);
                    let _ = output_tx.send(OutputMessage::Output(format!("Error: {}\n", e)));
                    let _ = output_tx.send(OutputMessage::Exit(-1));
                    ctx_clone.request_repaint();
                }
            }
        });
    }
}

enum OutputMessage {
    Output(String),
    Exit(i32),
}

impl eframe::App for ImmateriumApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Poll output receiver for new output
        let mut should_clear_receiver = false;
        if let Some(rx) = &mut self.output_receiver {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    OutputMessage::Output(text) => {
                        if let Some(block_id) = self.current_block_id {
                            if let Some(block) = self.block_manager.get_block_mut(&block_id) {
                                block.append_output(text);
                            }
                        }
                    }
                    OutputMessage::Exit(code) => {
                        if let Some(block_id) = self.current_block_id {
                            if let Some(block) = self.block_manager.get_block_mut(&block_id) {
                                block.complete_execution(code);
                            }
                        }
                        self.current_block_id = None;
                        should_clear_receiver = true;
                    }
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
                
                if self.current_block_id.is_some() {
                    ui.spinner();
                    ui.label("Running...");
                }
            });
            
            ui.separator();
            
            // Blocks area
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.set_min_height(400.0);
                    
                    if self.block_manager.count() == 0 {
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
                        // Display all blocks
                        let blocks_to_display: Vec<_> = self.block_manager.get_blocks()
                            .iter()
                            .map(|b| b.clone())
                            .collect();
                        
                        for block in blocks_to_display {
                            let widget = BlockWidget::new(&block, self.config.appearance.font_size);
                            let block_response = widget.show(ui);
                            
                            if block_response.selected {
                                self.block_manager.select_block(block.id);
                            }
                            
                            if block_response.toggle_collapsed {
                                self.block_manager.toggle_block_collapsed(&block.id);
                            }
                            
                            if block_response.show_context_menu {
                                self.context_menu_block = Some(block.id);
                            }
                            
                            ui.add_space(self.config.appearance.block_spacing);
                        }
                    }
                });
            
            // Context menu
            if let Some(block_id) = self.context_menu_block {
                egui::Window::new("Block Actions")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        if ui.button("📋 Copy Command").clicked() {
                            if let Some(cmd) = self.block_manager.copy_block_command(&block_id) {
                                ui.output_mut(|o| o.copied_text = cmd);
                            }
                            self.context_menu_block = None;
                        }
                        
                        if ui.button("📄 Copy Output").clicked() {
                            if let Some(output) = self.block_manager.copy_block_output(&block_id) {
                                ui.output_mut(|o| o.copied_text = output);
                            }
                            self.context_menu_block = None;
                        }
                        
                        if ui.button("📑 Copy Both").clicked() {
                            if let Some(full) = self.block_manager.copy_block_full(&block_id) {
                                ui.output_mut(|o| o.copied_text = full);
                            }
                            self.context_menu_block = None;
                        }
                        
                        ui.separator();
                        
                        if ui.button("✏️ Edit & Re-run").clicked() {
                            if let Some(block) = self.block_manager.get_block(&block_id) {
                                self.command_input = block.command.clone();
                            }
                            self.context_menu_block = None;
                        }
                        
                        if ui.button("🗑️ Delete Block").clicked() {
                            self.block_manager.remove_block(&block_id);
                            self.context_menu_block = None;
                        }
                        
                        ui.separator();
                        
                        if ui.button("❌ Close").clicked() {
                            self.context_menu_block = None;
                        }
                    });
            }
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
