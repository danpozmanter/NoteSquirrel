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

fn main() -> Result<(), eframe::Error> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0])
        .with_title("Note Squirrel");

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Note Squirrel",
        options,
        Box::new(|_cc| Ok(Box::new(AppFrame::default()))),
    )
}
