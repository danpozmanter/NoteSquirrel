use std::fs;
use std::path::PathBuf;
use crate::config::Config;

pub struct FileManager {
    notes_dir: PathBuf,
}

impl FileManager {
    pub fn new(config: &Config) -> Self {
        let notes_dir = config.notes_folder.clone();
        fs::create_dir_all(&notes_dir).ok();

        Self { notes_dir }
    }

    pub fn load_note_names(&self) -> Vec<String> {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.notes_dir) {
            files = entries
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    if path.extension()? == "md" {
                        let file_name = path.file_stem()?.to_str()?.to_string();
                        Some(file_name)
                    } else {
                        None
                    }
                })
                .collect();
            files.sort();
        }

        if files.is_empty() {
            let default_name = "Welcome".to_string();
            let default_path = self.notes_dir.join(format!("{}.md", default_name));
            fs::write(&default_path, "").ok();
            files.push(default_name);
        }

        files
    }

    pub fn read_note_content(&self, note_name: &str) -> String {
        let file_path = self.notes_dir.join(format!("{}.md", note_name));
        fs::read_to_string(&file_path).unwrap_or_default()
    }

    pub fn write_note_content(&self, note_name: &str, content: &str) -> bool {
        let file_path = self.notes_dir.join(format!("{}.md", note_name));
        fs::write(&file_path, content).is_ok()
    }

    pub fn create_note(&self, note_name: &str) -> bool {
        let file_path = self.notes_dir.join(format!("{}.md", note_name));
        fs::write(&file_path, "").is_ok()
    }

    pub fn delete_note(&self, note_name: &str) -> bool {
        let file_path = self.notes_dir.join(format!("{}.md", note_name));
        fs::remove_file(&file_path).is_ok()
    }

    pub fn rename_note(&self, old_name: &str, new_name: &str) -> bool {
        let old_path = self.notes_dir.join(format!("{}.md", old_name));
        let new_path = self.notes_dir.join(format!("{}.md", new_name));
        fs::rename(&old_path, &new_path).is_ok()
    }

    pub fn get_note_modified_time(&self, note_name: &str) -> Option<std::time::SystemTime> {
        let file_path = self.notes_dir.join(format!("{}.md", note_name));
        fs::metadata(file_path).and_then(|m| m.modified()).ok()
    }
}