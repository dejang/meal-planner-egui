pub mod fonts;
pub mod typography;
pub mod widgets;
use std::collections::BTreeMap;

use egui::{Color32, CornerRadius, FontFamily, FontId, Frame, RichText, Shadow, Stroke, TextStyle};
use fonts::install_fonts;
use serde::{Deserialize, Serialize};

/// Shared colors for the warm paper and botanical green theme.
pub mod palette {
    use egui::Color32;

    pub const PAPER: Color32 = Color32::from_rgb(246, 244, 235);
    pub const SURFACE: Color32 = Color32::from_rgb(255, 254, 249);
    pub const FOREST: Color32 = Color32::from_rgb(27, 63, 49);
    pub const GREEN: Color32 = Color32::from_rgb(48, 104, 76);
    pub const SAGE: Color32 = Color32::from_rgb(232, 238, 223);
    pub const CITRUS: Color32 = Color32::from_rgb(221, 235, 155);
    pub const INK: Color32 = Color32::from_rgb(35, 51, 42);
    pub const MUTED: Color32 = Color32::from_rgb(100, 111, 95);
    pub const BORDER: Color32 = Color32::from_rgb(217, 223, 206);
    pub const TERRACOTTA: Color32 = Color32::from_rgb(161, 76, 46);
}

pub fn card_shadow() -> Shadow {
    Shadow {
        offset: [0, 5],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(18),
    }
}

/// Shared paper window treatment for every app window.
pub fn paper_window_frame(style: &egui::Style) -> Frame {
    Frame::window(style).fill(palette::PAPER)
}

/// A quiet title and close control without egui's colored title bar.
pub fn paper_window_header(ui: &mut egui::Ui, title: &str) -> bool {
    let mut close = false;
    ui.horizontal_top(|ui| {
        let close_width =
            (16.0 + 2.0 * ui.spacing().button_padding.x).max(ui.spacing().interact_size.x);
        let title_width =
            (ui.available_width() - close_width - ui.spacing().item_spacing.x).max(80.0);
        ui.allocate_ui_with_layout(
            egui::vec2(title_width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(title_width);
                ui.label(
                    RichText::new(title)
                        .font(FontId::new(28.0, FontFamily::Name("recipe_display".into())))
                        .color(palette::FOREST),
                );
            },
        );
        close = ui
            .add_sized(
                [close_width, ui.spacing().interact_size.y],
                egui::Button::new(icon(typography::icons::ICON_X).size(16.0)).frame(false),
            )
            .on_hover_text("Close")
            .clicked();
    });
    ui.add_space(12.0);
    close
}

pub fn surface_frame() -> Frame {
    Frame::new()
        .fill(palette::SURFACE)
        .stroke(Stroke::new(1.0, palette::BORDER))
        .corner_radius(16)
        .shadow(card_shadow())
}

/// A scoped treatment for the menu bar and day headers, including their popups.
pub fn on_forest(ui: &mut egui::Ui) {
    let visuals = ui.visuals_mut();
    visuals.override_text_color = None;
    visuals.weak_text_color = Some(palette::CITRUS);
    visuals.widgets.noninteractive.fg_stroke.color = palette::SURFACE;
    visuals.widgets.noninteractive.bg_stroke.color = palette::GREEN;
    visuals.widgets.inactive.fg_stroke.color = palette::SURFACE;
    visuals.widgets.inactive.bg_fill = palette::FOREST;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    for widget in [
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.bg_fill = palette::GREEN;
        widget.weak_bg_fill = palette::GREEN;
        widget.fg_stroke.color = palette::CITRUS;
        widget.bg_stroke = Stroke::new(1.0, palette::GREEN);
    }
    visuals.window_fill = palette::FOREST;
    visuals.window_stroke = Stroke::new(1.0, palette::GREEN);
}

/// Apply the given theme to a [`Context`](egui::Context).
pub fn set_theme(ctx: &egui::Context, theme: Theme) {
    install_fonts(ctx);
    ctx.set_theme(egui::Theme::Light);
    theme.visuals(ctx);
    ctx.request_repaint();
}

pub fn recipe_title() -> TextStyle {
    TextStyle::Name("recipe_title".into())
}

pub fn handwriting() -> TextStyle {
    TextStyle::Name("handwriting".into())
}

pub fn icon(glyph: impl Into<String>) -> RichText {
    RichText::new(glyph).family(FontFamily::Name("icons".into()))
}

pub fn smallish() -> TextStyle {
    TextStyle::Name("inter_small".into())
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
        let web = cfg!(target_arch = "wasm32");
        let body = if web { 14.0 } else { 17.0 };
        let small = if web { 12.0 } else { 14.0 };
        let heading = if web { 20.0 } else { 22.0 };
        let text_styles: BTreeMap<TextStyle, FontId> = [
            (
                TextStyle::Heading,
                FontId::new(heading, Name("heading".into())),
            ),
            (TextStyle::Body, FontId::new(body, Proportional)),
            (TextStyle::Monospace, FontId::new(15.0, Monospace)),
            (
                TextStyle::Button,
                FontId::new(if web { 14.0 } else { 15.0 }, Proportional),
            ),
            (TextStyle::Small, FontId::new(small, Proportional)),
            (smallish(), FontId::new(body - 1.0, Proportional)),
            (
                recipe_title(),
                FontId::new(if web { 21.0 } else { 24.0 }, Name("heading".into())),
            ),
            (
                helvetica_heading(),
                FontId::new(heading, Name("heading".into())),
            ),
            (helvetica_body(), FontId::new(body, Proportional)),
            (helvetica_small(), FontId::new(small, Proportional)),
            (
                handwriting(),
                FontId::new(if web { 23.0 } else { 32.0 }, Name("handwriting".into())),
            ),
        ]
        .into();
        ctx.all_styles_mut(move |style| {
            style.text_styles = text_styles.clone();
            style.animation_time = 0.16;
            style.spacing.button_padding = egui::vec2(12.0, 7.0);
            style.spacing.interact_size.y = 30.0;
            style.spacing.window_margin = egui::Margin::same(16);
            style.spacing.menu_margin = egui::Margin::same(8);

            let mut visuals = egui::Visuals::light();
            visuals.panel_fill = palette::PAPER;
            visuals.window_fill = palette::SURFACE;
            visuals.extreme_bg_color = palette::SURFACE;
            visuals.text_edit_bg_color = Some(palette::SURFACE);
            visuals.faint_bg_color = palette::SAGE;
            visuals.code_bg_color = palette::SAGE;
            visuals.weak_text_color = Some(palette::MUTED);
            visuals.hyperlink_color = palette::GREEN;
            visuals.warn_fg_color = palette::TERRACOTTA;
            visuals.error_fg_color = Color32::from_rgb(175, 53, 49);
            visuals.selection.bg_fill = palette::CITRUS;
            visuals.selection.stroke = Stroke::new(1.0, palette::FOREST);
            visuals.text_cursor.stroke = Stroke::new(2.0, palette::GREEN);
            visuals.window_corner_radius = CornerRadius::same(16);
            visuals.menu_corner_radius = CornerRadius::same(10);
            visuals.window_stroke = Stroke::new(1.0, palette::BORDER);
            visuals.window_shadow = Shadow {
                offset: [0, 12],
                blur: 40,
                spread: 2,
                color: Color32::from_black_alpha(35),
            };
            visuals.popup_shadow = card_shadow();
            visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

            for widget in [
                &mut visuals.widgets.noninteractive,
                &mut visuals.widgets.inactive,
                &mut visuals.widgets.hovered,
                &mut visuals.widgets.active,
                &mut visuals.widgets.open,
            ] {
                widget.corner_radius = CornerRadius::same(8);
                widget.bg_fill = palette::SURFACE;
                widget.weak_bg_fill = palette::SURFACE;
                widget.bg_stroke = Stroke::new(1.0, palette::BORDER);
                widget.fg_stroke = Stroke::new(1.0, palette::INK);
                widget.expansion = 0.0;
            }
            visuals.widgets.hovered.bg_fill = palette::SAGE;
            visuals.widgets.hovered.weak_bg_fill = palette::SAGE;
            visuals.widgets.hovered.bg_stroke.color = palette::GREEN;
            visuals.widgets.hovered.fg_stroke.color = palette::FOREST;
            for widget in [&mut visuals.widgets.active, &mut visuals.widgets.open] {
                widget.bg_fill = palette::CITRUS;
                widget.weak_bg_fill = palette::CITRUS;
                widget.bg_stroke.color = palette::GREEN;
                widget.fg_stroke.color = palette::FOREST;
            }
            style.visuals = visuals;
        });
    }
}
