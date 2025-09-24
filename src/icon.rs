#[cfg(not(debug_assertions))]
mod embedded_icon {
    include!(concat!(env!("OUT_DIR"), "/embedded_icon.rs"));
}

pub fn load_app_icon() -> Option<egui::IconData> {
    #[cfg(not(debug_assertions))]
    {
        if option_env!("EMBEDDED_ICON_FILE").is_some() {
            return Some(egui::IconData {
                rgba: embedded_icon::ICON_RGBA.to_vec(),
                width: embedded_icon::ICON_WIDTH,
                height: embedded_icon::ICON_HEIGHT,
            });
        }
    }

    if let Some(icon_path) = option_env!("APP_ICON")
        && let Ok(icon_bytes) = std::fs::read(icon_path)
            && let Ok(image) = image::load_from_memory(&icon_bytes) {
                let rgba = image.to_rgba8();
                let (width, height) = rgba.dimensions();
                return Some(egui::IconData {
                    rgba: rgba.into_raw(),
                    width,
                    height,
                });
            }

    if let Ok(icon_bytes) = std::fs::read("images/NoteSquirrelIcon.png")
        && let Ok(image) = image::load_from_memory(&icon_bytes) {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            return Some(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            });
        }

    None
}