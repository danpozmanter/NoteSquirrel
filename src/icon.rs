pub fn load_app_icon() -> Option<egui::IconData> {
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

    if let Ok(icon_bytes) = std::fs::read("icons/NoteSquirrelIcon.png")
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