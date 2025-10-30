use crate::core::{Block, BlockState};
use egui::{Color32, RichText, Ui};

pub struct BlockWidget<'a> {
    block: &'a Block,
    font_size: f32,
}

impl<'a> BlockWidget<'a> {
    pub fn new(block: &'a Block, font_size: f32) -> Self {
        Self { block, font_size }
    }

    pub fn show(self, ui: &mut Ui) -> BlockResponse {
        let mut response = BlockResponse::default();

        let block_color = match self.block.state {
            BlockState::PendingApproval => Color32::from_rgb(255, 165, 0), // Orange - waiting for approval
            BlockState::Running => Color32::from_rgb(100, 149, 237), // Cornflower blue
            BlockState::Completed => Color32::from_rgb(72, 209, 204), // Medium turquoise
            BlockState::Failed => Color32::from_rgb(220, 20, 60), // Crimson
            BlockState::Editing => Color32::from_rgb(169, 169, 169), // Dark gray
            BlockState::Cancelled => Color32::from_rgb(255, 165, 0), // Orange
        };

        let bg_color = if self.block.is_selected {
            Color32::from_rgba_premultiplied(100, 100, 150, 30)
        } else {
            Color32::from_rgba_premultiplied(40, 40, 40, 20)
        };

        let frame_response = egui::Frame::none()
            .fill(bg_color)
            .stroke(egui::Stroke::new(1.0, block_color))
            .inner_margin(8.0)
            .show(ui, |ui| {
                // Header with command and metadata
                ui.horizontal(|ui| {
                    // Collapse/expand button
                    let collapse_icon = if self.block.is_collapsed { "▶" } else { "▼" };
                    if ui.button(collapse_icon).clicked() {
                        response.toggle_collapsed = true;
                    }

                    // Status indicator
                    let status_icon = match self.block.state {
                        BlockState::PendingApproval => "🤖",
                        BlockState::Running => "⏳",
                        BlockState::Completed => "✅",
                        BlockState::Failed => "❌",
                        BlockState::Editing => "✏️",
                        BlockState::Cancelled => "🚫",
                    };
                    ui.label(status_icon);

                    // Command
                    ui.label(
                        RichText::new(format!("$ {}", self.block.get_display_command()))
                            .font(egui::FontId::monospace(self.font_size))
                            .color(Color32::from_rgb(200, 200, 200)),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Context menu button
                        if ui.button("⋮").clicked() {
                            response.show_context_menu = true;
                        }

                        // Duration
                        if !self.block.format_duration().is_empty() {
                            ui.label(
                                RichText::new(self.block.format_duration())
                                    .color(Color32::from_rgb(150, 150, 150))
                                    .size(self.font_size - 2.0),
                            );
                        }

                        // Exit code
                        if let Some(code) = self.block.exit_code {
                            let code_color = if code == 0 {
                                Color32::from_rgb(72, 209, 204)
                            } else {
                                Color32::from_rgb(220, 20, 60)
                            };
                            ui.label(
                                RichText::new(format!("[{}]", code))
                                    .color(code_color)
                                    .size(self.font_size - 2.0),
                            );
                        }
                    });
                });

                // For PendingApproval blocks, show the original NL input and approval buttons
                if self.block.state == BlockState::PendingApproval {
                    if let Some(ref nl_input) = self.block.original_input {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("💭")
                                    .color(Color32::from_rgb(150, 150, 150)),
                            );
                            ui.label(
                                RichText::new(nl_input)
                                    .italics()
                                    .color(Color32::from_rgb(180, 180, 180))
                                    .size(self.font_size - 1.0),
                            );
                        });
                    }
                    
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("AI suggests:")
                                .color(Color32::from_rgb(255, 165, 0))
                                .strong(),
                        );
                        
                        let execute_btn = ui.button(
                            RichText::new("✅ Execute (Enter)")
                                .color(Color32::from_rgb(72, 209, 204))
                        );
                        
                        // Check for Enter key
                        if execute_btn.clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            response.approve_command = true;
                        }
                        
                        if ui.button(
                            RichText::new("✏️ Edit (E)")
                                .color(Color32::from_rgb(100, 149, 237))
                        ).clicked() || ui.input(|i| i.key_pressed(egui::Key::E)) {
                            response.edit_command = true;
                        }
                        
                        if ui.button(
                            RichText::new("🔄 Regenerate (R)")
                                .color(Color32::from_rgb(255, 215, 0))
                        ).clicked() || ui.input(|i| i.key_pressed(egui::Key::R)) {
                            response.regenerate_command = true;
                        }
                        
                        if ui.button(
                            RichText::new("❌ Cancel (Esc)")
                                .color(Color32::from_rgb(220, 20, 60))
                        ).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            response.reject_command = true;
                        }
                        
                        ui.label(
                            RichText::new("Press Enter to execute")
                                .italics()
                                .color(Color32::from_rgb(150, 150, 150))
                                .size(self.font_size - 2.0)
                        );
                    });
                }

                // Output (if not collapsed)
                if !self.block.is_collapsed && !self.block.output.is_empty() {
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .id_source(format!("block_output_{}", self.block.id))
                        .max_height(400.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&self.block.output)
                                        .font(egui::FontId::monospace(self.font_size))
                                        .color(Color32::from_rgb(220, 220, 220)),
                                )
                            );
                        });
                }

                // Metadata footer (only if expanded and completed)
                if !self.block.is_collapsed && self.block.is_completed() {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "📁 {}",
                                self.block.metadata.working_directory.display()
                            ))
                            .size(self.font_size - 2.0)
                            .color(Color32::from_rgb(150, 150, 150)),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(self.block.timestamp.format("%H:%M:%S").to_string())
                                    .size(self.font_size - 2.0)
                                    .color(Color32::from_rgb(150, 150, 150)),
                            );
                        });
                    });
                }
            });

        // Handle click for selection on the entire frame
        let interact_response = ui.interact(
            frame_response.response.rect,
            ui.id().with(&self.block.id),
            egui::Sense::click(),
        );

        if interact_response.clicked() {
            response.selected = true;
        }

        if interact_response.secondary_clicked() {
            response.show_context_menu = true;
        }

        response
    }
}

#[derive(Default)]
pub struct BlockResponse {
    pub selected: bool,
    pub toggle_collapsed: bool,
    pub show_context_menu: bool,
    pub approve_command: bool,
    pub reject_command: bool,
    pub edit_command: bool,
    pub regenerate_command: bool,
}
