use eframe::egui;

use crate::file_manager::FileManager;
use crate::config::Config;

pub struct NotesList {
    file_manager: FileManager,
    config: Config,
    notes_list: Vec<String>,
    current_note_index: usize,
    search_text: String,
    editing_note_name: Option<usize>,
    temp_note_name: String,
    original_content: Vec<String>,
    current_content: Vec<String>,
    is_dirty: Vec<bool>,
}

impl NotesList {
    pub fn new(config: &Config) -> Self {
        Self {
            file_manager: FileManager::new(config),
            config: config.clone(),
            notes_list: Vec::new(),
            current_note_index: 0,
            search_text: String::new(),
            editing_note_name: None,
            temp_note_name: String::new(),
            original_content: Vec::new(),
            current_content: Vec::new(),
            is_dirty: Vec::new(),
        }
    }

    pub fn load_notes(&mut self) {
        self.notes_list = self.file_manager.load_note_names();
        self.initialize_content_vectors();
        self.load_all_content();
    }

    pub fn get_search_text_mut(&mut self) -> &mut String {
        &mut self.search_text
    }

    pub fn get_current_note_name(&self) -> &str {
        self.notes_list.get(self.current_note_index).map(|s| s.as_str()).unwrap_or("No Note")
    }

    pub fn get_current_content(&self) -> &str {
        if self.current_note_index < self.current_content.len() {
            &self.current_content[self.current_note_index]
        } else {
            ""
        }
    }

    pub fn is_current_note_dirty(&self) -> bool {
        self.is_dirty.get(self.current_note_index).copied().unwrap_or(false)
    }

    pub fn has_any_dirty_notes(&self) -> bool {
        self.is_dirty.iter().any(|&dirty| dirty)
    }

    pub fn save_all_notes(&mut self) -> bool {
        let mut all_saved = true;
        for i in 0..self.notes_list.len() {
            if self.is_dirty.get(i).copied().unwrap_or(false) {
                let note_name = &self.notes_list[i].clone();
                let content = &self.current_content[i].clone();
                if self.file_manager.write_note_content(note_name, content) {
                    self.original_content[i] = content.clone();
                    self.is_dirty[i] = false;
                } else {
                    all_saved = false;
                }
            }
        }
        all_saved
    }

    pub fn save_current_note(&mut self, note_name: &str, content: &str) -> bool {
        if self.file_manager.write_note_content(note_name, content) {
            if self.current_note_index < self.original_content.len() {
                self.original_content[self.current_note_index] = content.to_string();
                self.current_content[self.current_note_index] = content.to_string();
            }
            true
        } else {
            false
        }
    }

    pub fn create_new_note(&mut self) -> Option<String> {
        let new_note_name = format!("Note {}", self.notes_list.len() + 1);
        if self.file_manager.create_note(&new_note_name) {
            self.notes_list.push(new_note_name.clone());
            self.original_content.push(String::new());
            self.current_content.push(String::new());
            self.is_dirty.push(false);

            self.current_note_index = self.notes_list.len() - 1;
            Some(new_note_name)
        } else {
            None
        }
    }

    pub fn delete_current_note(&mut self) -> bool {
        if self.current_note_index >= self.notes_list.len() {
            return false;
        }

        let note_name = &self.notes_list[self.current_note_index];
        if self.file_manager.delete_note(note_name) {
            self.remove_note_from_vectors(self.current_note_index);
            self.adjust_current_index_after_deletion();
            true
        } else {
            false
        }
    }

    pub fn switch_to_note(&mut self, index: usize) -> bool {
        if index < self.notes_list.len() {
            self.current_note_index = index;
            true
        } else {
            false
        }
    }

    pub fn save_current_content(&mut self, content: &str) {
        if self.current_note_index < self.current_content.len() {
            self.current_content[self.current_note_index] = content.to_string();
            self.update_dirty_state();
        }
    }

    pub fn update_dirty_state(&mut self) {
        if self.current_note_index < self.current_content.len() {
            self.is_dirty[self.current_note_index] =
                self.current_content[self.current_note_index] != self.original_content[self.current_note_index];
        }
    }

    pub fn mark_current_clean(&mut self) {
        if self.current_note_index < self.is_dirty.len() {
            self.is_dirty[self.current_note_index] = false;
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<usize> {
        let mut switch_to_note_index = None;
        let mut start_editing_index = None;
        let mut finish_editing = false;
        let mut rename_action = None;

        for (index, note_name) in self.notes_list.iter().enumerate() {
            if !self.search_text.is_empty()
                && !note_name
                    .to_lowercase()
                    .contains(&self.search_text.to_lowercase())
            {
                continue;
            }

            let is_selected = index == self.current_note_index;
            let is_dirty = self.is_dirty.get(index).copied().unwrap_or(false);

            ui.horizontal(|ui| {
                if self.editing_note_name == Some(index) {
                    let response = ui.add_sized(
                        [ui.available_width(), 25.0],
                        egui::TextEdit::singleline(&mut self.temp_note_name)
                            .id(egui::Id::new(format!("edit_note_{}", index)))
                    );

                    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let old_name = note_name.clone();
                        let new_name = self.temp_note_name.clone();

                        if !new_name.is_empty() && new_name != old_name {
                            rename_action = Some((old_name, new_name));
                        }
                        finish_editing = true;
                    }

                    response.request_focus();
                } else {
                    let button_label = if is_dirty {
                        egui::RichText::new(note_name)
                            .color(egui::Color32::from_rgb(255, 150, 150))
                            .font(self.config.get_list_font_id(self.config.list_font_size))
                            .strong()
                    } else {
                        egui::RichText::new(note_name)
                            .color(egui::Color32::WHITE)
                            .font(self.config.get_list_font_id(self.config.list_font_size))
                            .strong()
                    };

                    let button = if is_selected {
                        let button = egui::Button::new(button_label)
                            .fill(egui::Color32::from_rgb(60, 120, 200));
                        ui.add_sized([ui.available_width(), 25.0], button)
                    } else {
                        ui.add_sized([ui.available_width(), 25.0], egui::Button::new(button_label))
                    };

                    if button.clicked() && index != self.current_note_index {
                        switch_to_note_index = Some(index);
                    }

                    if button.double_clicked() {
                        start_editing_index = Some(index);
                    }
                }
            });
        }

        if let Some(idx) = start_editing_index {
            self.editing_note_name = Some(idx);
            self.temp_note_name = self.notes_list[idx].clone();
        }
        if finish_editing {
            self.editing_note_name = None;
        }
        if let Some((old, new)) = rename_action {
            self.rename_note(&old, &new);
        }

        switch_to_note_index
    }

    fn initialize_content_vectors(&mut self) {
        self.original_content.clear();
        self.current_content.clear();
        self.is_dirty.clear();

        for _ in &self.notes_list {
            self.original_content.push(String::new());
            self.current_content.push(String::new());
            self.is_dirty.push(false);
        }
    }

    fn load_all_content(&mut self) {
        for (i, note_name) in self.notes_list.iter().enumerate() {
            let content = self.file_manager.read_note_content(note_name);
            self.original_content[i] = content.clone();
            self.current_content[i] = content;
            self.is_dirty[i] = false;
        }
    }

    fn remove_note_from_vectors(&mut self, index: usize) {
        self.notes_list.remove(index);
        self.original_content.remove(index);
        self.current_content.remove(index);
        self.is_dirty.remove(index);
    }

    fn adjust_current_index_after_deletion(&mut self) {
        if self.current_note_index >= self.notes_list.len() && !self.notes_list.is_empty() {
            self.current_note_index = self.notes_list.len() - 1;
        }
    }

    fn rename_note(&mut self, old_name: &str, new_name: &str) {
        if self.file_manager.rename_note(old_name, new_name)
            && let Some(index) = self.notes_list.iter().position(|name| name == old_name) {
                self.notes_list[index] = new_name.to_string();
            }
    }
}