use std::sync::Arc;

use egui::{
    epaint::text::{FontInsert, InsertFontFamily},
    FontData,
};

pub static GEIST_BLACK: &[u8] = include_bytes!("geist/Geist-Black.ttf");
pub static GEIST_BOLD: &[u8] = include_bytes!("geist/Geist-Bold.ttf");
pub static GEIST_EXTRA_BOLD: &[u8] = include_bytes!("geist/Geist-ExtraBold.ttf");
pub static GEIST_EXTRA_LIGHT: &[u8] = include_bytes!("geist/Geist-ExtraLight.ttf");
pub static GEIST_MEDIUM: &[u8] = include_bytes!("geist/Geist-Medium.ttf");
pub static GEIST_REGULAR: &[u8] = include_bytes!("geist/Geist-Regular.ttf");
pub static GEIST_SEMI_BOLD: &[u8] = include_bytes!("geist/Geist-SemiBold.ttf");
pub static GEIST_THIN: &[u8] = include_bytes!("geist/Geist-Thin.ttf");
pub static HANDWRITING: &[u8] = include_bytes!("./Caveat-Regular.ttf");

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
        "handwriting",
        FontData::from_static(HANDWRITING),
        vec![InsertFontFamily {
            family: egui::FontFamily::Name(Arc::from("handwriting")),
            priority: egui::epaint::text::FontPriority::Lowest,
        }],
    ));

    // Helvetica
    let family = egui::FontFamily::Name(Arc::from("helvetica"));
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
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Name(Arc::from("icons")),
                priority: egui::epaint::text::FontPriority::Highest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Highest,
            },
        ],
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lucide_is_the_primary_proportional_font() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);

        // Font additions become active at the start of the next pass.
        let mut output = ctx.run_ui(Default::default(), |_| {});
        output.textures_delta.clear();
        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.ctx().fonts_mut(|fonts| {
                let proportional = &fonts.definitions().families[&egui::FontFamily::Proportional];
                assert_eq!(proportional.first().map(String::as_str), Some("icons"));
                let icon_family = egui::FontFamily::Name(Arc::from("icons"));
                assert_eq!(
                    fonts.definitions().families[&icon_family]
                        .first()
                        .map(String::as_str),
                    Some("icons")
                );
                let font_id = egui::FontId::new(16.0, icon_family);
                for glyph in ['\u{E18D}', '\u{E3EB}', '\u{E607}'] {
                    assert!(fonts.has_glyph(&font_id, glyph));
                    let galley = fonts.layout_no_wrap(
                        glyph.to_string(),
                        font_id.clone(),
                        egui::Color32::WHITE,
                    );
                    assert_eq!(galley.rows[0].glyphs[0].chr, glyph);
                }
            });
        });
        output.textures_delta.clear();
    }
}
