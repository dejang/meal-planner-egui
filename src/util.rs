use egui::RichText;

use crate::{helvetica_body, helvetica_heading, helvetica_small};

pub const DEFAULT_PADDING: f32 = 10.;

pub fn percentage(value: f32, percent: u8) -> f32 {
    percent as f32 * value / 100.
}

pub fn hh(str: &str) -> RichText {
    RichText::new(str).text_style(helvetica_heading())
}

pub fn hb(str: &str) -> RichText {
    RichText::new(str).text_style(helvetica_body())
}

pub fn hs(str: &str) -> RichText {
    RichText::new(str).text_style(helvetica_small())
}
