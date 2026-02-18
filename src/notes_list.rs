use eframe::egui;

use crate::file_manager::FileManager;
use crate::config::Config;

#[derive(PartialEq, Clone)]
pub enum SortOrder {
    Alphabetical,
    LastModified,
}

pub struct NotesList {
    file_manager: FileManager,
    config: Config,
    notes_list: Vec<String>,
    current_note_index: usize,
    search_text: String,
    editing_note_name: Option<usize>,
    temp_note_name: String,
    current_content: Vec<String>,
    sort_order: SortOrder,
    display_order: Vec<usize>,
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
            current_content: Vec::new(),
            sort_order: SortOrder::Alphabetical,
            display_order: Vec::new(),
        }
    }

    pub fn load_notes(&mut self) {
        self.notes_list = self.file_manager.load_note_names();
        self.initialize_content_vectors();
        self.load_all_content();
        self.compute_display_order();
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

    pub fn create_new_note(&mut self) -> Option<String> {
        let new_note_name = format!("Note {}", self.notes_list.len() + 1);
        if self.file_manager.create_note(&new_note_name) {
            self.notes_list.push(new_note_name.clone());
            self.current_content.push(String::new());

            self.current_note_index = self.notes_list.len() - 1;
            self.compute_display_order();
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
            self.compute_display_order();
            true
        } else {
            false
        }
    }

    pub fn find_note_index(&self, name: &str) -> Option<usize> {
        self.notes_list.iter().position(|n| n == name)
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
            let note_name = self.notes_list[self.current_note_index].clone();
            self.file_manager.write_note_content(&note_name, content);
        }
    }

    pub fn set_sort_order(&mut self, order: SortOrder) {
        self.sort_order = order;
        self.compute_display_order();
    }

    pub fn get_sort_order(&self) -> &SortOrder {
        &self.sort_order
    }

    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<usize> {
        let mut switch_to_note_index = None;
        let mut start_editing_index = None;
        let mut finish_editing = false;
        let mut rename_action = None;

        for display_pos in 0..self.display_order.len() {
            let index = self.display_order[display_pos];
            let note_name = self.notes_list[index].clone();

            if !self.search_text.is_empty()
                && !note_name
                    .to_lowercase()
                    .contains(&self.search_text.to_lowercase())
            {
                continue;
            }

            let is_selected = index == self.current_note_index;

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
                    let button_label = egui::RichText::new(note_name.as_str())
                        .color(egui::Color32::WHITE)
                        .font(self.config.get_list_font_id(self.config.list_font_size))
                        .strong();

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
        self.current_content.clear();

        for _ in &self.notes_list {
            self.current_content.push(String::new());
        }
    }

    fn load_all_content(&mut self) {
        for (i, note_name) in self.notes_list.iter().enumerate() {
            let content = self.file_manager.read_note_content(note_name);
            self.current_content[i] = content;
        }
    }

    fn remove_note_from_vectors(&mut self, index: usize) {
        self.notes_list.remove(index);
        self.current_content.remove(index);
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

    fn compute_display_order(&mut self) {
        let mut indices: Vec<usize> = (0..self.notes_list.len()).collect();

        match self.sort_order {
            SortOrder::Alphabetical => {
                let notes_list = &self.notes_list;
                indices.sort_by(|&a, &b| {
                    notes_list[a].to_lowercase().cmp(&notes_list[b].to_lowercase())
                });
            }
            SortOrder::LastModified => {
                let notes_list = &self.notes_list;
                let file_manager = &self.file_manager;
                indices.sort_by(|&a, &b| {
                    let time_a = file_manager.get_note_modified_time(&notes_list[a]);
                    let time_b = file_manager.get_note_modified_time(&notes_list[b]);
                    time_b.cmp(&time_a)
                });
            }
        }

        self.display_order = indices;
    }
}
