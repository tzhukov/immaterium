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

        // Subtle left border color based on state
        let block_color = match self.block.state {
            BlockState::PendingApproval => Color32::from_rgb(255, 165, 0), // Orange
            BlockState::Running => Color32::from_rgb(100, 149, 237), // Blue
            BlockState::Completed => Color32::from_rgb(80, 200, 120), // Green
            BlockState::Failed => Color32::from_rgb(220, 60, 80), // Red
            BlockState::Editing => Color32::from_rgb(150, 150, 150), // Gray
            BlockState::Cancelled => Color32::from_rgb(180, 140, 60), // Muted orange
        };

        let bg_color = if self.block.is_selected {
            Color32::from_rgba_premultiplied(50, 50, 70, 15)
        } else {
            Color32::from_rgba_premultiplied(0, 0, 0, 0) // Transparent
        };

        let frame_response = egui::Frame::none()
            .fill(bg_color)
            .stroke(egui::Stroke::NONE) // No border, just left accent
            .inner_margin(egui::Margin {
                left: 0.0,
                right: 8.0,
                top: 6.0,
                bottom: 6.0,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Left accent bar (Warp-style)
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(3.0, ui.available_height()),
                        egui::Sense::hover()
                    );
                    ui.painter().rect_filled(rect, 0.0, block_color);
                    ui.add_space(8.0);
                    
                    ui.vertical(|ui| {
                        // Header with command and metadata
                        ui.horizontal(|ui| {
                            // Collapse/expand button (subtle)
                            let collapse_icon = if self.block.is_collapsed { "›" } else { "⌄" };
                            if ui.small_button(collapse_icon).clicked() {
                                response.toggle_collapsed = true;
                            }

                            // Command (no $ prefix for cleaner look)
                            ui.label(
                                RichText::new(self.block.get_display_command())
                                    .font(egui::FontId::monospace(self.font_size))
                                    .color(Color32::from_rgb(220, 220, 220)),
                            );

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Context menu button (subtle)
                                if ui.small_button("⋯").clicked() {
                                    response.show_context_menu = true;
                                }

                                // Duration (more subtle)
                                if !self.block.format_duration().is_empty() {
                                    ui.label(
                                        RichText::new(self.block.format_duration())
                                            .color(Color32::from_rgb(120, 120, 120))
                                            .size(self.font_size - 3.0),
                                    );
                                }

                                // Exit code (only show if non-zero)
                                if let Some(code) = self.block.exit_code {
                                    if code != 0 {
                                        ui.label(
                                            RichText::new(format!("exit {}", code))
                                                .color(Color32::from_rgb(220, 60, 80))
                                                .size(self.font_size - 2.0),
                                        );
                                    }
                                }
                            });
                        });

                        // For PendingApproval blocks, show the original NL input and approval buttons
                        if self.block.state == BlockState::PendingApproval {
                            if let Some(ref nl_input) = self.block.original_input {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(format!("💭 {}", nl_input))
                                        .italics()
                                        .color(Color32::from_rgb(140, 140, 140))
                                        .size(self.font_size - 1.0),
                                );
                            }
                            
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.small_button(RichText::new("✓ Execute (Enter)").color(Color32::from_rgb(80, 200, 120)))
                                    .clicked().then(|| response.approve_command = true);
                                
                                if ui.small_button("✎ Edit").clicked() || ui.input(|i| i.key_pressed(egui::Key::E)) {
                                    response.edit_command = true;
                                }
                                
                                if ui.small_button("↻ Regenerate").clicked() || ui.input(|i| i.key_pressed(egui::Key::R)) {
                                    response.regenerate_command = true;
                                }
                                
                                if ui.small_button("✕ Cancel").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                    response.reject_command = true;
                                }
                            });
                        }

                        // Output (if not collapsed)
                        if !self.block.is_collapsed && !self.block.output.is_empty() {
                            ui.add_space(4.0);
                            
                            egui::ScrollArea::vertical()
                                .id_source(format!("block_output_{}", self.block.id))
                                .max_height(400.0)
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(&self.block.output)
                                                .font(egui::FontId::monospace(self.font_size))
                                                .color(Color32::from_rgb(200, 200, 200)),
                                        )
                                    );
                                });
                        }

                        // Metadata footer (only if expanded and completed)
                        if !self.block.is_collapsed && self.block.is_completed() {
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "📁 {}",
                                        self.block.metadata.working_directory.display()
                                    ))
                                    .size(self.font_size - 3.0)
                                    .color(Color32::from_rgb(110, 110, 110)),
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(self.block.timestamp.format("%H:%M:%S").to_string())
                                            .size(self.font_size - 3.0)
                                            .color(Color32::from_rgb(110, 110, 110)),
                                    );
                                });
                            });
                        }
                    });
                });
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
