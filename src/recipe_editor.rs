use egui::{vec2, DragValue, Response, RichText, Sense};
use uuid::Uuid;

use crate::{models::Recipe, util::DEFAULT_PADDING};

pub struct Editor {
    id: String,
}

impl Editor {
    pub fn new() -> Self {
        let id = Uuid::new_v4().to_string();
        Self { id }
    }

    fn panel_name(&self, name: &str) -> String {
        format!("{}_{}", self.id, name)
    }

    pub fn ui(&self, ui: &mut egui::Ui, recipe: &mut Recipe) -> Response {
        let mut response: Response = ui.interact(ui.clip_rect(), ui.id(), Sense::hover());
        // Note that the order we add the panels is very important!
        //sidepanel
        egui::SidePanel::left("edit_left_panel")
                .resizable(true)
                .default_width(150.0)
                .width_range(80.0..=400.0)
                .show_inside(ui, |ui| {
                ui.vertical_centered(|ui|  {
                    ui.heading("Ingredient list");
                    ui.label(RichText::new("One ingredient with measurement per line. For example:\n1 tbsp olive oil").italics());
                    response = ui.text_edit_multiline(&mut recipe.ingredients);
                });
            });

        egui::SidePanel::right("edit_right_panel")
            // .resizable(true)
            .default_width(220.0)
            // .width_range(80.0..=200.0)
            .show_inside(ui, |ui| {
                recipe.macros.ui(ui, recipe.servings, "Amount per serving");
            });

        egui::TopBottomPanel::top("edit_top_panel")
            .resizable(true)
            .min_height(32.0)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Recipe Title");
                        ui.label(
                            RichText::new(
                                "Be creative but also predictable when naming your recipe...",
                            )
                            .italics(),
                        );
                        ui.add_space(DEFAULT_PADDING);
                        ui.text_edit_singleline(&mut recipe.title);
                        ui.add_space(DEFAULT_PADDING);

                        ui.heading("Image url");
                        ui.label(
                            RichText::new(
                                "On desktop applications this can be file:// for local images...",
                            )
                            .italics(),
                        );
                        ui.add_space(DEFAULT_PADDING);
                        ui.text_edit_singleline(&mut recipe.image_url);
                        ui.add_space(DEFAULT_PADDING);
                        ui.allocate_ui(vec2(100.0, 200.0), |ui| {
                            ui.image(&recipe.image_url);
                        });
                    });
                });
            });
        //topbottom
        egui::TopBottomPanel::bottom("edit_bottom_panel")
            .resizable(false)
            .min_height(0.0)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Servings");
                    ui.add_space(DEFAULT_PADDING);
                    ui.add(DragValue::new(&mut recipe.servings).range(1..=u8::MAX));
                });
            });
        //central
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.set_width(ui.max_rect().width());
            ui.set_height(ui.max_rect().height());
            ui.vertical_centered(|ui| {
                ui.heading("Cooking Instructions");
                ui.label(
                    RichText::new("You can also add links here...")
                        .small()
                        .italics(),
                );
                ui.add_space(DEFAULT_PADDING);
                ui.text_edit_multiline(&mut recipe.instructions);
                ui.add_space(DEFAULT_PADDING);
            });
        });

        response
    }
}
