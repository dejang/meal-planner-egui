use egui::{Button, Id, Image, Margin, RichText, ScrollArea};

use crate::{
    fonts::heading2,
    icon, icons,
    models::{AnalysisResponseView, Recipe},
    planner::Location,
    theme,
    util::{percentage, DEFAULT_PADDING},
};

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum EditState<T> {
    Pending(T),
    Editing(T),
    DeleteRecipeAtIndex(T),
    #[default]
    Empty,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct RecipeBrowser {
    active_recipe: usize,
    analysis_response_view: AnalysisResponseView,
    pub edit_recipe_idx: EditState<usize>,
    recipe_name_search: String,
}

impl RecipeBrowser {
    pub fn show(&mut self, ui: &mut egui::Ui, recipes: &[Recipe]) {
        let max_width = ui.max_rect().width();

        // Recipe list sidebar
        egui::SidePanel::left("recipe list")
            .show_separator_line(true)
            .default_width(percentage(max_width, 10))
            .max_width(percentage(max_width, 40))
            .resizable(true)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Recipe List").text_style(heading2()).strong());
                    ui.vertical(|ui| {
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
                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.set_width(ui.max_rect().width());
                        for (i, recipe) in recipies_in_view {
                            ui.add_space(4.0); // Top padding
                            ui.horizontal(|ui| {
                                ui.add_space(8.0); // Left padding
                                let delete_btn =
                                    Button::image(Image::from_bytes("trash", icons::DELETE));
                                if ui.add(delete_btn).clicked() {
                                    self.edit_recipe_idx = EditState::DeleteRecipeAtIndex(i);
                                }

                                // Draggable icon
                                ui.dnd_drag_source(
                                    Id::new(("browser", recipe.to_string())),
                                    Location {
                                        col: 0,
                                        row: usize::MAX,
                                        recipe_index: i,
                                    },
                                    |ui| {
                                        egui::Frame::none()
                                            .fill(ui.visuals().extreme_bg_color)
                                            .inner_margin(Margin::symmetric(4.0, 4.0))
                                            .rounding(4.0)
                                            .show(ui, |ui| {
                                                ui.label(
                                                    RichText::new(
                                                        theme::typography::icons::ICON_MOVE,
                                                    )
                                                    .text_style(icon()),
                                                )
                                                .on_hover_text("Drag me to the planner");
                                            });
                                    },
                                );

                                // Clickable title
                                let text = if i == self.active_recipe {
                                    RichText::new(&recipe.title).strong().underline()
                                } else {
                                    RichText::new(&recipe.title)
                                };
                                let label =
                                    ui.add(egui::Label::new(text).sense(egui::Sense::click()));
                                if label.double_clicked() {
                                    self.edit_recipe_idx = EditState::Pending(i);
                                } else if label.clicked() {
                                    self.active_recipe = i;
                                }
                                label
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .on_hover_text("Click to view, double-click to edit");
                            });
                            ui.add_space(10.0); // Bottom padding
                        }
                    });
                });
            });

        let default_recipe = Recipe::default();
        let recipe = recipes.get(self.active_recipe).unwrap_or(&default_recipe);

        // Main recipe view
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let panel_width = ui.available_width();
            let side_panel_width = percentage(panel_width, 15);
            let nutrients_panel_width = percentage(panel_width, 30);

            // Ingredients panel (left)
            egui::SidePanel::left("ingredients_panel")
                .show_separator_line(false)
                .resizable(false)
                .exact_width(side_panel_width)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Ingredients");
                    });
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(&recipe.ingredients);
                    });
                });

            // Nutrients panel (right)
            egui::SidePanel::right("nutrients_panel")
                .resizable(false)
                .show_separator_line(false)
                .exact_width(nutrients_panel_width)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.analysis_response_view.ui(
                            ui,
                            &recipe.macros,
                            recipe.servings,
                            "Amount per serving",
                        );
                    });
                });

            // Central content
            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::Frame::none()
                    .fill(ui.visuals().window_fill)
                    .inner_margin(10.0)
                    .rounding(5.0)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(&recipe.title);
                            ui.add_space(DEFAULT_PADDING);
                        });

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let available_width = ui.available_width();
                                let image_width = percentage(available_width, 70);
                                ui.add(
                                    egui::Image::new(&recipe.image_url)
                                        .max_size(egui::vec2(image_width, image_width)),
                                );
                                ui.add_space(DEFAULT_PADDING * 2.0);
                                ui.heading("Instructions");
                                ui.label(
                                    RichText::new(&recipe.instructions).text_style(heading2()),
                                );
                            });
                        });
                    });
            });
        });
    }
}
