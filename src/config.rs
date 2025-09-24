use std::path::PathBuf;
use std::fs;
use egui::{Color32, FontId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownStyle {
    pub font_size: f32,
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownStyles {
    pub h1: MarkdownStyle,
    pub h2: MarkdownStyle,
    pub h3: MarkdownStyle,
    pub h4: MarkdownStyle,
    pub h5: MarkdownStyle,
    pub h6: MarkdownStyle,
    pub paragraph: MarkdownStyle,
    pub strong: MarkdownStyle,
    pub emphasis: MarkdownStyle,
    pub strikethrough: MarkdownStyle,
    pub code_inline: MarkdownStyle,
    pub code_block: MarkdownStyle,
    pub code_block_background: [u8; 3],
    pub list_bullet: MarkdownStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub notes_folder: PathBuf,
    pub editor_font_size: f32,
    pub list_font_size: f32,
    pub rendered_font_size: f32,
    pub markdown_styles: MarkdownStyles,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = std::env::home_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            notes_folder: home_dir.join("local-notes"),
            editor_font_size: 14.0,
            list_font_size: 14.0,
            rendered_font_size: 14.0,
            markdown_styles: MarkdownStyles {
                h1: MarkdownStyle { font_size: 24.0, color: [255, 220, 100] },
                h2: MarkdownStyle { font_size: 20.0, color: [220, 255, 180] },
                h3: MarkdownStyle { font_size: 18.0, color: [180, 220, 255] },
                h4: MarkdownStyle { font_size: 16.0, color: [255, 180, 220] },
                h5: MarkdownStyle { font_size: 14.0, color: [220, 180, 255] },
                h6: MarkdownStyle { font_size: 12.0, color: [255, 255, 180] },
                paragraph: MarkdownStyle { font_size: 14.0, color: [240, 240, 240] },
                strong: MarkdownStyle { font_size: 14.0, color: [255, 255, 255] },
                emphasis: MarkdownStyle { font_size: 14.0, color: [220, 180, 255] },
                strikethrough: MarkdownStyle { font_size: 14.0, color: [150, 150, 150] },
                code_inline: MarkdownStyle { font_size: 14.0, color: [200, 80, 20] },
                code_block: MarkdownStyle { font_size: 12.0, color: [150, 120, 200] },
                code_block_background: [40, 40, 50],
                list_bullet: MarkdownStyle { font_size: 14.0, color: [60, 120, 200] },
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match toml::from_str(&content) {
                        Ok(config) => config,
                        Err(_) => {
                            let default_config = Self::default();
                            default_config.save();
                            default_config
                        }
                    }
                }
                Err(_) => {
                    let default_config = Self::default();
                    default_config.save();
                    default_config
                }
            }
        } else {
            let default_config = Self::default();
            default_config.save();
            default_config
        }
    }

    pub fn save(&self) {
        let config_path = Self::get_config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        if let Ok(content) = toml::to_string_pretty(self) {
            fs::write(&config_path, content).ok();
        }
    }

    fn get_config_path() -> PathBuf {
        let home_dir = std::env::home_dir().unwrap_or_else(|| PathBuf::from("."));

        #[cfg(target_os = "linux")]
        let config_dir = home_dir.join(".config").join("NoteSquirrel");

        #[cfg(target_os = "macos")]
        let config_dir = home_dir.join("Library").join("Application Support").join("NoteSquirrel");

        #[cfg(target_os = "windows")]
        let config_dir = home_dir.join("AppData").join("Roaming").join("NoteSquirrel");

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let config_dir = home_dir.join(".config").join("NoteSquirrel");

        config_dir.join("config.toml")
    }
}

impl MarkdownStyle {
    pub fn to_color32(&self) -> Color32 {
        Color32::from_rgb(self.color[0], self.color[1], self.color[2])
    }

    pub fn to_font_id(&self) -> FontId {
        FontId::proportional(self.font_size)
    }
}