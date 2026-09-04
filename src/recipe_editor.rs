use egui::{
    vec2, Align, Button, CentralPanel, Color32, DragValue, FontFamily, FontId, Frame, Layout,
    Margin, Panel, Rect, Response, RichText, ScrollArea, Sense, Stroke, TextEdit, TextStyle, Ui,
    UiBuilder,
};

use crate::{
    icon,
    models::{AnalysisResponseView, Recipe},
    recipe_gallery::recipe_photo,
    theme::palette,
    typography::icons::{ICON_CHECK, ICON_X},
    widgets::notebook::Notebook,
};

#[derive(Default)]
pub struct EditorResponse {
    pub ingredients: Option<Response>,
    pub close: bool,
}

pub struct Editor;

impl Editor {
    pub fn new() -> Self {
        Self
    }

    pub fn ui(&mut self, ui: &mut Ui, recipe: &mut Recipe) -> EditorResponse {
        let mut result = EditorResponse::default();
        page_style(ui);

        Panel::top("recipe_editor_header")
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(palette::PAPER)
                    .inner_margin(Margin::symmetric(24, 8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(caption("RECIPE BOOK  /  EDIT RECIPE"));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        result.close |= ui
                            .add(Button::new(icon(ICON_X).size(16.0)).frame(false))
                            .on_hover_text("Close editor")
                            .clicked();
                    });
                });
            });

        Panel::bottom("recipe_editor_footer")
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(palette::PAPER)
                    .inner_margin(Margin::symmetric(24, 10)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(caption("Changes apply as you edit."));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        result.close |= ui
                            .add(
                                Button::new((
                                    icon(ICON_CHECK).color(palette::SURFACE),
                                    RichText::new("Done")
                                        .font(FontId::proportional(15.0))
                                        .color(palette::SURFACE),
                                ))
                                .fill(palette::FOREST)
                                .stroke(Stroke::NONE)
                                .corner_radius(8)
                                .min_size(vec2(100.0, 34.0)),
                            )
                            .on_hover_text("Close the editor with your changes applied")
                            .clicked();
                    });
                });
            });

        let page_margin = if ui.available_width() < 600.0 { 16 } else { 32 };
        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(palette::PAPER)
                    .inner_margin(Margin::symmetric(page_margin, 12)),
            )
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .id_salt((recipe.id, "cookbook_page"))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        Self::cover(ui, recipe);
                        ui.add_space(16.0);
                        Self::image_field(ui, recipe);
                        ui.add_space(20.0);

                        Frame::new()
                            .fill(palette::SURFACE)
                            .inner_margin(24)
                            .show(ui, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(caption("SERVINGS"));
                                    ui.add(
                                        DragValue::new(&mut recipe.servings)
                                            .range(1..=u32::MAX)
                                            .speed(0.1),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(caption(&format!(
                                            "{} KCAL / SERVING",
                                            i64::from(recipe.macros.calories)
                                                / i64::from(recipe.servings.max(1))
                                        )));
                                    });
                                });
                                rule(ui, palette::INK);
                                ui.add_space(16.0);

                                if ui.available_width() >= 580.0 {
                                    let width = ui.available_width();
                                    let ingredient_width = (width - 32.0) * 0.38;
                                    ui.horizontal_top(|ui| {
                                        ui.allocate_ui_with_layout(
                                            vec2(ingredient_width, 0.0),
                                            Layout::top_down(Align::Min),
                                            |ui| {
                                                result.ingredients =
                                                    Some(Self::ingredients(ui, recipe));
                                            },
                                        );
                                        ui.add_space(24.0);
                                        ui.allocate_ui_with_layout(
                                            vec2((width - ingredient_width - 32.0).max(100.0), 0.0),
                                            Layout::top_down(Align::Min),
                                            |ui| {
                                                Self::instructions(ui, recipe);
                                            },
                                        );
                                    });
                                } else {
                                    result.ingredients = Some(Self::ingredients(ui, recipe));
                                    ui.add_space(24.0);
                                    Self::instructions(ui, recipe);
                                }

                                ui.add_space(24.0);
                                rule(ui, palette::BORDER);
                                egui::CollapsingHeader::new(serif("NUTRITION", 23.0))
                                    .id_salt((recipe.id, "cookbook_nutrition"))
                                    .show_unindented(ui, |ui| {
                                        // Keep the existing full analysis available on the page.
                                        ui.set_max_width(480.0);
                                        AnalysisResponseView.ui(
                                            ui,
                                            &recipe.macros,
                                            recipe.servings,
                                            "Amount per serving",
                                        );
                                        ui.label(caption(
                                            "Refreshes when you leave the Ingredients field.",
                                        ));
                                    });
                            });
                        ui.add_space(8.0);
                    });
            });
        result
    }

    fn cover(ui: &mut Ui, recipe: &mut Recipe) {
        let width = ui.available_width();
        let compact = width < 580.0;
        let photo_height = (width * 0.39).clamp(220.0, 410.0);
        let photo_width = if compact { width * 0.90 } else { width * 0.60 };
        let title_width = if compact { width * 0.94 } else { width * 0.65 };
        let title_size = if compact {
            30.0
        } else {
            (width * 0.045).clamp(34.0, 46.0)
        };
        let title_font = font("recipe_display", title_size);
        let title_text_width = (title_width - 48.0).max(100.0);
        let title_height = ui.fonts_mut(|fonts| {
            fonts
                .layout(
                    recipe.title.clone(),
                    title_font.clone(),
                    palette::INK,
                    title_text_width,
                )
                .size()
                .y
                .max(fonts.row_height(&title_font))
        });
        let plate_height = title_height + 74.0;
        let plate_top = photo_height * 0.44;
        let height = photo_height.max(plate_top + plate_height);
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        let photo_rect = Rect::from_min_size(rect.min, vec2(photo_width, photo_height));
        let mut photo_ui = ui.new_child(UiBuilder::new().max_rect(photo_rect));
        recipe_photo(&mut photo_ui, recipe, photo_rect.size());

        let title_rect = Rect::from_min_size(
            rect.min + vec2(width - title_width, plate_top),
            vec2(title_width, plate_height),
        );
        // The cover's full height is already reserved. Independently positioned
        // children must not move the next field up to the shorter title plaque.
        let mut title_ui = ui.new_child(UiBuilder::new().max_rect(title_rect));
        Frame::new()
            .fill(palette::SURFACE)
            .inner_margin(24)
            .show(&mut title_ui, |ui| {
                ui.set_width(title_text_width);
                ui.label(caption("RECIPE TITLE"));
                ui.add(
                    TextEdit::multiline(&mut recipe.title)
                        .id_salt((recipe.id, "title"))
                        .font(title_font)
                        .desired_width(f32::INFINITY)
                        .desired_rows(1)
                        .margin(Margin::ZERO)
                        .hint_text(serif("Name your recipe", title_size))
                        .lock_focus(false),
                );
            });
    }

    fn image_field(ui: &mut Ui, recipe: &mut Recipe) {
        let control_height = 36.0;
        let file_button_width = 114.0;
        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), control_height),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.label(caption("IMAGE"));
                let file_button_space = if cfg!(target_arch = "wasm32") {
                    0.0
                } else {
                    file_button_width + ui.spacing().item_spacing.x
                };
                let url_width = (ui.available_width() - file_button_space).max(40.0);
                ui.add_sized(
                    [url_width, control_height],
                    TextEdit::singleline(&mut recipe.image_url)
                        .id_salt((recipe.id, "image"))
                        .font(FontId::proportional(13.0))
                        .vertical_align(Align::Center)
                        .margin(Margin::symmetric(8, 7))
                        .hint_text("Paste an image URL"),
                );
                #[cfg(not(target_arch = "wasm32"))]
                if ui
                    .add_sized(
                        [file_button_width, control_height],
                        Button::new(RichText::new("Choose file").font(FontId::proportional(13.0))),
                    )
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                        .pick_file()
                    {
                        recipe.image_url = format!("file://{}", path.display());
                    }
                }
            },
        );
    }

    fn ingredients(ui: &mut Ui, recipe: &mut Recipe) -> Response {
        ui.label(serif("INGREDIENTS", 27.0));
        ui.label(caption("One ingredient per line, with amount & unit."));
        ui.add_space(12.0);
        Notebook::ui(
            ui,
            &mut recipe.ingredients,
            (recipe.id, "ingredients"),
            7,
            "1 tablespoon olive oil\n1 onion, diced\n2 cloves garlic",
        )
    }

    fn instructions(ui: &mut Ui, recipe: &mut Recipe) {
        ui.label(serif("DIRECTIONS", 27.0));
        ui.label(caption(
            "Write or paste the instructions, one step per line.",
        ));
        ui.add_space(12.0);
        Notebook::ui(
            ui,
            &mut recipe.instructions,
            (recipe.id, "instructions"),
            7,
            "Add your cooking instructions here…",
        );
    }
}

fn font(family: &str, size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(family.into()))
}

fn serif(text: &str, size: f32) -> RichText {
    RichText::new(text)
        .font(font("recipe_display", size))
        .color(palette::INK)
}

fn caption(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(12.0))
        .color(palette::MUTED)
}

fn rule(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 12.0), Sense::hover());
    ui.painter()
        .hline(rect.x_range(), rect.center().y, Stroke::new(0.75, color));
}

fn page_style(ui: &mut Ui) {
    let style = ui.style_mut();
    style.spacing.item_spacing = vec2(8.0, 8.0);
    style
        .text_styles
        .insert(TextStyle::Body, font("recipe_text", 19.0));
    // Keep the app's shared colors and interaction states. Quiet resting borders
    // let editable text read like a recipe page; focus and hover reveal the fields.
    style.visuals.resize_corner_size = 18.0;
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
}
