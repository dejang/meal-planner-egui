use egui::{
    vec2, Color32, Frame, Id, Image, Layout, Margin, Pos2, RichText, Rounding, ScrollArea, Sense,
    Shadow, TextEdit, Widget,
};

use crate::{
    meal_planner::MealPlanner,
    models::{AnalysisResponseView, Recipe},
    planner::Location,
    recipe_title,
    util::{hb, percentage},
};

pub struct Alergens<'a> {
    recipe: &'a Recipe,
}

impl<'a> Alergens<'a> {
    pub fn new(recipe: &'a Recipe) -> Self {
        Self { recipe }
    }
}

impl<'a> Widget for Alergens<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(format!("{:?}", &self.recipe.macros.healthLabels))
    }
}

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
                    let detail = ingredient.parsed.as_ref().unwrap().get(0).unwrap();
                    ui.label(&detail.food);

                    let layout = Layout::right_to_left(egui::Align::Center);
                    ui.with_layout(layout, |ui| {
                        ui.set_width(40.);
                        ui.label(format!(
                            "{} {}",
                            &detail.quantity,
                            &detail.measure.as_ref().unwrap_or(&"N/A".to_string())
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

                    let frame = Frame::default().fill(ui.visuals().panel_fill);
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
}

impl<'a> GalleryItem<'a> {
    pub fn new(size: &'a (f32, f32), recipe: &'a Recipe) -> Self {
        Self { recipe, size }
    }
}

impl<'a> Widget for GalleryItem<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (width, height) = &self.size;
        let height = percentage(*height, 90);
        let frame = egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .shadow(Shadow {
                offset: vec2(0.0, 0.0),
                blur: 10.0,
                spread: 10.,
                color: Color32::from_gray(240),
            })
            .inner_margin(Margin::same(10.0))
            .outer_margin(Margin {
                left: 10.0,
                right: 10.0,
                top: 10.0,
                bottom: 10.0,
            });
        let response = frame
            .show(ui, |ui| {
                // ui.set_height(height);
                ui.set_width(*width);
                ui.set_height(height);
                let response = ui.interact(
                    ui.max_rect(),
                    Id::new(&self.recipe.title),
                    Sense::click_and_drag(),
                );

                ui.vertical_centered_justified(|ui| {
                    ui.scope(|ui| {
                        ui.label(RichText::new(&self.recipe.title).text_style(recipe_title()));
                    });
                    ui.scope(|ui| {
                        let image =
                            Image::new(&self.recipe.image_url).rounding(Rounding::same(10.))
                            .max_height(percentage(height, 65))
                            .maintain_aspect_ratio(true);

                        ui.add(image);
                    });

                    ui.add_space(10.);

                    let layout = Layout::bottom_up(egui::Align::Min);
                    ui.with_layout(layout, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(hb(&format!(
                                "Calories: {}",
                                &self.recipe.macros.calories / (self.recipe.servings as i32)
                            )));
    
                            let layout = Layout::right_to_left(egui::Align::Center);
                            ui.with_layout(layout, |ui| {
                                ui.label(hb(&format!("Servings: {}", &self.recipe.servings)));
                            });    
                        });
                        ui.separator();
                    });

                    // ui.add(Alergens::new(self.recipe));
                });
                response
            })
            .inner;

        response
    }
}

#[derive(Debug, Default)]
pub struct RecipeGallery {
    search_query: String,
    current_recipe: Option<usize>,
    nutrients_view: AnalysisResponseView,
    item_dragging: bool,
    drag_image: Option<Image<'static>>,
}

impl RecipeGallery {
    fn detail_panel(&mut self, ui: &mut egui::Ui, recipe: &Recipe) {
        let window_width = 500.;

        let frame = Frame::default()
            .fill(ui.visuals().panel_fill)
            .rounding(Rounding::same(10.))
            .shadow(Shadow {
                offset: vec2(-2., 0.0),
                blur: 20.,
                spread: 5.,
                color: Color32::from_gray(200),
            })
            .inner_margin(Margin::same(10.));

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
            .open(&mut self.current_recipe.is_some())
            .show(ui.ctx(), |ui| {
                ui.set_height(ui.ctx().screen_rect().height());
                ui.set_width(500.);

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        if let None = self.current_recipe {
                            ui.label("No Recipe to display...");
                            return;
                        }

                        ui.heading(&recipe.title);
                        ui.add_space(10.);

                        ui.separator();
                        ui.add(Ingredients::new(recipe));
                        ui.separator();
                        ui.heading("Cooking Instructions");
                        ui.label(&recipe.instructions);

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

        ui.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.current_recipe = None;
                self.nutrients_view = AnalysisResponseView::default();
            }
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, meal_planner: &MealPlanner) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                // search area
                ui.scope(|ui| {
                    ui.set_height(24.);
                    ui.set_width(ui.available_width());
                    ui.centered_and_justified(|ui| {
                        ui.add(
                            TextEdit::singleline(&mut self.search_query)
                                .hint_text("Search recipes..."),
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
                            &meal_planner.recipies
                        } else {
                            &meal_planner.search_recipe(&self.search_query)
                        };
                        let size = (item_width, item_height);
                        for (i, recipe) in recipes.iter().enumerate() {
                            let payload = Location {
                                col: 0,
                                row: usize::MAX,
                                recipe_index: i,
                            };

                            let item_response = ui.add(GalleryItem::new(&size, recipe));
                            if item_response.clicked() {
                                self.current_recipe.replace(i);
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

            if let Some(idx) = self.current_recipe {
                let recipe = meal_planner.recipies.get(idx).unwrap();
                self.detail_panel(ui, recipe);
            }
        });
    }
}
