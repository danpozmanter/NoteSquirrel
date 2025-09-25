use eframe::egui;
use arboard::Clipboard;

use crate::notes_list::NotesList;
use crate::config::Config;

pub struct Editor {
    markdown_text: String,
    clipboard: Option<Clipboard>,
    config: Config,
}

impl Editor {
    pub fn new(config: &Config) -> Self {
        Self {
            markdown_text: String::new(),
            clipboard: Clipboard::new().ok(),
            config: config.clone(),
        }
    }

    pub fn load_notes(&mut self, notes_list: &NotesList) {
        self.markdown_text = notes_list.get_current_content().to_string();
    }

    pub fn get_text(&self) -> &str {
        &self.markdown_text
    }

    pub fn set_text(&mut self, text: &str) {
        self.markdown_text = text.to_string();
    }

    pub fn copy_to_clipboard(&mut self) {
        if let Some(clipboard) = &mut self.clipboard {
            let _ = clipboard.set_text(self.markdown_text.clone());
        }
    }

    pub fn toggle_checkbox_at_line(&mut self, line_index: usize) {
        let lines: Vec<&str> = self.markdown_text.lines().collect();
        if line_index < lines.len() {
            let line = lines[line_index];
            let new_line = if line.contains("- [ ]") {
                line.replace("- [ ]", "- [x]")
            } else if line.contains("- [x]") {
                line.replace("- [x]", "- [ ]")
            } else {
                line.to_string()
            };

            if new_line != line {
                let mut new_lines = lines;
                new_lines[line_index] = &new_line;
                let new_text = new_lines.join("\n");
                self.markdown_text = new_text;
            }
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> bool {
        let inner = ui.available_size();
        let mut changed = false;
        ui.allocate_ui_with_layout(inner, egui::Layout::top_down(egui::Align::LEFT), |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .id_salt("editor_scroll")
                .show(ui, |ui| {
                    changed = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut self.markdown_text)
                            .font(self.config.get_editor_font_id(self.config.editor_font_size))
                            .desired_width(f32::INFINITY),
                    )
                    .changed();
                });
        });
        changed
    }
}