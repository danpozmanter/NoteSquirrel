use eframe::egui;
use egui::{Color32, ScrollArea};
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
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .id_salt("editor_scroll")
                .show(ui, |ui| {
                    changed = self.render_syntax_highlighted_editor(ui);
                });
        });

        changed
    }

    fn render_syntax_highlighted_editor(&mut self, ui: &mut egui::Ui) -> bool {
        use egui::TextEdit;

        let font_id = self.config.get_editor_font_id(self.config.editor_font_size);
        let editor_font_size = self.config.editor_font_size;

        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            let mut job = egui::text::LayoutJob::default();

            for line in string.lines() {
                Self::highlight_markdown_line_static(line, &mut job, font_id.clone(), editor_font_size);
                job.append("\n", 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: Color32::from_rgb(200, 200, 200),
                    ..Default::default()
                });
            }

            ui.fonts(|f| f.layout_job(job))
        };

        ui.add_sized(
            ui.available_size(),
            TextEdit::multiline(&mut self.markdown_text)
                .font(font_id.clone())
                .desired_width(f32::INFINITY)
                .layouter(&mut layouter),
        ).changed()
    }

    fn highlight_markdown_line_static(line: &str, job: &mut egui::text::LayoutJob, font_id: egui::FontId, font_size: f32) {
        let trimmed = line.trim_start();

        // Handle headers
        if trimmed.starts_with("######") {
            Self::add_header_text_static(line, 6, Color32::from_rgb(255, 255, 180), 14.0, job, font_id.clone(), font_size);
        } else if trimmed.starts_with("#####") {
            Self::add_header_text_static(line, 5, Color32::from_rgb(220, 180, 255), 16.0, job, font_id.clone(), font_size);
        } else if trimmed.starts_with("####") {
            Self::add_header_text_static(line, 4, Color32::from_rgb(255, 180, 220), 18.0, job, font_id.clone(), font_size);
        } else if trimmed.starts_with("###") {
            Self::add_header_text_static(line, 3, Color32::from_rgb(180, 220, 255), 20.0, job, font_id.clone(), font_size);
        } else if trimmed.starts_with("##") {
            Self::add_header_text_static(line, 2, Color32::from_rgb(220, 255, 180), 24.0, job, font_id.clone(), font_size);
        } else if trimmed.starts_with("#") {
            Self::add_header_text_static(line, 1, Color32::from_rgb(255, 220, 100), 28.0, job, font_id.clone(), font_size);
        }
        // Handle code blocks
        else if trimmed.starts_with("```") {
            job.append(line, 0.0, egui::TextFormat {
                font_id: egui::FontId::monospace(font_size),
                color: Color32::from_rgb(150, 120, 200),
                background: Color32::from_rgb(40, 40, 50),
                ..Default::default()
            });
        }
        // Handle quotes
        else if trimmed.starts_with(">") {
            job.append(line, 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: Color32::from_rgb(160, 160, 160),
                italics: true,
                ..Default::default()
            });
        }
        // Handle lists - simple version
        else if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            job.append(line, 0.0, egui::TextFormat {
                font_id,
                color: Color32::from_rgb(60, 120, 200),
                ..Default::default()
            });
        }
        // Handle numbered lists
        else if trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) && trimmed.contains(". ") {
            job.append(line, 0.0, egui::TextFormat {
                font_id,
                color: Color32::from_rgb(60, 120, 200),
                ..Default::default()
            });
        }
        // Handle regular text
        else {
            job.append(line, 0.0, egui::TextFormat {
                font_id,
                color: Color32::from_rgb(200, 200, 200),
                ..Default::default()
            });
        }
    }

    fn add_header_text_static(line: &str, level: usize, color: Color32, _size: f32, job: &mut egui::text::LayoutJob, font_id: egui::FontId, _font_size: f32) {
        let prefix = "#".repeat(level);
        let prefix_with_space = format!("{} ", prefix);

        if let Some(content_start) = line.find(&prefix_with_space) {
            // Add text before header
            if content_start > 0 {
                job.append(&line[..content_start], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: Color32::from_rgb(200, 200, 200),
                    ..Default::default()
                });
            }

            // Add header prefix with distinct color
            job.append(&prefix, 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: Color32::from_rgb(100, 100, 100),
                ..Default::default()
            });

            // Add space
            job.append(" ", 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: Color32::from_rgb(200, 200, 200),
                ..Default::default()
            });

            // Add header content
            let content = &line[content_start + prefix_with_space.len()..];
            job.append(content, 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color,
                ..Default::default()
            });
        } else {
            job.append(line, 0.0, egui::TextFormat {
                font_id,
                color: Color32::from_rgb(200, 200, 200),
                ..Default::default()
            });
        }
    }

}