use crate::ai::{build_minimal_context, AiEngine, ChatRequest, ContextConfig};
use crate::ai::providers::{GroqProvider, OllamaProvider, OpenAiProvider};
use crate::config::Config;
use crate::core::{Block, BlockManager, Database, ExportedSession, Session, SessionManager};
use crate::shell::{OutputLine, ShellExecutor};
use crate::theme::ThemeLoader;
use crate::ui::{AiAction, AiPanel, BlockWidget};
use egui::{CentralPanel, Context, ScrollArea, SidePanel, TopBottomPanel, ViewportCommand};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct ImmateriumApp {
    config: Config,
    command_input: String,
    session: Session,
    block_manager: BlockManager,
    runtime: tokio::runtime::Runtime,
    session_manager: Option<SessionManager>,
    current_block_id: Option<Uuid>,
    output_receiver: Option<mpsc::UnboundedReceiver<OutputMessage>>,
    ai_receiver: Option<mpsc::UnboundedReceiver<AiMessage>>,
    context_menu_block: Option<Uuid>,
    last_save: Instant,
    save_needed: bool,
    // Session UI state
    show_session_list: bool,
    show_new_session_dialog: bool,
    new_session_name: String,
    available_sessions: Vec<crate::core::SessionInfo>,
    show_export_dialog: bool,
    // Theme
    theme_loader: ThemeLoader,
    show_theme_selector: bool,
    // AI
    ai_panel: AiPanel,
    ai_engine: Option<Arc<AiEngine>>,
    // Natural language command generation state
    original_nl_input: String,
    is_generating_command: bool,
}

impl ImmateriumApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        // Initialize theme loader
        let mut theme_loader = ThemeLoader::new();
        
        // Try to load custom themes from config directory
        if let Some(config_dir) = directories::ProjectDirs::from("com", "immaterium", "immaterium") {
            let themes_dir = config_dir.config_dir().join("themes");
            if let Err(e) = theme_loader.load_from_directory(&themes_dir) {
                tracing::warn!("Failed to load custom themes: {}", e);
            }
        }
        
        // Apply theme to egui context
        theme_loader.apply_to_egui(&cc.egui_ctx);
        
        // Customize egui style
        let mut style = (*cc.egui_ctx.style()).clone();
        
        // Set fonts
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(config.appearance.font_size, egui::FontFamily::Monospace),
        );
        
        cc.egui_ctx.set_style(style);

        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let mut session = Session::new("default".to_string(), working_dir.clone());
        
        let runtime = tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime");

        // Initialize database and session manager
        let session_manager = runtime.block_on(async {
            let db_path = PathBuf::from("immaterium.db");
            match Database::new(db_path).await {
                Ok(db) => {
                    tracing::info!("Database initialized successfully");
                    
                    match SessionManager::new(db).await {
                        Ok(sm) => {
                            // Try to load active session
                            match sm.get_active_session().await {
                                Ok(Some(loaded_session)) => {
                                    tracing::info!("Loaded active session: {}", loaded_session.name);
                                    session = loaded_session;
                                }
                                Ok(None) => {
                                    tracing::info!("No active session found, creating new one");
                                    // Create new session in database
                                    if let Err(e) = sm.create_session(&session).await {
                                        tracing::error!("Failed to create session: {}", e);
                                    } else if let Err(e) = sm.set_active_session(&session.id).await {
                                        tracing::error!("Failed to set active session: {}", e);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to load active session: {}", e);
                                }
                            }
                            Some(sm)
                        }
                        Err(e) => {
                            tracing::error!("Failed to create SessionManager: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to initialize database: {}", e);
                    None
                }
            }
        });

        // Populate block manager with loaded blocks
        let mut block_manager = BlockManager::new();
        for block in &session.blocks {
            block_manager.add_block(block.clone());
        }

        // Initialize AI engine before moving config
        let ai_engine = Self::initialize_ai_engine(&config).map(Arc::new);

        Self {
            config,
            command_input: String::new(),
            session,
            block_manager,
            runtime,
            session_manager,
            current_block_id: None,
            output_receiver: None,
            context_menu_block: None,
            last_save: Instant::now(),
            save_needed: false,
            show_session_list: false,
            show_new_session_dialog: false,
            new_session_name: String::new(),
            available_sessions: Vec::new(),
            show_export_dialog: false,
            theme_loader,
            show_theme_selector: false,
            ai_panel: AiPanel::new(),
            ai_engine,
            ai_receiver: None,
            original_nl_input: String::new(),
            is_generating_command: false,
        }
    }

    /// Initialize AI engine with configured providers
    fn initialize_ai_engine(config: &Config) -> Option<AiEngine> {
        let mut engine = AiEngine::new();
        let mut providers_registered = 0;

        // Initialize Ollama provider
        if let Some(ollama_config) = config.ai.providers.get("ollama") {
            if ollama_config.enabled {
                let base_url = ollama_config.base_url.clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string());
                
                let provider = OllamaProvider::new(base_url, ollama_config.model.clone());
                engine.register_provider(Arc::new(provider));
                providers_registered += 1;
                tracing::info!("Registered Ollama provider");
            }
        }

        // Initialize OpenAI provider
        if let Some(openai_config) = config.ai.providers.get("openai") {
            if openai_config.enabled {
                if let Some(api_key) = &openai_config.api_key {
                    // Expand environment variables
                    let api_key = shellexpand::env(api_key)
                        .unwrap_or(std::borrow::Cow::Borrowed(api_key))
                        .to_string();
                    
                    if !api_key.is_empty() && !api_key.starts_with("${") {
                        let provider = OpenAiProvider::new(api_key, openai_config.model.clone());
                        engine.register_provider(Arc::new(provider));
                        providers_registered += 1;
                        tracing::info!("Registered OpenAI provider");
                    } else {
                        tracing::warn!("OpenAI API key not set or is a placeholder");
                    }
                } else {
                    tracing::warn!("OpenAI enabled but no API key configured");
                }
            }
        }

        // Initialize Groq provider
        if let Some(groq_config) = config.ai.providers.get("groq") {
            if groq_config.enabled {
                if let Some(api_key) = &groq_config.api_key {
                    // Expand environment variables
                    let api_key = shellexpand::env(api_key)
                        .unwrap_or(std::borrow::Cow::Borrowed(api_key))
                        .to_string();
                    
                    if !api_key.is_empty() && !api_key.starts_with("${") {
                        let base_url = groq_config.base_url.clone()
                            .unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string());
                        
                        let provider = GroqProvider::new(api_key, groq_config.model.clone());
                        engine.register_provider(Arc::new(provider));
                        providers_registered += 1;
                        tracing::info!("Registered Groq provider");
                    } else {
                        tracing::warn!("Groq API key not set or is a placeholder");
                    }
                } else {
                    tracing::warn!("Groq enabled but no API key configured");
                }
            }
        }

        // Set default provider
        if providers_registered > 0 {
            if let Err(e) = engine.set_default_provider(&config.ai.default_provider) {
                tracing::warn!("Failed to set default provider: {}, using first available", e);
            }
            Some(engine)
        } else {
            tracing::warn!("No AI providers registered");
            None
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

        let input = self.command_input.trim().to_string();
        
        // Check operation mode and handle accordingly
        use crate::config::OperationMode;
        match self.config.ai.operation_mode {
            OperationMode::TerminalOnly => {
                // Mode 1: Always execute as shell command
                self.execute_shell_command(input, ctx);
                self.command_input.clear();
            }
            OperationMode::AiPromptOnly => {
                // Mode 2: Always treat as AI prompt for command generation
                tracing::info!("AI Prompt mode: converting to command: {}", input);
                self.convert_natural_language_to_command(input, ctx);
                self.command_input.clear();
            }
            OperationMode::Hybrid => {
                // Mode 3: Auto-detect if it's NL or shell command
                if self.config.ai.enable_suggestions && self.is_natural_language(&input) {
                    tracing::info!("Detected natural language input, converting to command: {}", input);
                    self.convert_natural_language_to_command(input, ctx);
                } else {
                    self.execute_shell_command(input, ctx);
                }
                self.command_input.clear();
            }
        }
    }

    /// Detect if input is natural language vs a shell command
    fn is_natural_language(&self, input: &str) -> bool {
        let input_lower = input.to_lowercase();
        
        // If it starts with common shell patterns, it's a command
        let shell_patterns = [
            "cd ", "ls ", "cat ", "grep ", "find ", "echo ", "pwd", "mkdir ",
            "rm ", "cp ", "mv ", "chmod ", "chown ", "ps ", "kill ", "sudo ",
            "apt ", "yum ", "dnf ", "pacman ", "git ", "docker ", "npm ", "cargo ",
            "./", "../", "~/", "/", "|", "&&", "||", ">", "<", "$(", "`",
        ];
        
        for pattern in &shell_patterns {
            if input.starts_with(pattern) || input.contains(pattern) {
                return false;
            }
        }
        
        // If it contains question words or is a sentence, likely natural language
        let nl_indicators = [
            "how do i", "how to", "show me", "list all", "find all", "what is",
            "create a", "delete the", "remove all", "search for", "get the",
            "install", "uninstall", "update", "upgrade", "check", "help",
        ];
        
        for indicator in &nl_indicators {
            if input_lower.contains(indicator) {
                return true;
            }
        }
        
        // If it's a question or has multiple words without shell syntax, likely NL
        input.contains('?') || (input.split_whitespace().count() > 2 && !input.contains('/'))
    }

    /// Convert natural language to shell command using AI
    fn convert_natural_language_to_command(&mut self, nl_input: String, ctx: &Context) {
        if self.ai_engine.is_none() {
            tracing::warn!("AI engine not available, executing as regular command");
            self.execute_shell_command(nl_input, ctx);
            return;
        }

        self.original_nl_input = nl_input.clone();
        self.is_generating_command = true;
        
        let engine = self.ai_engine.as_ref().unwrap().clone();
        let ctx_clone = ctx.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        self.ai_receiver = Some(rx);
        
        // Get current provider and model
        let provider_name = self.ai_panel.selected_provider().to_string();
        let model = self.ai_panel.selected_model().to_string();
        
        if model.is_empty() {
            tracing::error!("No AI model selected");
            self.is_generating_command = false;
            return;
        }
        
        // Build context for command generation
        let system_prompt = "You are a helpful shell command generator. Convert natural language requests into valid bash commands. \
                            Reply ONLY with the shell command, no explanations, no markdown, no code blocks. \
                            If the request is ambiguous, choose the most common interpretation.";
        
        let user_prompt = format!("Convert this request to a bash command: {}", nl_input);
        
        let request = ChatRequest::new(model)
            .with_system_message(system_prompt.to_string())
            .with_user_message(user_prompt);
        
        self.runtime.spawn(async move {
            match engine.chat_completion_with_provider(&provider_name, request).await {
                Ok(response) => {
                    let command = response.content.trim().to_string();
                    tracing::info!("AI generated command: {}", command);
                    let _ = tx.send(AiMessage::CommandGenerated(command));
                    ctx_clone.request_repaint();
                }
                Err(e) => {
                    tracing::error!("Failed to generate command: {}", e);
                    let _ = tx.send(AiMessage::Error(format!("Failed to generate command: {}", e)));
                    ctx_clone.request_repaint();
                }
            }
        });
    }

    fn execute_shell_command(&mut self, command: String, ctx: &Context) {
        tracing::info!("Executing command: {}", command);

        // Create a new block
        let mut block = Block::new(command.clone(), self.session.working_directory.clone());
        block.start_execution();
        let block_id = block.id;
        self.block_manager.add_block(block);
        self.current_block_id = Some(block_id);
        self.save_needed = true; // Mark that we need to save

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

    fn auto_save(&mut self) {
        // Check if enough time has elapsed since last save
        let save_interval = Duration::from_secs(self.config.general.auto_save_interval);
        if self.last_save.elapsed() < save_interval || !self.save_needed {
            return;
        }

        if let Some(ref session_manager) = self.session_manager {
            let session_id = self.session.id;
            let blocks: Vec<_> = self.block_manager.get_blocks().iter().cloned().collect();
            
            let session_manager = session_manager.clone();
            self.runtime.spawn(async move {
                // Save all blocks
                for (index, block) in blocks.iter().enumerate() {
                    if let Err(e) = session_manager.save_block(&session_id, block, index as i32).await {
                        tracing::error!("Failed to save block: {}", e);
                    }
                }
                
                // Update session timestamp
                if let Err(e) = session_manager.touch_session(&session_id).await {
                    tracing::error!("Failed to update session timestamp: {}", e);
                } else {
                    tracing::debug!("Auto-saved session {}", session_id);
                }
            });
            
            self.last_save = Instant::now();
            self.save_needed = false;
        }
    }

    fn load_available_sessions(&mut self) {
        if let Some(ref session_manager) = self.session_manager {
            let session_manager = session_manager.clone();
            let runtime = &self.runtime;
            
            if let Ok(sessions) = runtime.block_on(async {
                session_manager.list_sessions().await
            }) {
                self.available_sessions = sessions;
            }
        }
    }

    fn switch_to_session(&mut self, session_id: Uuid) {
        if let Some(ref session_manager) = self.session_manager {
            let session_manager = session_manager.clone();
            
            match self.runtime.block_on(async {
                session_manager.load_session(&session_id).await
            }) {
                Ok(loaded_session) => {
                    // Save current session first
                    self.auto_save();
                    
                    // Switch to new session
                    self.session = loaded_session;
                    self.block_manager = BlockManager::new();
                    for block in &self.session.blocks {
                        self.block_manager.add_block(block.clone());
                    }
                    
                    // Set as active
                    let _ = self.runtime.block_on(async {
                        session_manager.set_active_session(&session_id).await
                    });
                    
                    tracing::info!("Switched to session: {}", self.session.name);
                }
                Err(e) => {
                    tracing::error!("Failed to load session: {}", e);
                }
            }
        }
    }

    fn create_new_session(&mut self, name: String) {
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let new_session = Session::new(name, working_dir);
        
        if let Some(ref session_manager) = self.session_manager {
            let session_manager = session_manager.clone();
            let session_clone = new_session.clone();
            
            match self.runtime.block_on(async {
                session_manager.create_session(&session_clone).await
            }) {
                Ok(_) => {
                    // Switch to the new session
                    self.switch_to_session(new_session.id);
                }
                Err(e) => {
                    tracing::error!("Failed to create session: {}", e);
                }
            }
        }
    }

    fn handle_ai_action(&mut self, action: AiAction, ctx: &Context) {
        match action {
            AiAction::ProviderChanged(provider) => {
                tracing::info!("AI provider changed to: {}", provider);
                self.ai_panel.set_selected_provider(provider.clone());
                // Request model list load
                self.handle_ai_action(AiAction::LoadModels, ctx);
            }
            AiAction::LoadModels => {
                tracing::info!("Loading AI models...");
                
                if let Some(engine) = &self.ai_engine {
                    let provider_name = self.ai_panel.selected_provider().to_string();
                    
                    if let Some(provider) = engine.get_provider(&provider_name) {
                        let provider_clone = provider.clone();
                        let ctx_clone = ctx.clone();
                        
                        // Create channel for receiving models
                        let (tx, rx) = mpsc::unbounded_channel();
                        self.ai_receiver = Some(rx);
                        
                        self.runtime.spawn(async move {
                            match provider_clone.list_models().await {
                                Ok(models) => {
                                    tracing::info!("Loaded {} models from {}", models.len(), provider_name);
                                    let _ = tx.send(AiMessage::ModelsLoaded(models));
                                    ctx_clone.request_repaint();
                                }
                                Err(e) => {
                                    tracing::error!("Failed to load models: {}", e);
                                    let _ = tx.send(AiMessage::Error(format!("Failed to load models: {}", e)));
                                    ctx_clone.request_repaint();
                                }
                            }
                        });
                    }
                } else {
                    tracing::warn!("No AI engine available");
                    // Set some default models for offline use
                    let models = vec![
                        "qwen2.5-coder:3b".to_string(),
                        "qwen2.5-coder:7b".to_string(),
                    ];
                    self.ai_panel.set_available_models(models);
                }
            }
            AiAction::SendPrompt(prompt) => {
                tracing::info!("Sending prompt to AI: {}", prompt);
                
                // Add to conversation
                self.ai_panel.add_user_message(prompt.clone());
                self.ai_panel.start_streaming();
                
                // Build context from recent blocks if enabled
                let blocks: Vec<Block> = self.block_manager.get_blocks()
                    .iter()
                    .cloned()
                    .collect();
                
                let context = if self.ai_panel.include_context {
                    build_minimal_context(&blocks, &prompt, self.ai_panel.context_blocks)
                } else {
                    prompt.clone()
                };
                
                // Send to AI engine
                if let Some(engine) = &self.ai_engine {
                    let provider_name = self.ai_panel.selected_provider().to_string();
                    let model = self.ai_panel.selected_model().to_string();
                    
                    if model.is_empty() {
                        tracing::error!("No model selected");
                        self.ai_panel.set_response("Error: No model selected".to_string());
                        return;
                    }
                    
                    let engine_clone = engine.clone();
                    let ctx_clone = ctx.clone();
                    
                    // Create channel for receiving AI response
                    let (tx, rx) = mpsc::unbounded_channel();
                    self.ai_receiver = Some(rx);
                    
                    // Create chat request
                    let request = ChatRequest::new(model)
                        .with_user_message(context);
                    
                    self.runtime.spawn(async move {
                        match engine_clone.chat_completion_with_provider(&provider_name, request).await {
                            Ok(response) => {
                                tracing::info!("Received AI response: {} chars", response.content.len());
                                let _ = tx.send(AiMessage::Response(response.content));
                                ctx_clone.request_repaint();
                            }
                            Err(e) => {
                                tracing::error!("AI request failed: {}", e);
                                let _ = tx.send(AiMessage::Error(format!("AI request failed: {}", e)));
                                ctx_clone.request_repaint();
                            }
                        }
                    });
                } else {
                    tracing::warn!("No AI engine available");
                    self.ai_panel.set_response("Error: AI engine not initialized".to_string());
                }
                
                ctx.request_repaint();
            }
        }
    }
}

enum OutputMessage {
    Output(String),
    Exit(i32),
}

enum AiMessage {
    Response(String),
    StreamChunk(String),
    Error(String),
    ModelsLoaded(Vec<String>),
    CommandGenerated(String), // Generated shell command from natural language
}

impl eframe::App for ImmateriumApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Auto-save session periodically
        self.auto_save();
        
        // Poll output receiver for new output
        let mut should_clear_receiver = false;
        if let Some(rx) = &mut self.output_receiver {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    OutputMessage::Output(text) => {
                        if let Some(block_id) = self.current_block_id {
                            if let Some(block) = self.block_manager.get_block_mut(&block_id) {
                                block.append_output(text);
                                self.save_needed = true; // Mark for save when output changes
                            }
                        }
                    }
                    OutputMessage::Exit(code) => {
                        if let Some(block_id) = self.current_block_id {
                            if let Some(block) = self.block_manager.get_block_mut(&block_id) {
                                block.complete_execution(code);
                                self.save_needed = true; // Save when command completes
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
        
        // Poll AI receiver for AI responses
        if let Some(rx) = &mut self.ai_receiver {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    AiMessage::Response(content) => {
                        self.ai_panel.set_response(content.clone());
                        self.ai_panel.add_assistant_message(content);
                    }
                    AiMessage::StreamChunk(chunk) => {
                        self.ai_panel.append_response(chunk);
                    }
                    AiMessage::Error(err) => {
                        self.ai_panel.set_response(format!("Error: {}", err));
                        self.ai_panel.stop_streaming();
                        self.is_generating_command = false;
                    }
                    AiMessage::ModelsLoaded(models) => {
                        self.ai_panel.set_available_models(models);
                    }
                    AiMessage::CommandGenerated(command) => {
                        // Create a pending approval block instead of showing modal
                        let block = Block::new_pending_approval(
                            self.original_nl_input.clone(),
                            command,
                            self.session.working_directory.clone(),
                        );
                        self.block_manager.add_block(block);
                        self.is_generating_command = false;
                        self.original_nl_input.clear();
                    }
                }
            }
        }
        
        // Top menu bar
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Session").clicked() {
                        self.show_new_session_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("Open Session...").clicked() {
                        self.load_available_sessions();
                        self.show_session_list = true;
                        ui.close_menu();
                    }
                    if ui.button("Save Session").clicked() {
                        self.save_needed = true;
                        self.auto_save();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export...").clicked() {
                        self.show_export_dialog = true;
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
                    if ui.button("🎨 Change Theme...").clicked() {
                        self.show_theme_selector = true;
                        ui.close_menu();
                    }
                    ui.separator();
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
                    if ui.button("Toggle AI Panel").clicked() {
                        self.ai_panel.toggle_sidebar();
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    ui.label("Operation Mode:");
                    
                    use crate::config::OperationMode;
                    
                    if ui.selectable_label(
                        self.config.ai.operation_mode == OperationMode::TerminalOnly,
                        "🖥️ Terminal Only"
                    ).on_hover_text("Always execute as shell commands").clicked() {
                        self.config.ai.operation_mode = OperationMode::TerminalOnly;
                        ui.close_menu();
                    }
                    
                    if ui.selectable_label(
                        self.config.ai.operation_mode == OperationMode::AiPromptOnly,
                        "🤖 AI Prompt Only"
                    ).on_hover_text("Always convert to commands using AI").clicked() {
                        self.config.ai.operation_mode = OperationMode::AiPromptOnly;
                        ui.close_menu();
                    }
                    
                    if ui.selectable_label(
                        self.config.ai.operation_mode == OperationMode::Hybrid,
                        "🔀 Hybrid (Auto-detect)"
                    ).on_hover_text("Automatically detect NL vs commands").clicked() {
                        self.config.ai.operation_mode = OperationMode::Hybrid;
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

        // AI Panel (right sidebar)
        if self.ai_panel.is_open() {
            SidePanel::right("ai_panel")
                .default_width(350.0)
                .resizable(true)
                .show(ctx, |ui| {
                    let providers: Vec<String> = self.config.ai.providers
                        .keys()
                        .filter(|k| self.config.ai.providers[*k].enabled)
                        .cloned()
                        .collect();
                    
                    if let Some(action) = self.ai_panel.show_sidebar(ui, &providers) {
                        self.handle_ai_action(action, ctx);
                    }
                });
        }

        // Main terminal area
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("🚀 Immaterium Terminal");
            ui.separator();
            
            // Blocks area (takes remaining space)
            let input_height = 60.0; // Reserve space for command input
            let available_height = ui.available_height() - input_height;
            
            ScrollArea::vertical()
                .id_source("blocks_scroll_area")
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .max_height(available_height)
                .show(ui, |ui| {
                    if self.block_manager.count() == 0 {
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label("✨ Ready to execute commands!");
                            ui.add_space(20.0);
                            ui.label("Type a command below and press Enter");
                            ui.add_space(10.0);
                            ui.label(format!("Working directory: {}", 
                                self.session.working_directory.display()));
                        });
                    } else {
                        // Display all blocks (newest at bottom)
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
                            
                            if block_response.approve_command {
                                // Execute the AI-suggested command
                                let command = block.command.clone();
                                self.block_manager.remove_block(&block.id);
                                self.execute_shell_command(command, ctx);
                            }
                            
                            if block_response.reject_command {
                                // Remove the pending block
                                self.block_manager.remove_block(&block.id);
                            }
                            
                            if block_response.edit_command {
                                // Put command in input for editing
                                self.command_input = block.command.clone();
                                self.block_manager.remove_block(&block.id);
                            }
                            
                            if block_response.regenerate_command {
                                // Regenerate command from original NL input
                                if let Some(nl_input) = block.original_input.clone() {
                                    self.block_manager.remove_block(&block.id);
                                    self.convert_natural_language_to_command(nl_input, ctx);
                                }
                            }
                            
                            ui.add_space(self.config.appearance.block_spacing);
                        }
                    }
                });
            
            ui.separator();
            
            // Command input area at the bottom
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
                ui.label(format!("Session: {}", self.session.name));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                    ui.separator();
                    ui.label(format!("{} blocks", self.block_manager.count()));
                });
            });
        });

        // Session list dialog
        if self.show_session_list {
            egui::Window::new("📂 Open Session")
                .collapsible(false)
                .resizable(true)
                .default_width(500.0)
                .show(ctx, |ui| {
                    ui.label("Select a session to open:");
                    ui.separator();
                    
                    ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for session_info in &self.available_sessions.clone() {
                                ui.horizontal(|ui| {
                                    let is_current = session_info.id == self.session.id;
                                    let label = if is_current {
                                        format!("▶ {} (current)", session_info.name)
                                    } else if session_info.is_active {
                                        format!("● {}", session_info.name)
                                    } else {
                                        session_info.name.clone()
                                    };
                                    
                                    if ui.selectable_label(is_current, label).clicked() && !is_current {
                                        self.switch_to_session(session_info.id);
                                        self.show_session_list = false;
                                    }
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(format!("Updated: {}", 
                                            session_info.updated_at.format("%Y-%m-%d %H:%M")));
                                    });
                                });
                                ui.separator();
                            }
                        });
                    
                    ui.separator();
                    if ui.button("❌ Close").clicked() {
                        self.show_session_list = false;
                    }
                });
        }

        // Generating command indicator (small corner indicator)
        if self.is_generating_command {
            egui::Area::new(egui::Id::new("generating_indicator"))
                .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
                .show(ctx, |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_premultiplied(40, 40, 40, 220))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 165, 0)))
                        .inner_margin(10.0)
                        .rounding(5.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(
                                    egui::RichText::new("🤖 Generating command...")
                                        .color(egui::Color32::from_rgb(255, 165, 0))
                                );
                            });
                        });
                });
        }

        // New session dialog
        if self.show_new_session_dialog {
            egui::Window::new("✨ New Session")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Enter a name for the new session:");
                    
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.new_session_name)
                            .hint_text("Session name...")
                            .desired_width(300.0)
                    );
                    
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.new_session_name.trim().is_empty() {
                            let name = self.new_session_name.trim().to_string();
                            self.create_new_session(name);
                            self.new_session_name.clear();
                            self.show_new_session_dialog = false;
                        }
                    }
                    
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("✅ Create").clicked() {
                            if !self.new_session_name.trim().is_empty() {
                                let name = self.new_session_name.trim().to_string();
                                self.create_new_session(name);
                                self.new_session_name.clear();
                                self.show_new_session_dialog = false;
                            }
                        }
                        
                        if ui.button("❌ Cancel").clicked() {
                            self.new_session_name.clear();
                            self.show_new_session_dialog = false;
                        }
                    });
                });
        }

        // Export dialog
        if self.show_export_dialog {
            egui::Window::new("📤 Export Session")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Export session: {}", self.session.name));
                    ui.separator();
                    
                    ui.label("Choose export format:");
                    ui.add_space(10.0);
                    
                    if ui.button("📄 Export as JSON").clicked() {
                        let filename = format!("{}.json", self.session.name.replace(' ', "_"));
                        let exported = ExportedSession::new(self.session.clone());
                        match exported.to_json_file(&filename) {
                            Ok(_) => tracing::info!("Exported session to {}", filename),
                            Err(e) => tracing::error!("Failed to export: {}", e),
                        }
                        self.show_export_dialog = false;
                    }
                    
                    if ui.button("📝 Export as Markdown").clicked() {
                        let filename = format!("{}.md", self.session.name.replace(' ', "_"));
                        let exported = ExportedSession::new(self.session.clone());
                        match exported.to_markdown_file(&filename) {
                            Ok(_) => tracing::info!("Exported session to {}", filename),
                            Err(e) => tracing::error!("Failed to export: {}", e),
                        }
                        self.show_export_dialog = false;
                    }
                    
                    if ui.button("📋 Export as Text").clicked() {
                        let filename = format!("{}.txt", self.session.name.replace(' ', "_"));
                        let exported = ExportedSession::new(self.session.clone());
                        match exported.to_text_file(&filename) {
                            Ok(_) => tracing::info!("Exported session to {}", filename),
                            Err(e) => tracing::error!("Failed to export: {}", e),
                        }
                        self.show_export_dialog = false;
                    }
                    
                    ui.separator();
                    if ui.button("❌ Cancel").clicked() {
                        self.show_export_dialog = false;
                    }
                });
        }

        // Theme selector dialog
        if self.show_theme_selector {
            egui::Window::new("🎨 Select Theme")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Choose a theme:");
                    ui.separator();
                    
                    let current_theme = self.theme_loader.current().name.clone();
                    let themes = self.theme_loader.available_themes();
                    
                    for theme_name in themes {
                        let is_current = theme_name == current_theme;
                        if ui.selectable_label(is_current, &theme_name).clicked() {
                            if let Err(e) = self.theme_loader.set_theme(&theme_name) {
                                tracing::error!("Failed to switch theme: {}", e);
                            } else {
                                self.theme_loader.apply_to_egui(ctx);
                                tracing::info!("Switched to theme: {}", theme_name);
                            }
                            self.show_theme_selector = false;
                        }
                    }
                    
                    ui.separator();
                    if ui.button("❌ Close").clicked() {
                        self.show_theme_selector = false;
                    }
                });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Save window state
        if let Ok(config_json) = serde_json::to_string(&self.config) {
            storage.set_string("config", config_json);
        }
    }
}
