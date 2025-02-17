use egui::{
    CentralPanel, DragValue, Frame, Margin, Response, RichText, ScrollArea, SidePanel, Stroke, TextEdit
};

use crate::{
    models::{AnalysisResponseView, Recipe},
    util::DEFAULT_PADDING,
    widgets::notebook::Notebook,
};

pub struct Editor;
impl Editor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, recipe: &mut Recipe) -> Option<Response> {
        Frame::group(ui.style())
            .inner_margin(Margin::same(DEFAULT_PADDING))
            .stroke(Stroke::NONE)
            .show(ui, |ui| {
                SidePanel::right("nutrients").show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("Servings: "));
                        ui.add(DragValue::new(&mut recipe.servings));
                    });        
                    
                    AnalysisResponseView::default().ui(
                        ui,
                        &recipe.macros,
                        recipe.servings,
                        "Amount per serving",
                    );
                });
                CentralPanel::default()
                    .show_inside(ui, |ui| {
                        ScrollArea::vertical()
                            .show(ui, |ui| {
                                // Recipe Title
                                ui.horizontal(|ui| {
                                    ui.heading("Recipe Title");
                                    ui.text_edit_singleline(&mut recipe.title);
                                });
                                ui.add_space(DEFAULT_PADDING);

                                // Recipe Image
                                ui.horizontal(|ui| {
                                    ui.heading("Recipe Image");
                                    #[cfg(not(target_arch = "wasm32"))]
                                    ui.label("URL or local file: ");
                                    ui.add(TextEdit::singleline(&mut recipe.image_url));
                                    #[cfg(not(target_arch = "wasm32"))]
                                    if ui.button("Choose file").clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter(
                                                "Images",
                                                &["png", "jpg", "jpeg", "gif", "webp"],
                                            )
                                            .pick_file()
                                        {
                                            recipe.image_url = format!("file://{}", path.display());
                                        }
                                    }
                                });
                                ui.add_space(DEFAULT_PADDING);

                                // Ingredients
                                let response = ui.vertical(|ui| {
                                    Collapsible::new("ingredients_collapsible", "Ingredients")
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new("One ingredient per line in this format: quantity measurement ingredient.\nFor example: 1 teaspoon olive oil")
                                                    .text_style(egui::TextStyle::Small),
                                            );
                                            Notebook::ui(ui, &mut recipe.ingredients)
                                        }
                                    )
                                }).inner;
                                ui.add_space(DEFAULT_PADDING);

                                // Instructions
                                ui.vertical(|ui| {
                                    Collapsible::new("instructions_collapsible", "Instructions")
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new("Add each step on a new line")
                                                .text_style(egui::TextStyle::Small),
                                        );
                                        Notebook::ui(ui, &mut recipe.instructions)
                                    });
                                });
                                response.1
                            }).inner

                    })
                    .inner
            })
            .inner
    }
}

/// A simple collapsible widget that manages its own "collapsed" state
/// without requiring the parent to hold it.
pub struct Collapsible {
    /// Some unique string for generating an internal Egui ID
    id_source: String,
    /// The main label/header of the collapsible
    label: String,
}

impl Collapsible {
    /// Create a new collapsible widget.
    ///
    /// `id_source` must be unique for each collapsible in the UI.
    pub fn new(id_source: impl ToString, label: impl ToString) -> Self {
        Self {
            id_source: id_source.to_string(),
            label: label.to_string(),
        }
    }

    /// Render the collapsible widget. The `add_contents` closure will only be
    /// called (and its UI drawn) if the collapsible is not collapsed.
    ///
    /// Returns a tuple `(header_response, inner_response)`.
    /// `header_response` is the `Response` from clicking on the header row;
    /// `inner_response` is whatever your `add_contents` closure returns.
    pub fn show<R>(
        self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> (Response, Option<R>) {
        let collapsible_id = ui.make_persistent_id(self.id_source);
        let mut is_collapsed = ui
            .data_mut(|data| data.get_persisted::<bool>(collapsible_id))
            .unwrap_or(false);

        // Draw header row with arrow and label
        let header_response = ui
            .horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.set_height(30.);
                    ui.heading(&self.label);
                    let arrow = if is_collapsed { "▶" } else { "▼" };
                    if ui.selectable_label(false, arrow).clicked() {
                        is_collapsed = !is_collapsed;
                        ui.data_mut(|data| data.insert_persisted(collapsible_id, is_collapsed));
                    }
                })
            })
            .response;

        // Draw contents only if not collapsed
        let inner_response = if !is_collapsed {
            Some(add_contents(ui))
        } else {
            None
        };

        (header_response, inner_response)
    }
}
