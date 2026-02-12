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
    {
        if sw_render_flag_path().exists() {
            unsafe {
                std::env::set_var("__GLX_VENDOR_LIBRARY_NAME", "mesa");
                std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
            }
        }
    }

    match run_app() {
        Ok(()) => Ok(()),
        #[cfg(target_os = "linux")]
        Err(first_err) => {
            if sw_render_flag_path().exists() {
                return Err(first_err);
            }

            let flag = sw_render_flag_path();
            if let Some(parent) = flag.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&flag, "");

            eprintln!("Hardware rendering failed: {first_err}");
            eprintln!("Falling back to software rendering...");

            let exe = std::env::current_exe().map_err(|_| first_err)?;
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new(exe)
                .args(std::env::args_os().skip(1))
                .env("__GLX_VENDOR_LIBRARY_NAME", "mesa")
                .env("LIBGL_ALWAYS_SOFTWARE", "1")
                .exec();
            panic!("Failed to re-exec: {err}");
        }
        #[cfg(not(target_os = "linux"))]
        Err(e) => Err(e),
    }
}

#[cfg(target_os = "linux")]
fn sw_render_flag_path() -> std::path::PathBuf {
    let home = std::env::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    home.join(".config").join("NoteSquirrel").join(".sw_render")
}

fn run_app() -> Result<(), eframe::Error> {
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
