use eframe::egui;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Match {
    pub start: usize,
    pub end: usize,
}

pub struct FindReplace {
    pub show_dialog: bool,
    pub find_text: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub matches: Vec<Match>,
    pub current_match_index: Option<usize>,
    find_text_changed: bool,
    should_focus: bool,
}

impl FindReplace {
    pub fn new() -> Self {
        Self {
            show_dialog: false,
            find_text: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            use_regex: false,
            matches: Vec::new(),
            current_match_index: None,
            find_text_changed: false,
            should_focus: false,
        }
    }

    pub fn toggle_dialog(&mut self) {
        self.show_dialog = !self.show_dialog;
        if self.show_dialog {
            self.find_text_changed = true;
            self.should_focus = true;
        }
    }

    pub fn close_dialog(&mut self) {
        self.show_dialog = false;
        self.matches.clear();
        self.current_match_index = None;
    }

    pub fn update_matches(&mut self, text: &str) {
        if self.find_text.is_empty() {
            self.matches.clear();
            self.current_match_index = None;
            return;
        }

        self.matches.clear();

        if self.use_regex {
            if let Ok(regex) = self.build_regex() {
                for mat in regex.find_iter(text) {
                    self.matches.push(Match {
                        start: mat.start(),
                        end: mat.end(),
                    });
                }
            }
        } else {
            let search_text = if self.case_sensitive {
                self.find_text.clone()
            } else {
                self.find_text.to_lowercase()
            };

            let haystack = if self.case_sensitive {
                text.to_string()
            } else {
                text.to_lowercase()
            };

            let mut start = 0;
            while let Some(pos) = haystack[start..].find(&search_text) {
                let absolute_pos = start + pos;
                self.matches.push(Match {
                    start: absolute_pos,
                    end: absolute_pos + self.find_text.len(),
                });
                start = absolute_pos + 1;
            }
        }

        if !self.matches.is_empty() && self.current_match_index.is_none() {
            self.current_match_index = Some(0);
        } else if self.current_match_index.is_some() && self.matches.is_empty() {
            self.current_match_index = None;
        } else if let Some(idx) = self.current_match_index
            && idx >= self.matches.len()
        {
            self.current_match_index = Some(self.matches.len().saturating_sub(1));
        }
    }

    fn build_regex(&self) -> Result<Regex, regex::Error> {
        let pattern = if self.case_sensitive {
            self.find_text.clone()
        } else {
            format!("(?i){}", self.find_text)
        };
        Regex::new(&pattern)
    }

    pub fn next_match(&mut self) {
        if self.matches.is_empty() {
            return;
        }

        self.current_match_index = Some(match self.current_match_index {
            Some(idx) => (idx + 1) % self.matches.len(),
            None => 0,
        });
    }

    pub fn previous_match(&mut self) {
        if self.matches.is_empty() {
            return;
        }

        self.current_match_index = Some(match self.current_match_index {
            Some(idx) => {
                if idx == 0 {
                    self.matches.len() - 1
                } else {
                    idx - 1
                }
            }
            None => self.matches.len() - 1,
        });
    }

    pub fn replace_current(&mut self, text: &mut String) -> bool {
        if let Some(idx) = self.current_match_index
            && idx < self.matches.len()
        {
            let mat = &self.matches[idx];

            let replacement = if self.use_regex {
                if let Ok(regex) = self.build_regex() {
                    regex.replace(&text[mat.start..mat.end], self.replace_text.as_str()).to_string()
                } else {
                    self.replace_text.clone()
                }
            } else {
                self.replace_text.clone()
            };

            text.replace_range(mat.start..mat.end, &replacement);

            self.find_text_changed = true;
            return true;
        }
        false
    }

    pub fn replace_all(&mut self, text: &mut String) -> usize {
        let count = self.matches.len();

        if count == 0 {
            return 0;
        }

        if self.use_regex {
            if let Ok(regex) = self.build_regex() {
                *text = regex.replace_all(text, self.replace_text.as_str()).to_string();
            }
        } else {
            for mat in self.matches.iter().rev() {
                if mat.start <= text.len() && mat.end <= text.len() && mat.start <= mat.end {
                    text.replace_range(mat.start..mat.end, &self.replace_text);
                }
            }
        }

        self.find_text_changed = true;
        count
    }

    pub fn render(&mut self, ctx: &egui::Context) -> FindReplaceAction {
        let mut action = FindReplaceAction::None;

        if !self.show_dialog {
            return action;
        }

        let mut close = false;

        egui::Window::new("Find & Replace")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .fixed_size(egui::Vec2::new(400.0, 0.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Find:");
                        let find_response = ui.add_sized(
                            egui::Vec2::new(ui.available_width(), 20.0),
                            egui::TextEdit::singleline(&mut self.find_text)
                                .hint_text("Enter search text...")
                        );

                        if self.should_focus {
                            find_response.request_focus();
                            self.should_focus = false;
                        }

                        if find_response.changed() {
                            self.find_text_changed = true;
                        }

                        if self.find_text_changed && find_response.has_focus() {
                            action = FindReplaceAction::UpdateMatches;
                        }

                        if find_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            action = FindReplaceAction::NextMatch;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Replace:");
                        let replace_response = ui.add_sized(
                            egui::Vec2::new(ui.available_width(), 20.0),
                            egui::TextEdit::singleline(&mut self.replace_text)
                                .hint_text("Enter replacement text...")
                        );

                        if replace_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            action = FindReplaceAction::ReplaceCurrent;
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut self.case_sensitive, "Match case").changed() {
                            self.find_text_changed = true;
                            action = FindReplaceAction::UpdateMatches;
                        }
                        if ui.checkbox(&mut self.use_regex, "Regex").changed() {
                            self.find_text_changed = true;
                            action = FindReplaceAction::UpdateMatches;
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        let match_text = if self.matches.is_empty() {
                            "No matches".to_string()
                        } else if let Some(idx) = self.current_match_index {
                            format!("{} of {}", idx + 1, self.matches.len())
                        } else {
                            format!("{} matches", self.matches.len())
                        };

                        ui.label(match_text);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mut replace_all_text = egui::text::LayoutJob::default();
                            replace_all_text.append("Replace ", 0.0, egui::TextFormat::default());
                            replace_all_text.append("A", 0.0, egui::TextFormat {
                                underline: egui::Stroke::new(1.0, ui.style().visuals.text_color()),
                                ..Default::default()
                            });
                            replace_all_text.append("ll", 0.0, egui::TextFormat::default());

                            if ui.button(replace_all_text).clicked() {
                                action = FindReplaceAction::ReplaceAll;
                            }

                            let mut replace_text = egui::text::LayoutJob::default();
                            replace_text.append("R", 0.0, egui::TextFormat {
                                underline: egui::Stroke::new(1.0, ui.style().visuals.text_color()),
                                ..Default::default()
                            });
                            replace_text.append("eplace", 0.0, egui::TextFormat::default());

                            if ui.button(replace_text).clicked() {
                                action = FindReplaceAction::ReplaceCurrent;
                            }

                            if ui.button("Previous (Shift+F3)").clicked() {
                                action = FindReplaceAction::PreviousMatch;
                            }
                            if ui.button("Next (F3)").clicked() {
                                action = FindReplaceAction::NextMatch;
                            }
                        });
                    });
                });

                ui.input_mut(|i| {
                    if i.key_pressed(egui::Key::Escape) {
                        close = true;
                    }

                    if i.consume_key(egui::Modifiers::ALT, egui::Key::R) {
                        action = FindReplaceAction::ReplaceCurrent;
                    }

                    if i.consume_key(egui::Modifiers::ALT, egui::Key::A) {
                        action = FindReplaceAction::ReplaceAll;
                    }
                });
            });

        if close {
            self.close_dialog();
        }

        if self.find_text_changed && matches!(action, FindReplaceAction::UpdateMatches) {
            self.find_text_changed = false;
        }

        action
    }

    pub fn get_match_ranges(&self) -> Vec<(usize, usize)> {
        self.matches.iter().map(|m| (m.start, m.end)).collect()
    }
}

#[derive(Debug, PartialEq)]
pub enum FindReplaceAction {
    None,
    UpdateMatches,
    NextMatch,
    PreviousMatch,
    ReplaceCurrent,
    ReplaceAll,
}

impl Default for FindReplace {
    fn default() -> Self {
        Self::new()
    }
}
