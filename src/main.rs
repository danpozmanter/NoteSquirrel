use eframe::egui;

use crate::app_frame::AppFrame;
use crate::icon::load_app_icon;

mod file_manager;
mod icon;
mod app_frame;
mod notes_list;
mod editor;
mod rendered_view;
mod config;
mod find_replace;

fn main() -> Result<(), eframe::Error> {
    #[cfg(target_os = "linux")]
    if std::env::var_os("LIBGL_ALWAYS_SOFTWARE").is_none() {
        use std::os::unix::process::CommandExt;
        let exe = std::env::current_exe().expect("failed to get current exe path");
        let err = std::process::Command::new(exe)
            .args(std::env::args_os().skip(1))
            .env("__GLX_VENDOR_LIBRARY_NAME", "mesa")
            .env("LIBGL_ALWAYS_SOFTWARE", "1")
            .exec();
        panic!("failed to re-exec with software rendering: {err}");
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_title("Note Squirrel");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    eframe::run_native(
        "Note Squirrel",
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(|cc| {
            let mut app = AppFrame::default();
            app.setup_fonts_and_collect_errors(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}
