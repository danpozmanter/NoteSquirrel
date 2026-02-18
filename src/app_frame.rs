use eframe::egui;

use crate::notes_list::{NotesList, SortOrder};
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
    last_window_title: String,
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
            last_window_title: String::new(),
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
        if let Some(ref name) = self.config.last_open_note
            && let Some(index) = self.notes_list.find_note_index(name) {
                self.notes_list.switch_to_note(index);
            }
        self.editor.load_notes(&self.notes_list);
    }

    pub fn update_window_title(&mut self, ctx: &egui::Context) {
        let note_name = self.notes_list.get_current_note_name();
        let title = format!("Note Squirrel - {}", note_name);

        if title != self.last_window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_window_title = title;
        }
    }

    pub fn save_config(&self) {
        if let Err(e) = self.config.save() {
            eprintln!("Failed to save config: {}", e);
        }
    }

    pub fn handle_global_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input_mut(|i| {
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
                    let is_alpha = self.notes_list.get_sort_order() == &SortOrder::Alphabetical;
                    let is_recent = self.notes_list.get_sort_order() == &SortOrder::LastModified;
                    if ui.selectable_label(is_alpha, "A-Z").clicked() {
                        self.notes_list.set_sort_order(SortOrder::Alphabetical);
                    }
                    if ui.selectable_label(is_recent, "Recent").clicked() {
                        self.notes_list.set_sort_order(SortOrder::LastModified);
                    }
                });
                ui.horizontal(|ui| {
                    let icon_size = egui::vec2(16.0, 16.0);
                    let (rect, _) = ui.allocate_exact_size(icon_size, egui::Sense::hover());
                    if ui.is_rect_visible(rect) {
                        let painter = ui.painter();
                        let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(170, 170, 170));
                        let center = rect.center() - egui::vec2(1.5, 1.5);
                        painter.circle_stroke(center, 4.5, stroke);
                        let h0 = center + egui::vec2(3.2, 3.2);
                        painter.line_segment([h0, h0 + egui::vec2(3.0, 3.0)], stroke);
                    }
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
            self.config.last_open_note = Some(self.notes_list.get_current_note_name().to_string());
            self.save_config();
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
        if ctx.input(|i| i.viewport().close_requested()) {
            self.config.last_open_note = Some(self.notes_list.get_current_note_name().to_string());
            self.save_config();
        }

        self.update_window_title(ctx);
        self.handle_global_shortcuts(ctx);
        self.render_delete_confirmation_dialog(ctx);
        self.render_error_dialog(ctx);
        self.handle_find_replace(ctx);
        self.render_main_layout(ctx);
    }
}