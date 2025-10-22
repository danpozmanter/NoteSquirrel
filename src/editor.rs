use eframe::egui;
use egui::{Color32, ScrollArea};
use arboard::Clipboard;

use crate::notes_list::NotesList;
use crate::config::Config;

pub struct Editor {
    markdown_text: String,
    clipboard: Option<Clipboard>,
    config: Config,
    should_focus: bool,
    match_ranges: Vec<(usize, usize)>,
    current_match: Option<usize>,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    cursor_override: Option<egui::text::CCursorRange>,
    current_cursor_pos: Option<usize>,
    text_edit_id: Option<egui::Id>,
}

impl Editor {
    pub fn new(config: &Config) -> Self {
        Self {
            markdown_text: String::new(),
            clipboard: Clipboard::new().ok(),
            config: config.clone(),
            should_focus: true,
            match_ranges: Vec::new(),
            current_match: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            cursor_override: None,
            current_cursor_pos: None,
            text_edit_id: None,
        }
    }

    pub fn load_notes(&mut self, notes_list: &NotesList) {
        self.markdown_text = notes_list.get_current_content().to_string();
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn get_text(&self) -> &str {
        &self.markdown_text
    }

    pub fn set_text(&mut self, text: &str) {
        self.markdown_text = text.to_string();
    }

    pub fn set_text_with_undo(&mut self, text: &str) {
        if self.markdown_text != text {
            self.undo_stack.push(self.markdown_text.clone());
            self.redo_stack.clear();
            self.markdown_text = text.to_string();
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(previous_state) = self.undo_stack.pop() {
            self.redo_stack.push(self.markdown_text.clone());
            self.markdown_text = previous_state;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next_state) = self.redo_stack.pop() {
            self.undo_stack.push(self.markdown_text.clone());
            self.markdown_text = next_state;
            true
        } else {
            false
        }
    }

    pub fn copy_to_clipboard(&mut self) {
        if let Some(clipboard) = &mut self.clipboard {
            let _ = clipboard.set_text(self.markdown_text.clone());
        }
    }

    pub fn insert_list_entry(&mut self, cursor_pos: Option<usize>) -> bool {
        let pos = cursor_pos.or(self.current_cursor_pos).unwrap_or(self.markdown_text.len());
        let line_start = self.markdown_text[..pos].rfind('\n').map_or(0, |p| p + 1);

        let at_line_start = pos == line_start;
        let current_line = if let Some(line_end) = self.markdown_text[line_start..].find('\n') {
            &self.markdown_text[line_start..line_start + line_end]
        } else {
            &self.markdown_text[line_start..]
        };
        let line_empty = current_line.trim().is_empty();

        let final_indent = if line_start > 0 {
            let prev_line_start = self.markdown_text[..line_start.saturating_sub(1)].rfind('\n').map_or(0, |p| p + 1);
            let prev_line = &self.markdown_text[prev_line_start..line_start.saturating_sub(1)];
            let trimmed = prev_line.trim_start();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") || trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
                prev_line.chars().take_while(|c| c.is_whitespace()).collect::<String>()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        self.undo_stack.push(self.markdown_text.clone());
        self.redo_stack.clear();

        let insert_text = if at_line_start && line_empty {
            format!("{}- ", final_indent)
        } else {
            format!("\n{}- ", final_indent)
        };

        self.markdown_text.insert_str(pos, &insert_text);

        let new_cursor_pos = pos + insert_text.len();
        self.cursor_override = Some(egui::text::CCursorRange::one(egui::text::CCursor::new(new_cursor_pos)));

        true
    }

    pub fn insert_checkbox_entry(&mut self, cursor_pos: Option<usize>) -> bool {
        let pos = cursor_pos.or(self.current_cursor_pos).unwrap_or(self.markdown_text.len());
        let line_start = self.markdown_text[..pos].rfind('\n').map_or(0, |p| p + 1);

        let at_line_start = pos == line_start;
        let current_line = if let Some(line_end) = self.markdown_text[line_start..].find('\n') {
            &self.markdown_text[line_start..line_start + line_end]
        } else {
            &self.markdown_text[line_start..]
        };
        let line_empty = current_line.trim().is_empty();

        let final_indent = if line_start > 0 {
            let prev_line_start = self.markdown_text[..line_start.saturating_sub(1)].rfind('\n').map_or(0, |p| p + 1);
            let prev_line = &self.markdown_text[prev_line_start..line_start.saturating_sub(1)];
            let trimmed = prev_line.trim_start();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") || trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
                prev_line.chars().take_while(|c| c.is_whitespace()).collect::<String>()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        self.undo_stack.push(self.markdown_text.clone());
        self.redo_stack.clear();

        let insert_text = if at_line_start && line_empty {
            format!("{}- [ ] ", final_indent)
        } else {
            format!("\n{}- [ ] ", final_indent)
        };

        self.markdown_text.insert_str(pos, &insert_text);

        let new_cursor_pos = pos + insert_text.len();
        self.cursor_override = Some(egui::text::CCursorRange::one(egui::text::CCursor::new(new_cursor_pos)));

        true
    }

    pub fn set_match_ranges(&mut self, ranges: Vec<(usize, usize)>, current: Option<usize>) {
        self.match_ranges = ranges;
        self.current_match = current;
    }

    pub fn clear_matches(&mut self) {
        self.match_ranges.clear();
        self.current_match = None;
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
        let match_ranges = self.match_ranges.clone();
        let current_match = self.current_match;

        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            let mut job = egui::text::LayoutJob::default();

            let lines: Vec<&str> = string.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                Self::highlight_markdown_line_static(line, &mut job, font_id.clone(), editor_font_size);
                if i < lines.len() - 1 {
                    job.append("\n", 0.0, egui::TextFormat {
                        font_id: font_id.clone(),
                        color: Color32::from_rgb(200, 200, 200),
                        ..Default::default()
                    });
                }
            }

            Self::apply_match_highlighting(&mut job, &match_ranges, current_match);

            ui.fonts(|f| f.layout_job(job))
        };

        let previous_text = self.markdown_text.clone();

        let text_edit = TextEdit::multiline(&mut self.markdown_text)
            .font(font_id.clone())
            .desired_width(f32::INFINITY)
            .lock_focus(true)
            .layouter(&mut layouter);

        let response = ui.add_sized(ui.available_size(), text_edit);

        self.text_edit_id = Some(response.id);

        if let Some(state) = egui::TextEdit::load_state(ui.ctx(), response.id)
            && let Some(cursor) = state.cursor.char_range()
        {
            self.current_cursor_pos = Some(cursor.primary.index);
        }

        if let Some(cursor_range) = self.cursor_override.take()
            && let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), response.id)
        {
            state.cursor.set_char_range(Some(cursor_range));
            state.store(ui.ctx(), response.id);
        }

        if self.should_focus {
            response.request_focus();
            self.should_focus = false;
        }

        let changed = response.changed() && response.has_focus();
        if changed && self.markdown_text != previous_text {
            self.undo_stack.push(previous_text);
            self.redo_stack.clear();
        }

        changed
    }

    fn highlight_markdown_line_static(line: &str, job: &mut egui::text::LayoutJob, font_id: egui::FontId, font_size: f32) {
        let trimmed = line.trim_start();

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
        } else if trimmed.starts_with("```") {
            job.append(line, 0.0, egui::TextFormat {
                font_id: egui::FontId::monospace(font_size),
                color: Color32::from_rgb(150, 120, 200),
                background: Color32::from_rgb(40, 40, 50),
                ..Default::default()
            });
        } else if trimmed.starts_with(">") {
            job.append(line, 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: Color32::from_rgb(160, 160, 160),
                italics: true,
                ..Default::default()
            });
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ")
            || (trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) && trimmed.contains(". ")) {
            job.append(line, 0.0, egui::TextFormat {
                font_id,
                color: Color32::from_rgb(60, 120, 200),
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

    fn add_header_text_static(line: &str, level: usize, color: Color32, _size: f32, job: &mut egui::text::LayoutJob, font_id: egui::FontId, _font_size: f32) {
        let prefix = "#".repeat(level);
        let prefix_with_space = format!("{} ", prefix);

        if let Some(content_start) = line.find(&prefix_with_space) {
            if content_start > 0 {
                job.append(&line[..content_start], 0.0, egui::TextFormat {
                    font_id: font_id.clone(),
                    color: Color32::from_rgb(200, 200, 200),
                    ..Default::default()
                });
            }

            job.append(&prefix, 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: Color32::from_rgb(100, 100, 100),
                ..Default::default()
            });

            job.append(" ", 0.0, egui::TextFormat {
                font_id: font_id.clone(),
                color: Color32::from_rgb(200, 200, 200),
                ..Default::default()
            });

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

    fn apply_match_highlighting(
        job: &mut egui::text::LayoutJob,
        match_ranges: &[(usize, usize)],
        current_match: Option<usize>
    ) {
        for (match_idx, &(match_start, match_end)) in match_ranges.iter().enumerate() {
            if match_start >= job.text.len() || match_end > job.text.len() || match_start >= match_end {
                continue;
            }

            let is_current = current_match == Some(match_idx);
            let bg_color = if is_current {
                Color32::from_rgb(255, 165, 0)
            } else {
                Color32::from_rgb(100, 100, 50)
            };

            let mut sections_to_add = Vec::new();
            let mut byte_pos = 0;
            let mut section_idx = 0;

            while section_idx < job.sections.len() {
                let section = &job.sections[section_idx];
                let section_start = byte_pos;
                let section_end = byte_pos + section.byte_range.len();

                if section_start < match_end && section_end > match_start {
                    let overlap_start = match_start.max(section_start);
                    let overlap_end = match_end.min(section_end);

                    if overlap_start == section_start && overlap_end == section_end {
                        job.sections[section_idx].format.background = bg_color;
                    } else {
                        let section = job.sections.remove(section_idx);
                        let text_offset = section.byte_range.start;

                        if overlap_start > section_start {
                            sections_to_add.push((section_idx, egui::text::LayoutSection {
                                leading_space: section.leading_space,
                                byte_range: text_offset..(text_offset + (overlap_start - section_start)),
                                format: section.format.clone(),
                            }));
                        }

                        let mut highlighted_format = section.format.clone();
                        highlighted_format.background = bg_color;
                        sections_to_add.push((section_idx, egui::text::LayoutSection {
                            leading_space: if overlap_start > section_start { 0.0 } else { section.leading_space },
                            byte_range: (text_offset + (overlap_start - section_start))..(text_offset + (overlap_end - section_start)),
                            format: highlighted_format,
                        }));

                        if overlap_end < section_end {
                            sections_to_add.push((section_idx, egui::text::LayoutSection {
                                leading_space: 0.0,
                                byte_range: (text_offset + (overlap_end - section_start))..section.byte_range.end,
                                format: section.format,
                            }));
                        }

                        for (idx, new_section) in sections_to_add.drain(..).rev() {
                            job.sections.insert(idx, new_section);
                        }

                        byte_pos = section_end;
                        continue;
                    }
                }

                byte_pos = section_end;
                section_idx += 1;
            }
        }
    }

}