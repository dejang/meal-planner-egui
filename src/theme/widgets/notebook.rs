use eframe::egui::{self, Color32, Pos2, Response, Sense, Stroke, TextEdit, Ui, Vec2};

use crate::handwriting;

#[derive(Default)]
pub struct Notebook;

impl Notebook {
    pub fn ui(ui: &mut Ui, value: &mut String) -> Response {
        // 1) Decide how tall the "page" is:
        let line_count = 15;
        let text_style = handwriting();
        let font_id = ui.style().text_styles[&text_style].clone();
        let line_height = ui.fonts(|fonts| fonts.row_height(&font_id));
        let total_height = line_count as f32 * line_height;

        // 2) Allocate the full rectangle for lines + text
        let desired_size = egui::vec2(ui.available_width(), total_height);
        let (rect, resp) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        // If it's offscreen or scrolled away, no need to paint or do anything
        if !ui.is_rect_visible(rect) {
            return resp;
        }

        // 3) Paint the "notebook lines" behind where the text will go
        let painter = ui.painter_at(rect);
        let offset_y = 2.0; // small vertical offset so text baseline sits nicely on the line
        for i in 0..line_count {
            let y = rect.top() + i as f32 * line_height + offset_y;
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(1.0, Color32::LIGHT_GRAY),
            );
        }

        // 4) Place a text editor *over* the same rect with a transparent background
        ui.allocate_ui_at_rect(rect, |ui| {
            // We want the text editor to fill the entire rect:
            let size = ui.available_size(); // same as `rect.size()`

            ui.add_sized(
                size,
                TextEdit::multiline(value)
                    .frame(false) // no background box
                    .margin(Vec2::ZERO) // no internal padding
                    .lock_focus(true)
                    .desired_width(size.x) // ensure it’s as wide as the rect
                    .desired_rows(line_count) // helps ensure it’s tall enough
                    .font(text_style),
            )
        })
        .inner
    }
}
