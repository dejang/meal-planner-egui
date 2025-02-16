pub mod fonts;
pub mod typography;
use std::collections::BTreeMap;

use egui::{Color32, FontFamily, FontId, TextStyle};
use fonts::install_fonts;
use serde::{Deserialize, Serialize};

/// Apply the given theme to a [`Context`](egui::Context).
pub fn set_theme(ctx: &egui::Context, theme: Theme) {
    install_fonts(ctx);
    theme.visuals(ctx);
    ctx.request_repaint();
}

pub fn icon() -> TextStyle {
    TextStyle::Name("icons".into())
}

pub fn recipe_title() -> TextStyle {
    TextStyle::Name("recipe_title".into())
}

// Helvetica
pub fn helvetica_heading() -> TextStyle {
    TextStyle::Name("helvetica_bold".into())
}

pub fn helvetica_body() -> TextStyle {
    TextStyle::Name("helvetica".into())
}

pub fn helvetica_small() -> TextStyle {
    TextStyle::Name("helvetica_light".into())
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Theme {
    pub primary: Color32,
}

impl Theme {
    pub fn visuals(&self, ctx: &egui::Context) {
        use FontFamily::{Monospace, Name, Proportional};

        let text_styles: BTreeMap<TextStyle, FontId> = [
            (
                TextStyle::Heading,
                FontId::new(24.0, Name("inter_heading".into())),
            ),
            (
                TextStyle::Body,
                FontId::new(18.0, Name("inter_body".into())),
            ),
            (TextStyle::Monospace, FontId::new(15.0, Monospace)),
            (TextStyle::Button, FontId::new(15.0, Proportional)),
            (
                TextStyle::Small,
                FontId::new(14.0, Name("inter_small".into())),
            ),
            (icon(), FontId::new(16.0, Name("icons".into()))),
            (
                recipe_title(),
                FontId::new(28., Name("inter_heading".into())),
            ),
            (
                helvetica_heading(),
                FontId::new(24., Name("helvetica".into())),
            ),
            (helvetica_body(), FontId::new(18., Name("helvetica".into()))),
            (
                helvetica_small(),
                FontId::new(14., Name("helvetica".into())),
            ),
        ]
        .into();
        ctx.all_styles_mut(move |style| style.text_styles = text_styles.clone());
    }
}
