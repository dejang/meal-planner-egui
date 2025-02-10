use std::sync::Arc;

use egui::{
    epaint::text::{FontInsert, InsertFontFamily},
    FontData, TextStyle,
};

pub static GEIST_BLACK: &[u8] = include_bytes!("geist/Geist-Black.ttf");
pub static GEIST_BOLD: &[u8] = include_bytes!("geist/Geist-Bold.ttf");
pub static GEIST_EXTRA_BOLD: &[u8] = include_bytes!("geist/Geist-ExtraBold.ttf");
pub static GEIST_EXTRA_LIGHT: &[u8] = include_bytes!("geist/Geist-ExtraLight.ttf");
pub static GEIST_MEDIUM: &[u8] = include_bytes!("geist/Geist-Medium.ttf");
pub static GEIST_REGULAR: &[u8] = include_bytes!("geist/Geist-Regular.ttf");
pub static GEIST_SEMI_BOLD: &[u8] = include_bytes!("geist/Geist-SemiBold.ttf");
pub static GEIST_THIN: &[u8] = include_bytes!("geist/Geist-Thin.ttf");

#[inline]
pub fn heading2() -> TextStyle {
    TextStyle::Name("Heading2".into())
}

#[inline]
pub fn heading3() -> TextStyle {
    TextStyle::Name("ContextHeading".into())
}

pub fn install_fonts(ctx: &egui::Context) {
    ctx.add_font(FontInsert::new(
        "my_font",
        FontData::from_static(GEIST_REGULAR),
        vec![InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: egui::epaint::text::FontPriority::Highest,
        }],
    ));

    ctx.add_font(FontInsert::new(
        "icons",
        FontData::from_static(include_bytes!("icons/lucide.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name(Arc::from("icons")),
            priority: egui::epaint::text::FontPriority::Highest,
        }],
    ));
}
