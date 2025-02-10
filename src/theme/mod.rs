pub mod fonts;
pub mod typography;
use std::collections::BTreeMap;

use egui::{Color32, FontFamily, FontId, TextStyle};
use fonts::{heading2, heading3, install_fonts};
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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Theme {
    pub primary: Color32,
}

impl Theme {
    pub fn visuals(&self, ctx: &egui::Context) {
        use FontFamily::{Monospace, Proportional, Name};

        let text_styles: BTreeMap<TextStyle, FontId> = [
            (TextStyle::Heading, FontId::new(18.0, Proportional)),
            (heading2(), FontId::new(16.0, Proportional)),
            (heading3(), FontId::new(14.0, Proportional)),
            (TextStyle::Body, FontId::new(13.0, Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, Monospace)),
            (TextStyle::Button, FontId::new(10.0, Proportional)),
            (TextStyle::Small, FontId::new(6.0, Proportional)),
            (icon(), FontId::new(12.0, Name("icons".into()))),
        ]
        .into();
        ctx.all_styles_mut(move |style| style.text_styles = text_styles.clone());
    }
}
