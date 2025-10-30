use crate::ai::{build_minimal_context, AiEngine, ChatRequest, ContextConfig, LlmProvider};
use crate::core::Block;
use egui::{ScrollArea, TextEdit, Ui};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum AiPanelMode {
    Closed,
    Sidebar,
    FullPanel,
}

pub struct AiPanel {
    mode: AiPanelMode,
    prompt: String,
    response: String,
    is_streaming: bool,
    selected_provider: String,
    selected_model: String,
    available_models: Vec<String>,
    pub include_context: bool,
    pub context_blocks: usize,
    // Conversation history
    conversation: Vec<ConversationMessage>,
}

#[derive(Debug, Clone)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl Default for AiPanel {
    fn default() -> Self {
        Self {
            mode: AiPanelMode::Closed,
            prompt: String::new(),
            response: String::new(),
            is_streaming: false,
            selected_provider: "ollama".to_string(),
            selected_model: String::new(),
            available_models: Vec::new(),
            include_context: true,
            context_blocks: 5,
            conversation: Vec::new(),
        }
    }
}

impl AiPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mode(&self) -> &AiPanelMode {
        &self.mode
    }

    pub fn set_mode(&mut self, mode: AiPanelMode) {
        self.mode = mode;
    }

    pub fn toggle_sidebar(&mut self) {
        self.mode = match self.mode {
            AiPanelMode::Closed => AiPanelMode::Sidebar,
            AiPanelMode::Sidebar => AiPanelMode::Closed,
            AiPanelMode::FullPanel => AiPanelMode::Sidebar,
        };
    }

    pub fn is_open(&self) -> bool {
        self.mode != AiPanelMode::Closed
    }

    pub fn set_selected_provider(&mut self, provider: String) {
        self.selected_provider = provider;
        self.available_models.clear();
    }

    pub fn set_available_models(&mut self, models: Vec<String>) {
        self.available_models = models;
        if !self.available_models.is_empty() && self.selected_model.is_empty() {
            self.selected_model = self.available_models[0].clone();
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        self.conversation.push(ConversationMessage {
            role: MessageRole::User,
            content,
            timestamp: chrono::Utc::now(),
        });
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.conversation.push(ConversationMessage {
            role: MessageRole::Assistant,
            content,
            timestamp: chrono::Utc::now(),
        });
    }

    pub fn clear_conversation(&mut self) {
        self.conversation.clear();
        self.response.clear();
    }

    /// Draw the AI panel in sidebar mode
    pub fn show_sidebar(&mut self, ui: &mut Ui, providers: &[String]) -> Option<AiAction> {
        let mut action = None;

        ui.heading("🤖 AI Assistant");
        ui.separator();

        // Provider selection
        ui.horizontal(|ui| {
            ui.label("Provider:");
            egui::ComboBox::from_id_source("ai_provider")
                .selected_text(&self.selected_provider)
                .show_ui(ui, |ui| {
                    for provider in providers {
                        if ui
                            .selectable_label(provider == &self.selected_provider, provider)
                            .clicked()
                        {
                            action = Some(AiAction::ProviderChanged(provider.clone()));
                        }
                    }
                });
        });

        // Model selection
        ui.horizontal(|ui| {
            ui.label("Model:");
            if self.available_models.is_empty() {
                if ui.button("Load Models").clicked() {
                    action = Some(AiAction::LoadModels);
                }
            } else {
                egui::ComboBox::from_id_source("ai_model")
                    .selected_text(&self.selected_model)
                    .show_ui(ui, |ui| {
                        for model in &self.available_models {
                            ui.selectable_value(&mut self.selected_model, model.clone(), model);
                        }
                    });
            }
        });

        ui.separator();

        // Context options
        ui.checkbox(&mut self.include_context, "Include command history");
        if self.include_context {
            ui.horizontal(|ui| {
                ui.label("Recent blocks:");
                ui.add(egui::Slider::new(&mut self.context_blocks, 1..=20));
            });
        }

        ui.separator();

        // Conversation history
        ui.label("Conversation:");
        ScrollArea::vertical()
            .id_source("conversation_scroll")
            .max_height(200.0)
            .show(ui, |ui| {
                for msg in &self.conversation {
                    let (icon, color) = match msg.role {
                        MessageRole::User => ("👤", egui::Color32::from_rgb(100, 149, 237)),
                        MessageRole::Assistant => ("🤖", egui::Color32::from_rgb(76, 175, 80)),
                        MessageRole::System => ("ℹ️", egui::Color32::from_rgb(158, 158, 158)),
                    };

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(icon).color(color));
                        ui.label(
                            egui::RichText::new(&msg.content)
                                .color(color)
                                .text_style(egui::TextStyle::Small),
                        );
                    });
                    ui.separator();
                }
            });

        ui.separator();

        // Prompt input
        ui.label("Ask AI:");
        let response = ui.add(
            TextEdit::multiline(&mut self.prompt)
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .hint_text("Type your question..."),
        );

        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if !self.prompt.trim().is_empty() {
                action = Some(AiAction::SendPrompt(self.prompt.clone()));
                self.prompt.clear();
            }
        }

        ui.horizontal(|ui| {
            if ui
                .button("Send")
                .on_hover_text("Send prompt to AI (Enter)")
                .clicked()
                && !self.prompt.trim().is_empty()
            {
                action = Some(AiAction::SendPrompt(self.prompt.clone()));
                self.prompt.clear();
            }

            if ui.button("Clear").clicked() {
                self.clear_conversation();
            }
        });

        // Response area
        if !self.response.is_empty() {
            ui.separator();
            ui.label("Response:");
            ScrollArea::vertical()
                .id_source("response_scroll")
                .max_height(150.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.label(&self.response);
                });
        }

        if self.is_streaming {
            ui.spinner();
            ui.label("Receiving response...");
        }

        action
    }

    pub fn set_response(&mut self, response: String) {
        self.response = response;
        self.is_streaming = false;
    }

    pub fn append_response(&mut self, chunk: String) {
        self.response.push_str(&chunk);
    }

    pub fn start_streaming(&mut self) {
        self.is_streaming = true;
        self.response.clear();
    }

    pub fn stop_streaming(&mut self) {
        self.is_streaming = false;
    }
}

#[derive(Debug, Clone)]
pub enum AiAction {
    ProviderChanged(String),
    LoadModels,
    SendPrompt(String),
}
