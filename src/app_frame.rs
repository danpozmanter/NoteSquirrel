use eframe::egui;

use crate::notes_list::NotesList;
use crate::editor::Editor;
use crate::rendered_view::RenderedView;
use crate::config::{Config, ConfigLoadResult};

#[allow(dead_code)]
pub struct AppFrame {
    pub notes_list: NotesList,
    pub editor: Editor,
    pub rendered_view: RenderedView,
    pub show_delete_confirmation: bool,
    pub config: Config,
    pub error_dialog_errors: Vec<String>,
    pub show_error_dialog: bool,
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
}

impl Default for AppFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for AppFrame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_window_title(ctx);
        self.handle_global_shortcuts(ctx);
        self.render_delete_confirmation_dialog(ctx);
        self.render_error_dialog(ctx);
        self.render_main_layout(ctx);
    }
}