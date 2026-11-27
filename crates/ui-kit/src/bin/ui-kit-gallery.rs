//! The UI kit gallery window (#4394): every token of the dark and the
//! light theme side by side, for reviewers. Build and run it with
//! `cargo run -p camcraft-ui-kit --features gallery --bin ui-kit-gallery`.
//! Provenance: our own tool over eframe's documented native runner.

use camcraft_ui_kit::{gallery, theme};

struct Gallery;

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::ScrollArea::vertical().show(ui, gallery::show);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("camcraft UI kit gallery")
            .with_inner_size([960.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "camcraft-ui-kit-gallery",
        options,
        Box::new(|cc| {
            // The window ground wears the dark column; each half paints its own.
            theme::apply_to(&cc.egui_ctx, &theme::Tokens::DARK);
            Ok(Box::new(Gallery))
        }),
    )
}
