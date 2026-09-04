use egui::{
    vec2, Id, Image, Layout, Margin, Pos2, RichText, ScrollArea, Sense, Stroke, TextEdit, Widget,
};
use uuid::Uuid;

use crate::{
    meal_planner::MealPlanner,
    models::{AnalysisResponseView, Recipe},
    planner::Location,
    recipe_title,
    theme::{palette, paper_window_frame, surface_frame},
    typography::icons::{ICON_CHEF_HAT, ICON_SEARCH},
    util::{hb, percentage},
};

pub struct Ingredients<'a> {
    recipe: &'a Recipe,
}
impl<'a> Ingredients<'a> {
    pub fn new(recipe: &'a Recipe) -> Self {
        Self { recipe }
    }
}

impl<'a> Widget for Ingredients<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Ingredients");
        ui.vertical(|ui| {
            for ingredient in &self.recipe.macros.ingredients {
                ui.horizontal(|ui| {
                    if ingredient.parsed.is_none() {
                        return;
                    }
                    let detail = ingredient.parsed.as_ref().unwrap().first().unwrap();
                    ui.label(&detail.food);

                    let layout = Layout::right_to_left(egui::Align::Center);
                    ui.with_layout(layout, |ui| {
                        ui.set_width(40.);
                        ui.label(format!(
                            "{} {}",
                            detail.quantity,
                            detail.measure.as_deref().unwrap_or("N/A")
                        ));
                    });
                });
            }
        })
        .response
    }
}

pub struct GalleryItemDragPreview;

impl GalleryItemDragPreview {
    fn show(ui: &mut egui::Ui, image: &Option<Image>) {
        if let Some(pos) = ui.ctx().pointer_hover_pos() {
            egui::Area::new(Id::new("drag_preview"))
                .fixed_pos(pos)
                .show(ui.ctx(), |ui| {
                    ui.set_width(100.);
                    ui.set_height(100.);

                    let frame = surface_frame().inner_margin(6);
                    frame.show(ui, |ui| {
                        if let Some(image) = &image {
                            ui.add(image.to_owned());
                        } else {
                            ui.label("Dragging");
                        }
                    });
                });
        }
    }
}

pub struct GalleryItem<'a> {
    recipe: &'a Recipe,
    size: &'a (f32, f32),
    selected: bool,
}

impl<'a> GalleryItem<'a> {
    pub fn new(size: &'a (f32, f32), recipe: &'a Recipe, selected: bool) -> Self {
        Self {
            recipe,
            size,
            selected,
        }
    }
}

impl<'a> Widget for GalleryItem<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (width, height) = &self.size;
        let height = percentage(*height, 90);
        let frame = surface_frame()
            .inner_margin(Margin::same(10))
            .outer_margin(Margin::same(10));

        let card = frame.show(ui, |ui| {
            ui.style_mut().interaction.selectable_labels = false;
            ui.set_width(*width);
            ui.set_height(height);
            let response = ui.interact(
                ui.max_rect(),
                Id::new(("recipe_card", self.recipe.id)),
                Sense::click_and_drag(),
            );

            ui.vertical_centered_justified(|ui| {
                ui.scope(|ui| {
                    let title_font = recipe_title().resolve(ui.style());
                    let title_height = ui.fonts_mut(|fonts| fonts.row_height(&title_font)) * 2.0;
                    let mut title = egui::text::LayoutJob::simple(
                        self.recipe.title.clone(),
                        title_font,
                        palette::FOREST,
                        *width,
                    );
                    title.wrap.max_rows = 2;
                    title.halign = egui::Align::Center;
                    let title = ui.fonts_mut(|fonts| fonts.layout_job(title));
                    ui.add_sized(
                        vec2(*width, title_height),
                        egui::Label::new(title).selectable(false),
                    );
                });
                ui.scope(|ui| {
                    let photo_height =
                        percentage(height, 65).min((ui.available_height() - 48.0).max(32.0));
                    recipe_photo(ui, self.recipe, vec2(*width, photo_height));
                });

                ui.add_space(10.);

                let layout = Layout::bottom_up(egui::Align::Min);
                ui.with_layout(layout, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            hb(&format!(
                                "{} kcal",
                                self.recipe.macros.calories / (self.recipe.servings as i32)
                            ))
                            .strong()
                            .color(palette::FOREST),
                        );

                        let layout = Layout::right_to_left(egui::Align::Center);
                        ui.with_layout(layout, |ui| {
                            ui.label(
                                hb(&format!("{} servings", self.recipe.servings))
                                    .color(palette::MUTED),
                            );
                        });
                    });
                    ui.separator();
                });
            });
            response
        });

        let response = card.inner.on_hover_cursor(egui::CursorIcon::Grab);
        let hover =
            ui.ctx()
                .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.16);
        let border = if self.selected {
            palette::GREEN
        } else {
            palette::BORDER.lerp_to_gamma(palette::GREEN, hover)
        };
        ui.painter().rect_stroke(
            card.response.rect.shrink(10.0),
            16,
            Stroke::new(if self.selected { 2.0 } else { 1.0 }, border),
            egui::StrokeKind::Inside,
        );
        response
    }
}

/// Fill the existing photo area without stretching the source image. A quiet
/// illustrated placeholder also keeps cards stable while remote photos load.
pub(crate) fn recipe_photo(ui: &mut egui::Ui, recipe: &Recipe, size: egui::Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let image = Image::new(&recipe.image_url);
    let texture = if recipe.image_url.is_empty() {
        None
    } else {
        image.load_for_size(ui.ctx(), size).ok()
    };
    if let Some(egui::load::TexturePoll::Ready { texture }) = texture {
        let source_aspect = texture.size.x / texture.size.y;
        let target_aspect = size.x / size.y;
        let uv_size = if source_aspect > target_aspect {
            vec2(target_aspect / source_aspect, 1.0)
        } else {
            vec2(1.0, source_aspect / target_aspect)
        };
        Image::from_texture(texture)
            .uv(egui::Rect::from_center_size(egui::pos2(0.5, 0.5), uv_size))
            .corner_radius(10)
            .paint_at(ui, rect);
    } else {
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 10, palette::SAGE);
        let center = rect.center() - vec2(0.0, 12.0);
        painter.circle_stroke(center, 42.0, Stroke::new(1.0, palette::BORDER));
        painter.circle_stroke(center, 34.0, Stroke::new(1.0, palette::BORDER));
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            ICON_CHEF_HAT,
            egui::FontId::new(32.0, egui::FontFamily::Name("icons".into())),
            palette::GREEN,
        );
        painter.text(
            center + vec2(0.0, 58.0),
            egui::Align2::CENTER_CENTER,
            "A little kitchen inspiration",
            egui::TextStyle::Small.resolve(ui.style()),
            palette::MUTED,
        );
    }
}

#[derive(Debug, Default)]
pub struct RecipeGallery {
    search_query: String,
    current_recipe: Option<Uuid>,
    nutrients_view: AnalysisResponseView,
    item_dragging: bool,
    drag_image: Option<Image<'static>>,
    show_details: bool,
}

impl RecipeGallery {
    fn detail_panel(&mut self, ui: &mut egui::Ui, recipe: &Recipe) -> bool {
        let window_width = 500.;
        let mut edit_clicked = false;

        let frame = paper_window_frame(ui.style());

        egui::Window::new("Recipe")
            .title_bar(false)
            .collapsible(true)
            .current_pos(Pos2 {
                x: ui.max_rect().width(),
                y: 0.0,
            })
            .fade_in(true)
            .frame(frame)
            .min_width(window_width)
            .max_width(window_width)
            .collapsible(false)
            .open(&mut self.show_details)
            .show(ui.ctx(), |ui| {
                ui.set_height(ui.ctx().content_rect().height());
                ui.set_width(500.);

                egui::CentralPanel::default().show(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        if self.current_recipe.is_none() {
                            ui.label("No Recipe to display...");
                            return;
                        }

                        ui.horizontal(|ui| {
                            ui.add_sized(
                                vec2(percentage(ui.available_width(), 90), 40.),
                                |ui: &mut egui::Ui| {
                                    ui.horizontal_wrapped(|ui| ui.heading(&recipe.title))
                                        .response
                                },
                            );
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Edit").clicked() {
                                    edit_clicked = true;
                                }
                            });
                        });

                        ui.add_space(10.);

                        ui.separator();
                        ui.add(Ingredients::new(recipe));
                        ui.separator();
                        ui.heading("Cooking Instructions");
                        let _ = &recipe.instructions.split("\n").for_each(|line| {
                            let line = line.trim();
                            if !line.is_empty() {
                                ui.label(line);
                            }
                            ui.add_space(5.);
                        });

                        ui.separator();
                        self.nutrients_view.ui(
                            ui,
                            &recipe.macros,
                            recipe.servings,
                            "Calories per portion",
                        );
                    });
                });
            });

        if self.show_details {
            ui.input(|i| {
                if i.key_pressed(egui::Key::Escape) {
                    self.show_details = false;
                    self.current_recipe = None;
                    self.nutrients_view = AnalysisResponseView;
                }
            });
        }
        edit_clicked
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, meal_planner: &mut MealPlanner) -> Option<Uuid> {
        let mut edit_recipe = None;
        ui.input(|input_state| {
            if input_state.key_pressed(egui::Key::Delete) && self.current_recipe.is_some() {
                meal_planner.remove_recipe(&self.current_recipe.unwrap());
                self.current_recipe = None;
            }
        });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical(|ui| {
                // search area
                ui.scope(|ui| {
                    ui.set_height(38.);
                    ui.set_width(ui.available_width());
                    ui.centered_and_justified(|ui| {
                        ui.add(
                            TextEdit::singleline(&mut self.search_query)
                                .hint_text(
                                    RichText::new(format!(
                                        "{ICON_SEARCH}   Find something delicious…"
                                    ))
                                    .color(palette::MUTED),
                                )
                                .margin(egui::Margin::symmetric(16, 9)),
                        );
                    });
                });

                ui.add_space(10.);
                ui.separator();
                ui.add_space(10.);

                let item_height = percentage(ui.available_height(), 100);
                let item_width = 400.;
                ScrollArea::horizontal().show(ui, |ui| {
                    let layout = Layout::left_to_right(egui::Align::Center);
                    ui.with_layout(layout, |ui| {
                        let recipes = if self.search_query.is_empty() {
                            meal_planner.get_recipes()
                        } else {
                            meal_planner.search_recipe(&self.search_query)
                        };
                        let size = (item_width, item_height);

                        for recipe in recipes {
                            let payload = Location {
                                col: 0,
                                row: usize::MAX,
                                recipe_id: recipe.id,
                            };

                            let is_selected = match self.current_recipe {
                                Some(id) => recipe.id == id,
                                None => false,
                            };
                            let item_response =
                                ui.add(GalleryItem::new(&size, recipe, is_selected));
                            if item_response.clicked() {
                                self.current_recipe.replace(recipe.id);
                                self.show_details = true;
                            }

                            if item_response.drag_started() {
                                self.item_dragging = true;
                                self.drag_image
                                    .replace(Image::from_uri(recipe.image_url.clone()));
                                item_response.dnd_set_drag_payload(payload);
                            }

                            if item_response.drag_stopped() {
                                self.drag_image = None;
                                self.item_dragging = false;
                            }

                            if self.item_dragging {
                                GalleryItemDragPreview::show(ui, &self.drag_image);
                            }
                        }
                    });
                });
            });

            if let Some(id) = self.current_recipe {
                let recipe = meal_planner.get_recipe_by_id(&id).unwrap();
                let edit_clicked = self.detail_panel(ui, recipe);
                if edit_clicked {
                    self.show_details = false;
                    edit_recipe.replace(id);
                }
            }
        });

        edit_recipe
    }
}
