use std::path::PathBuf;
use std::fs;
use egui::{Color32, FontId, FontDefinitions, FontData, FontFamily};
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
    pub editor_font_family: String,
    pub list_font_family: String,
    pub rendered_font_family: String,
    pub markdown_styles: MarkdownStyles,
    #[serde(default)]
    pub last_open_note: Option<String>,
    #[serde(skip)]
    pub loaded_fonts: LoadedFonts,
}

#[derive(Debug, Clone)]
pub struct ConfigLoadResult {
    pub config: Config,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct LoadedFonts {
    pub editor_loaded: bool,
    pub list_loaded: bool,
    pub rendered_loaded: bool,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = std::env::home_dir().unwrap_or_else(|| PathBuf::from("."));

        let default_mono_font = "monospace".to_string();

        Self {
            notes_folder: home_dir.join("local-notes"),
            editor_font_size: 14.0,
            list_font_size: 14.0,
            rendered_font_size: 14.0,
            editor_font_family: default_mono_font.clone(),
            list_font_family: default_mono_font.clone(),
            rendered_font_family: default_mono_font.clone(),
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
            last_open_note: None,
            loaded_fonts: LoadedFonts::default(),
        }
    }
}

impl Config {
    pub fn setup_fonts(&self, ctx: &egui::Context) -> (LoadedFonts, Vec<String>) {
        let mut fonts = FontDefinitions::default();
        let mut errors = Vec::new();
        let mut loaded_fonts = LoadedFonts {
            editor_loaded: false,
            list_loaded: false,
            rendered_loaded: false,
        };

        let font_configs = [
            (&self.editor_font_family, "editor_font", "Editor", &mut loaded_fonts.editor_loaded),
            (&self.list_font_family, "list_font", "List", &mut loaded_fonts.list_loaded),
            (&self.rendered_font_family, "rendered_font", "Rendered view", &mut loaded_fonts.rendered_loaded),
        ];

        for (font_name, family_key, component_name, loaded_flag) in font_configs {
            if font_name != "monospace" && font_name != "proportional" {
                match Self::try_load_system_font(font_name, &mut fonts, family_key) {
                    Ok(()) => {
                        *loaded_flag = true;
                    },
                    Err(e) => {
                        errors.push(format!("{} font '{}' not found: {}, using fallback", component_name, font_name, e));
                    }
                }
            }
        }

        ctx.set_fonts(fonts);
        (loaded_fonts, errors)
    }

    fn try_load_system_font(font_name: &str, fonts: &mut FontDefinitions, family_key: &str) -> Result<(), String> {
        let font_paths = Self::get_system_font_paths(font_name);

        for path in font_paths {
            if let Ok(font_data) = fs::read(&path) {
                fonts.font_data.insert(
                    family_key.to_owned(),
                    FontData::from_owned(font_data)
                );
                fonts.families.insert(
                    FontFamily::Name(family_key.into()),
                    vec![family_key.to_owned()]
                );
                return Ok(());
            }
        }

        Err("Font file not found in system paths".to_string())
    }

    fn get_system_font_paths(font_name: &str) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        let mut font_variations = vec![
            format!("{}.ttf", font_name),
            format!("{}.otf", font_name),
            format!("{}.TTF", font_name),
            format!("{}.OTF", font_name),
            font_name.replace(" ", "").to_lowercase() + ".ttf",
            font_name.replace(" ", "").to_lowercase() + ".otf",
            font_name.replace(" ", "-").to_lowercase() + ".ttf",
            font_name.replace(" ", "-").to_lowercase() + ".otf",
        ];

        match font_name.to_lowercase().as_str() {
            "arial" => {
                font_variations.extend([
                    "arial.ttf".to_string(),
                    "Arial.ttf".to_string(),
                    "ARIAL.TTF".to_string(),
                    "arial-regular.ttf".to_string(),
                    "ArialRegular.ttf".to_string(),
                ]);
            }
            "courier new" => {
                font_variations.extend([
                    "cour.ttf".to_string(),
                    "courr.ttf".to_string(),
                    "courier-new.ttf".to_string(),
                    "CourierNew.ttf".to_string(),
                    "CourierNewRegular.ttf".to_string(),
                    "Courier New.ttf".to_string(),
                    "COUR.TTF".to_string(),
                ]);
            }
            "roboto" => {
                font_variations.extend([
                    "Roboto-Regular.ttf".to_string(),
                    "RobotoRegular.ttf".to_string(),
                ]);
            }
            "open sans" => {
                font_variations.extend([
                    "OpenSans-Regular.ttf".to_string(),
                    "OpenSansRegular.ttf".to_string(),
                ]);
            }
            "dejavu sans" => {
                font_variations.extend([
                    "DejaVuSans.ttf".to_string(),
                    "dejavu/DejaVuSans.ttf".to_string(),
                ]);
            }
            "dejavu sans mono" => {
                font_variations.extend([
                    "DejaVuSansMono.ttf".to_string(),
                    "dejavu/DejaVuSansMono.ttf".to_string(),
                ]);
            }
            "dejavu serif" => {
                font_variations.extend([
                    "DejaVuSerif.ttf".to_string(),
                    "dejavu/DejaVuSerif.ttf".to_string(),
                ]);
            }
            "liberation mono" => {
                font_variations.extend([
                    "LiberationMono-Regular.ttf".to_string(),
                    "liberation/LiberationMono-Regular.ttf".to_string(),
                ]);
            }
            "liberation sans" => {
                font_variations.extend([
                    "LiberationSans-Regular.ttf".to_string(),
                    "liberation/LiberationSans-Regular.ttf".to_string(),
                ]);
            }
            "liberation serif" => {
                font_variations.extend([
                    "LiberationSerif-Regular.ttf".to_string(),
                    "liberation/LiberationSerif-Regular.ttf".to_string(),
                ]);
            }
            _ => {}
        }

        #[cfg(target_os = "windows")]
        {
            let system_paths = [
                PathBuf::from("C:/Windows/Fonts/"),
                PathBuf::from("C:/Windows/System32/Fonts/"),
            ];
            for base_path in &system_paths {
                for variation in &font_variations {
                    paths.push(base_path.join(variation));
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let system_paths = [
                PathBuf::from("/System/Library/Fonts/"),
                PathBuf::from("/Library/Fonts/"),
                PathBuf::from(std::env::home_dir().unwrap_or_default()).join("Library/Fonts/"),
            ];
            for base_path in &system_paths {
                for variation in &font_variations {
                    paths.push(base_path.join(variation));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let system_paths = [
                PathBuf::from("/usr/share/fonts/"),
                PathBuf::from("/usr/local/share/fonts/"),
                std::env::home_dir().unwrap_or_default().join(".fonts/"),
                std::env::home_dir().unwrap_or_default().join(".local/share/fonts/"),
            ];
            for base_path in &system_paths {
                for variation in &font_variations {
                    paths.push(base_path.join(variation));
                    paths.push(base_path.join("truetype").join(variation));
                    paths.push(base_path.join("opentype").join(variation));
                    paths.push(base_path.join("TTF").join(variation));
                    paths.push(base_path.join("OTF").join(variation));
                }
            }
        }

        paths
    }

    pub fn get_editor_font_id(&self, size: f32) -> FontId {
        if self.editor_font_family == "proportional" {
            FontId::proportional(size)
        } else if self.editor_font_family == "monospace" {
            FontId::monospace(size)
        } else if self.loaded_fonts.editor_loaded {
            FontId {
                size,
                family: FontFamily::Name("editor_font".into()),
            }
        } else {
            FontId::monospace(size)
        }
    }

    pub fn get_list_font_id(&self, size: f32) -> FontId {
        if self.list_font_family == "proportional" {
            FontId::proportional(size)
        } else if self.list_font_family == "monospace" {
            FontId::monospace(size)
        } else if self.loaded_fonts.list_loaded {
            FontId {
                size,
                family: FontFamily::Name("list_font".into()),
            }
        } else {
            FontId::monospace(size)
        }
    }

    pub fn get_rendered_font_id(&self, size: f32) -> FontId {
        if self.rendered_font_family == "proportional" {
            FontId::proportional(size)
        } else if self.rendered_font_family == "monospace" {
            FontId::monospace(size)
        } else if self.loaded_fonts.rendered_loaded {
            FontId {
                size,
                family: FontFamily::Name("rendered_font".into()),
            }
        } else {
            FontId::monospace(size)
        }
    }

    pub fn load() -> ConfigLoadResult {
        let config_path = Self::get_config_path();
        let mut errors = Vec::new();

        let config = if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match toml::from_str(&content) {
                        Ok(config) => config,
                        Err(e) => {
                            errors.push(format!("Failed to parse config file: {}", e));
                            let default_config = Self::default();
                            if let Err(e) = default_config.save() {
                                errors.push(format!("Failed to save default config: {}", e));
                            }
                            default_config
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to read config file: {}", e));
                    let default_config = Self::default();
                    if let Err(e) = default_config.save() {
                        errors.push(format!("Failed to save default config: {}", e));
                    }
                    default_config
                }
            }
        } else {
            if let Some(parent) = config_path.parent()
                && !parent.exists()
                && let Err(e) = fs::create_dir_all(parent) {
                    errors.push(format!("Failed to create config directory '{}': {}", parent.display(), e));
                }
            let default_config = Self::default();
            if let Err(e) = default_config.save() {
                errors.push(format!("Failed to save default config: {}", e));
            }
            default_config
        };

        ConfigLoadResult { config, errors }
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::get_config_path();

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&config_path, content).map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
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

}