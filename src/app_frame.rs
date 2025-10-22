use eframe::egui;

use crate::notes_list::NotesList;
use crate::editor::Editor;
use crate::rendered_view::RenderedView;
use crate::config::{Config, ConfigLoadResult};
use crate::find_replace::{FindReplace, FindReplaceAction};

#[allow(dead_code)]
pub struct AppFrame {
    pub notes_list: NotesList,
    pub editor: Editor,
    pub rendered_view: RenderedView,
    pub show_delete_confirmation: bool,
    pub config: Config,
    pub error_dialog_errors: Vec<String>,
    pub show_error_dialog: bool,
    pub find_replace: FindReplace,
    pub show_unsaved_dialog: bool,
    pub pending_close: bool,
    pub force_close: bool,
}

impl AppFrame {
    pub fn new() -> Self {
        let ConfigLoadResult { config, errors } = Config::load();
        let mut app_frame = Self {
            notes_list: NotesList::new(&config),
            editor: Editor::new(&config),
            rendered_view: RenderedView::new(&config),
            show_delete_confirmation: false,
            config,
            error_dialog_errors: errors,
            show_error_dialog: false,
            find_replace: FindReplace::new(),
            show_unsaved_dialog: false,
            pending_close: false,
            force_close: false,
        };

        app_frame.load_notes();
        app_frame
    }

    pub fn setup_fonts_and_collect_errors(&mut self, ctx: &egui::Context) {
        let (loaded_fonts, font_errors) = self.config.setup_fonts(ctx);
        self.config.loaded_fonts = loaded_fonts;
        self.error_dialog_errors.extend(font_errors);
        if !self.error_dialog_errors.is_empty() {
            self.show_error_dialog = true;
        }
    }

    pub fn load_notes(&mut self) {
        self.notes_list.load_notes();
        self.editor.load_notes(&self.notes_list);
    }

    pub fn update_window_title(&self, ctx: &egui::Context) {
        let note_name = self.notes_list.get_current_note_name();
        let is_dirty = self.notes_list.is_current_note_dirty();
        let dirty_indicator = if is_dirty { "*" } else { "" };
        let title = format!("Note Squirrel - {}{}", note_name, dirty_indicator);

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    #[allow(dead_code)]
    pub fn save_config(&self) {
        if let Err(e) = self.config.save() {
            eprintln!("Failed to save config: {}", e);
        }
    }

    pub fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::S)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::S)
            {
                self.save_current_note();
            }

            if i.consume_key(egui::Modifiers::CTRL, egui::Key::N)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::N)
            {
                self.create_new_note();
            }

            if (i.consume_key(egui::Modifiers::CTRL, egui::Key::C)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::C))
                && !i.focused
            {
                self.editor.copy_to_clipboard();
            }

            if i.consume_key(egui::Modifiers::CTRL, egui::Key::D)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::D)
            {
                self.show_delete_confirmation = true;
            }

            if i.consume_key(egui::Modifiers::CTRL, egui::Key::F)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::F)
            {
                self.find_replace.toggle_dialog();
            }

            if (i.consume_key(egui::Modifiers::CTRL, egui::Key::Z)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::Z))
                && self.editor.undo()
            {
                self.notes_list.save_current_content(self.editor.get_text());
            }

            if (i.consume_key(egui::Modifiers::CTRL, egui::Key::Y)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::Y))
                && self.editor.redo()
            {
                self.notes_list.save_current_content(self.editor.get_text());
            }

            if i.consume_key(egui::Modifiers::NONE, egui::Key::F3)
                && self.find_replace.show_dialog
            {
                self.find_replace.next_match();
            }

            if i.consume_key(egui::Modifiers::SHIFT, egui::Key::F3)
                && self.find_replace.show_dialog
            {
                self.find_replace.previous_match();
            }

            if (i.consume_key(egui::Modifiers::CTRL, egui::Key::Comma)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::Comma))
                && self.editor.insert_list_entry(None)
            {
                self.notes_list.save_current_content(self.editor.get_text());
            }

            if (i.consume_key(egui::Modifiers::CTRL, egui::Key::Period)
                || i.consume_key(egui::Modifiers::MAC_CMD, egui::Key::Period))
                && self.editor.insert_checkbox_entry(None)
            {
                self.notes_list.save_current_content(self.editor.get_text());
            }
        });
    }

    pub fn render_delete_confirmation_dialog(&mut self, ctx: &egui::Context) {
        if self.show_delete_confirmation {
            egui::Window::new("Delete Note")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Are you sure you want to delete '{}'?",
                        self.notes_list.get_current_note_name()
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() || ui.input(|i| i.key_pressed(egui::Key::Y)) {
                            self.delete_current_note();
                            self.show_delete_confirmation = false;
                        }
                        if ui.button("No").clicked() || ui.input(|i| i.key_pressed(egui::Key::N)) {
                            self.show_delete_confirmation = false;
                        }
                    });
                });
        }
    }

    pub fn render_error_dialog(&mut self, ctx: &egui::Context) {
        if self.show_error_dialog {
            egui::Window::new("Configuration Errors")
                .collapsible(false)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("The following errors occurred while loading the configuration:");
                    ui.separator();

                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for error in &self.error_dialog_errors {
                                ui.label(format!("• {}", error));
                            }
                        });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.show_error_dialog = false;
                            self.error_dialog_errors.clear();
                        }
                    });
                });
        }
    }

    pub fn handle_find_replace(&mut self, ctx: &egui::Context) {
        let action = self.find_replace.render(ctx);

        match action {
            FindReplaceAction::UpdateMatches => {
                self.find_replace.update_matches(self.editor.get_text());
                self.update_editor_matches();
            }
            FindReplaceAction::NextMatch => {
                self.find_replace.next_match();
                self.update_editor_matches();
            }
            FindReplaceAction::PreviousMatch => {
                self.find_replace.previous_match();
                self.update_editor_matches();
            }
            FindReplaceAction::ReplaceCurrent => {
                let mut text = self.editor.get_text().to_string();
                if self.find_replace.replace_current(&mut text) {
                    self.editor.set_text_with_undo(&text);
                    self.notes_list.save_current_content(&text);
                    self.find_replace.update_matches(&text);
                    self.update_editor_matches();
                }
            }
            FindReplaceAction::ReplaceAll => {
                let mut text = self.editor.get_text().to_string();
                let count = self.find_replace.replace_all(&mut text);
                if count > 0 {
                    self.editor.set_text_with_undo(&text);
                    self.notes_list.save_current_content(&text);
                    self.find_replace.update_matches(&text);
                    self.update_editor_matches();
                }
            }
            FindReplaceAction::None => {}
        }

        // Update matches if dialog is shown
        if self.find_replace.show_dialog {
            self.update_editor_matches();
        } else {
            self.editor.clear_matches();
        }
    }

    fn update_editor_matches(&mut self) {
        let ranges = self.find_replace.get_match_ranges();
        let current = self.find_replace.current_match_index;
        self.editor.set_match_ranges(ranges, current);
    }

    pub fn render_main_layout(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar_panel")
            .exact_width(200.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(self.notes_list.get_search_text_mut());
                });
                ui.separator();

                let inner = ui.available_size();
                ui.allocate_ui_with_layout(inner, egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .id_salt("notes_list_scroll")
                        .show(ui, |ui| {
                            if let Some(switch_to_index) = self.notes_list.render(ui) {
                                self.switch_to_note(switch_to_index);
                            }
                        });
                });
            });

        self.render_editor_and_preview(ctx);
    }

    fn render_editor_and_preview(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    let inner = ui.available_size();
                    ui.allocate_ui_with_layout(inner, egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        if self.editor.render(ui) {
                            self.notes_list.save_current_content(self.editor.get_text());
                        }
                    });
                });

                columns[1].vertical(|ui| {
                    let inner = ui.available_size();
                    ui.allocate_ui_with_layout(inner, egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        if let Some(checkbox_toggles) = self.rendered_view.render(ui, self.editor.get_text())
                            && !checkbox_toggles.is_empty() {
                                for line in checkbox_toggles {
                                    self.editor.toggle_checkbox_at_line(line);
                                }
                                self.notes_list.save_current_content(self.editor.get_text());
                            }
                    });
                });
            });
        });
    }

    fn save_current_note(&mut self) {
        let note_name = self.notes_list.get_current_note_name().to_string();
        if self.notes_list.save_current_note(&note_name, self.editor.get_text()) {
            self.notes_list.mark_current_clean();
        }
    }

    fn create_new_note(&mut self) {
        if let Some(_new_note_name) = self.notes_list.create_new_note() {
            self.editor.set_text("");
        }
    }

    fn delete_current_note(&mut self) {
        if self.notes_list.delete_current_note() {
            self.editor.set_text(self.notes_list.get_current_content());
        }
    }

    fn switch_to_note(&mut self, index: usize) {
        self.notes_list.save_current_content(self.editor.get_text());
        if self.notes_list.switch_to_note(index) {
            self.editor.set_text(self.notes_list.get_current_content());
        }
    }

    pub fn render_unsaved_changes_dialog(&mut self, ctx: &egui::Context) {
        if self.show_unsaved_dialog {
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("You have unsaved information");
                    ui.separator();

                    ui.horizontal(|ui| {
                        let mut exit_text = egui::text::LayoutJob::default();
                        exit_text.append("E", 0.0, egui::TextFormat {
                            underline: egui::Stroke::new(1.0, ui.style().visuals.text_color()),
                            ..Default::default()
                        });
                        exit_text.append("xit", 0.0, egui::TextFormat::default());

                        if ui.button(exit_text).clicked() || ui.input(|i| i.key_pressed(egui::Key::E) && i.modifiers.alt) {
                            self.show_unsaved_dialog = false;
                            self.force_close = true;
                        }

                        let mut cancel_text = egui::text::LayoutJob::default();
                        cancel_text.append("C", 0.0, egui::TextFormat {
                            underline: egui::Stroke::new(1.0, ui.style().visuals.text_color()),
                            ..Default::default()
                        });
                        cancel_text.append("ancel", 0.0, egui::TextFormat::default());

                        if ui.button(cancel_text).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::C) && i.modifiers.alt)
                            || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.show_unsaved_dialog = false;
                            self.pending_close = false;
                        }

                        let mut save_text = egui::text::LayoutJob::default();
                        save_text.append("S", 0.0, egui::TextFormat {
                            underline: egui::Stroke::new(1.0, ui.style().visuals.text_color()),
                            ..Default::default()
                        });
                        save_text.append("ave All", 0.0, egui::TextFormat::default());

                        if ui.button(save_text).clicked() || ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.alt) {
                            self.notes_list.save_all_notes();
                            self.show_unsaved_dialog = false;
                            if self.pending_close {
                                self.force_close = true;
                            }
                        }
                    });
                });
        }
    }
}

impl Default for AppFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for AppFrame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.force_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        if ctx.input(|i| i.viewport().close_requested())
            && !self.show_unsaved_dialog
            && self.notes_list.has_any_dirty_notes()
        {
            self.show_unsaved_dialog = true;
            self.pending_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        self.update_window_title(ctx);
        self.handle_global_shortcuts(ctx);
        self.render_delete_confirmation_dialog(ctx);
        self.render_error_dialog(ctx);
        self.render_unsaved_changes_dialog(ctx);
        self.handle_find_replace(ctx);
        self.render_main_layout(ctx);
    }
}