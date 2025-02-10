use egui::{vec2, DragValue, Response, RichText, Sense};

use crate::{
    models::Recipe,
    util::{percentage, DEFAULT_PADDING},
};

pub struct Editor;
impl Editor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn ui(&self, ui: &mut egui::Ui, recipe: &mut Recipe) -> Response {
        let mut response: Response = ui.interact(ui.clip_rect(), ui.id(), Sense::hover());
        let max_width = ui.max_rect().width();
        let max_height = ui.max_rect().height();

        egui::SidePanel::left("edit_left_panel")
                .resizable(true)
                .default_width(percentage(max_width, 25))
                .show_inside(ui, |ui| {
                ui.vertical_centered(|ui|  {
                    ui.heading("Ingredient list");
                    ui.label(RichText::new("One ingredient with measurement per line. For example:\n1 tbsp olive oil").italics());
                    response = ui.text_edit_multiline(&mut recipe.ingredients);
                });
            });

        egui::SidePanel::right("edit_right_panel")
            .default_width(percentage(max_width, 25))
            .show_inside(ui, |ui| {
                recipe.macros.ui(ui, recipe.servings, "Amount per serving");
            });

        egui::TopBottomPanel::top("edit_top_panel")
            .resizable(true)
            .default_height(percentage(max_height, 50))
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

                        ui.heading("Image");
                        ui.horizontal(|ui| {
                            ui.label("URL or local file: ");
                            ui.text_edit_singleline(&mut recipe.image_url);
                            if ui.button("Choose file").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"])
                                    .pick_file() 
                                {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        recipe.image_url = format!("file://{}", path.display());
                                    }
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        recipe.image_url = path.display().to_string();
                                    }
                                }
                            }
                        });
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
