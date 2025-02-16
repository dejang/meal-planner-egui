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
pub static RECIPE_TITLE: &[u8] = include_bytes!("./SweetButtermilkScript-Regular.ttf");

pub static HELVETICA: &[u8] = include_bytes!("./helvetica/Helvetica.ttf");
pub static HELVETICA_BOLD: &[u8] = include_bytes!("./helvetica/Helvetica-Bold.ttf");
pub static HELVETICA_LIGHT: &[u8] = include_bytes!("./helvetica/helvetica-light.ttf");

pub static INTER_HEADING: &[u8] = include_bytes!("./inter/Inter_28pt-Bold.ttf");
pub static INTER_REGULAR: &[u8] = include_bytes!("./inter/Inter_24pt-Regular.ttf");
pub static INTER_SMALL: &[u8] = include_bytes!("./inter/Inter_18pt-Light.ttf");

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
        "recipe_title",
        FontData::from_static(RECIPE_TITLE),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name(Arc::from("recipe_title")),
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));

    // Helvetica
    let family =  egui::FontFamily::Name(Arc::from("helvetica"));
    ctx.add_font(FontInsert::new(
        "helvetica",
        FontData::from_static(HELVETICA),
        vec![InsertFontFamily {
            family: family.clone(),
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));
    ctx.add_font(FontInsert::new(
        "helvetica_bold",
        FontData::from_static(HELVETICA_BOLD),
        vec![InsertFontFamily {
            family: family.clone(),
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));

    ctx.add_font(FontInsert::new(
        "helvetica_light",
        FontData::from_static(HELVETICA_LIGHT),
        vec![InsertFontFamily {
            family,
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));

    // inter
    ctx.add_font(FontInsert::new(
        "inter_heading",
        FontData::from_static(INTER_HEADING),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name(Arc::from("inter_heading")),
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));
    ctx.add_font(FontInsert::new(
        "inter_body",
        FontData::from_static(INTER_REGULAR),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name(Arc::from("inter_body")),
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));

    ctx.add_font(FontInsert::new(
        "inter_small",
        FontData::from_static(HELVETICA_LIGHT),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name(Arc::from("inter_small")),
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));

    // Lucide Icons
    ctx.add_font(FontInsert::new(
        "icons",
        FontData::from_static(include_bytes!("icons/lucide.ttf")),
        vec![InsertFontFamily {
            family: egui::FontFamily::Proportional,
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));
}
