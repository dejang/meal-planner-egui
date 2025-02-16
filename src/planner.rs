use egui::*;

use crate::{
    meal_planner::MealPlanner,
    models::{AnalysisResponseView, Recipe},
    typography::icons::{ICON_CLIPBOARD_PASTE, ICON_TRASH_2},
    util::hb,
};

/// What is being dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub col: usize,
    pub row: usize,
    pub recipe_index: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Planner {
    search_term: String,
    collapsible_nutrients: Vec<AnalysisResponseView>,
}

impl Default for Planner {
    fn default() -> Self {
        Self {
            search_term: String::new(),
            collapsible_nutrients: (0..7).map(|_| AnalysisResponseView::default()).collect(),
        }
    }
}

impl Planner {
    pub fn ui(&mut self, ui: &mut egui::Ui, meal_planner: &mut MealPlanner) {
        let delete_zone_frame = Frame::default().inner_margin(4.0);
        let (_x, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(delete_zone_frame, |ui| {
            ui.set_width(ui.max_rect().width());
            ui.label("Start by searching for a recipe.");
            ui.label("Drag the recipe you want in the desired column.");
            ui.label("Total macros is per 1 serving, drag the same recipe multiple times for multiple servings.");
        });

        if let Some(dragged_payload) = dropped_payload {
            meal_planner.daily_plan[dragged_payload.col].remove(dragged_payload.row);
        }

        // If there is a drop, store the location of the item being dragged, and the destination for the drop.
        let mut from = None;
        let mut to = None;
        ui.columns(meal_planner.daily_plan.len(), |uis| {
            for (col_idx, column) in meal_planner.daily_plan.clone().into_iter().enumerate() {
                let ui = &mut uis[col_idx];
                ui.horizontal(|ui| {
                    ui.heading(format!("Day {}", col_idx + 1));
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        let clear_btn = Button::new(ICON_TRASH_2);
                        let tooltip_ui = |ui: &mut Ui| {
                            ui.label("Clear meals");
                        };
                        if ui.add(clear_btn).on_hover_ui(tooltip_ui).clicked() {
                            meal_planner.daily_plan.get_mut(col_idx).unwrap().clear();
                        };

                        if col_idx > 0 {
                            let duplicate_btn = Button::new(ICON_CLIPBOARD_PASTE);
                            let tooltip_ui = |ui: &mut Ui| {
                                ui.label("Duplicate from previous day.");
                            };
                            if ui.add(duplicate_btn).on_hover_ui(tooltip_ui).clicked() {
                                meal_planner.duplicate_day(col_idx - 1, col_idx);
                            };
                        }
                    });
                });
                ScrollArea::new([false, true])
                    .id_source(format!("scroll_area{}", col_idx))
                    .show(ui, |ui| {
                        let frame = Frame::default().inner_margin(4.0);

                        let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                            ui.set_min_size(vec2(ui.available_width(), 100.0));
                            for (row_idx, item) in column.iter().enumerate() {
                                let item_id = Id::new(("my_drag_and_drop_demo", col_idx, row_idx));
                                let item_location = Location {
                                    col: col_idx,
                                    row: row_idx,
                                    recipe_index: *item,
                                };
                                let response = ui
                                    .dnd_drag_source(item_id, item_location, |ui| {
                                        ui.label(hb(&meal_planner
                                            .recipies
                                            .get(*item)
                                            .unwrap()
                                            .to_string()));
                                        ui.separator()
                                    })
                                    .response;

                                if response.clicked() {
                                    // println!("Clicked {} {} {}", col_idx, row_idx, &item);
                                }
                                // Detect drops onto this item:
                                if let (Some(pointer), Some(hovered_payload)) = (
                                    ui.input(|i| i.pointer.interact_pos()),
                                    response.dnd_hover_payload::<Location>(),
                                ) {
                                    let rect = response.rect;

                                    // Preview insertion:
                                    let stroke = egui::Stroke::new(1.0, Color32::WHITE);
                                    let insert_row_idx = if *hovered_payload == item_location {
                                        // We are dragged onto ourselves
                                        ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                                        row_idx
                                    } else if pointer.y < rect.center().y {
                                        // Above us
                                        ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                        row_idx
                                    } else {
                                        // Below us
                                        ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                        row_idx + 1
                                    };

                                    if let Some(dragged_payload) = response.dnd_release_payload() {
                                        // The user dropped onto this item.
                                        from = Some(dragged_payload);
                                        to = Some(Location {
                                            col: col_idx,
                                            row: insert_row_idx,
                                            recipe_index: *item,
                                        });
                                    }
                                }
                            }
                        });

                        if let Some(dragged_payload) = dropped_payload {
                            // The user dropped onto the column, but not on any one item.
                            let recipe_index = dragged_payload.recipe_index;
                            from = Some(dragged_payload);
                            to = Some(Location {
                                col: col_idx,
                                row: usize::MAX, // Inset last
                                recipe_index,
                            });
                        }

                        // footer
                        let mut total_daily = Recipe::default();
                        for recipe_index in column {
                            total_daily = meal_planner.recipies[recipe_index].merge(&total_daily);
                        }

                        self.collapsible_nutrients[col_idx].ui(
                            ui,
                            &total_daily.macros,
                            1,
                            "Amount per day",
                        );
                    });
            }
        });

        if let (Some(from), Some(mut to)) = (from, to) {
            if from.col == to.col {
                // Dragging within the same column.
                // Adjust row index if we are re-ordering:
                to.row -= (from.row < to.row) as usize;
            }

            let item = if from.row == usize::MAX {
                from.recipe_index
            } else {
                meal_planner.daily_plan[from.col].remove(from.row)
            };

            let column = &mut meal_planner.daily_plan[to.col];
            to.row = to.row.min(column.len());
            column.insert(to.row, item);
        }
    }
    // ui.interact(ui.clip_rect(), ui.id(), Sense::click_and_drag())
}
