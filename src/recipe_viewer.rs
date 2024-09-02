use std::usize;

use egui::{Button, Id, Image, ScrollArea};

use crate::{
    icons,
    models::{AnalysisResponseView, Recipe},
    planner::Location,
    util::{percentage, DEFAULT_PADDING},
};

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum EditState<T> {
    PENDING(T),
    EDITING(T),
    DELETE_RECIPE_AT_INDEX(T),
    #[default]
    EMPTY,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct RecipeBrowser {
    active_recipe: usize,
    analysis_response_view: AnalysisResponseView,
    pub edit_recipe_idx: EditState<usize>,
    recipe_name_search: String,
}

impl RecipeBrowser {
    pub fn show(&mut self, ui: &mut egui::Ui, recipes: &Vec<Recipe>) {
        let max_width = ui.max_rect().width();
        egui::SidePanel::left("recipe list")
            .show_separator_line(false)
            .default_width(percentage(max_width, 25))
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Recipe List");
                    ui.vertical_centered_justified(|ui| {
                        ui.text_edit_singleline(&mut self.recipe_name_search);
                    });
                    ui.add_space(DEFAULT_PADDING);
                });

                let recipies_in_view = recipes
                    .iter()
                    .enumerate()
                    .filter(|(_i, r)| {
                        r.title
                            .to_lowercase()
                            .contains(&self.recipe_name_search.to_lowercase())
                    })
                    .collect::<Vec<(usize, &Recipe)>>();
                ScrollArea::vertical().show(ui, |ui| {
                    for (i, recipe) in recipies_in_view {
                        ui.horizontal(|ui| {
                            let view_btn = Button::image(Image::from_bytes("view", icons::VIEW));
                            if ui.add(view_btn).clicked() {
                                self.active_recipe = i;
                            }

                            let edit_btn = Button::image(Image::from_bytes("edit", icons::EDIT));
                            if ui.add(edit_btn).clicked() {
                                self.edit_recipe_idx = EditState::PENDING(i);
                            }

                            let delete_btn =
                                Button::image(Image::from_bytes("trash", icons::DELETE));
                            if ui.add(delete_btn).clicked() {
                                self.edit_recipe_idx = EditState::DELETE_RECIPE_AT_INDEX(i);
                            }

                            ui.add_space(DEFAULT_PADDING);

                            let _response = ui
                                .dnd_drag_source(
                                    Id::new(("browser", recipe.to_string())),
                                    Location {
                                        col: 0,
                                        row: usize::MAX,
                                        recipe_index: i,
                                    },
                                    |ui| {
                                        ui.label(&recipe.title);
                                    },
                                )
                                .response;
                        });
                    }
                });
            });

        let default_recipe = Recipe::default();
        let recipe = recipes
            .get(self.active_recipe)
            .or(Some(&default_recipe))
            .unwrap();

        egui::SidePanel::right("view recipe")
            .resizable(true)
            .default_width(percentage(max_width, 75))
            .show_inside(ui, |ui| {
                let max_width = ui.max_rect().width();
                let max_height = ui.max_rect().height();

                egui::SidePanel::left("view_left_panel")
                    .resizable(true)
                    .default_width(percentage(max_width, 25))
                    .show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Ingredients");
                        });
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(&recipe.ingredients);
                        });
                    });

                egui::SidePanel::right("view_right_panel")
                    .resizable(true)
                    .default_width(percentage(max_width, 25))
                    .show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                self.analysis_response_view.ui(
                                    ui,
                                    &recipe.macros,
                                    recipe.servings,
                                    "Amount per serving",
                                );
                            });
                        });
                    });

                egui::TopBottomPanel::top("view_top_panel")
                    .resizable(true)
                    .default_height(percentage(max_height, 50))
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading(&recipe.title);
                                ui.add_space(DEFAULT_PADDING);
                                ui.image(&recipe.image_url);
                            });
                        });
                    });

                egui::TopBottomPanel::bottom("view_bottom_panel")
                    .resizable(false)
                    .min_height(0.0)
                    .show_inside(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(format!("Servings: {}", recipe.servings));
                        });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.set_width(ui.max_rect().width());
                    ui.set_height(ui.max_rect().height());

                    ui.vertical_centered(|ui| {
                        ui.heading("Cooking Instructions");
                    });
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(&recipe.instructions);
                    });
                });
            });
    }
}
