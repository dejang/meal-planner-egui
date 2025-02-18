use egui::*;
use uuid::Uuid;

use crate::{
    meal_planner::MealPlanner,
    models::{AnalysisResponseView, Recipe},
    typography::icons::{ICON_CLIPBOARD_PASTE, ICON_MONITOR_COG, ICON_TRASH_2},
    util::ls,
};

/// What is being dragged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct Location {
    pub col: usize,
    pub row: usize,
    pub recipe_id: Uuid,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Planner {
    search_term: String,
    collapsible_nutrients: Vec<AnalysisResponseView>,
    context_menu_pos: Pos2,
    show_context_menu: bool,
    context_menu_payload: Option<Location>,
}

impl Default for Planner {
    fn default() -> Self {
        Self {
            search_term: String::new(),
            collapsible_nutrients: (0..7).map(|_| AnalysisResponseView::default()).collect(),
            context_menu_pos: Pos2::default(),
            show_context_menu: false,
            context_menu_payload: None,
        }
    }
}

impl Planner {
    pub fn ui(&mut self, ui: &mut egui::Ui, meal_planner: &mut MealPlanner) {
        // If there is a drop, store the location of the item being dragged, and the destination for the drop.
        let mut from = None;
        let mut to = None;
        ui.columns(meal_planner.get_daily_plan().len(), |uis| {
            for (col_idx, column) in meal_planner.get_daily_plan().clone().iter().enumerate() {
                let ui = &mut uis[col_idx];
                ui.horizontal(|ui| {
                    ui.heading(format!("Day {}", col_idx + 1));
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        let clear_btn = Button::new(ICON_TRASH_2);
                        let tooltip_ui = |ui: &mut Ui| {
                            ui.label("Clear meals");
                        };
                        if ui.add(clear_btn).on_hover_ui(tooltip_ui).clicked() {
                            meal_planner.clear_planner_day(col_idx);
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
                        let pointer_pos = ui.ctx().pointer_latest_pos();
                        let pointer_any_pressed = ui.ctx().input(|i| i.pointer.any_pressed());
                        let pointer_is_over_area = ui.ctx().is_pointer_over_area();

                        let frame = Frame::default().inner_margin(4.0);

                        let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                            ui.set_min_size(vec2(ui.available_width(), 100.0));
                            for (row_idx, recipe_id) in column.iter().enumerate() {
                                let ui_item_id =
                                    Id::new(("my_drag_and_drop_demo", col_idx, row_idx));
                                let item_location = Location {
                                    col: col_idx,
                                    row: row_idx,
                                    recipe_id: *recipe_id,
                                };
                                let response = ui
                                    .dnd_drag_source(ui_item_id, item_location, |ui| {
                                        Frame::default()
                                            .show(ui, |ui| {
                                                ui.label(
                                                    ls(&meal_planner
                                                        .get_recipe_by_id(recipe_id)
                                                        .unwrap()
                                                        .title)
                                                    .size(16.),
                                                );
                                                ui.separator();
                                                ui.interact(
                                                    ui.max_rect(),
                                                    ui_item_id,
                                                    Sense::click_and_drag(),
                                                )
                                            })
                                            .inner
                                    })
                                    .inner;

                                if response.clicked_by(PointerButton::Secondary) {
                                    if let Some(pos) = pointer_pos {
                                        self.context_menu_payload = Some(Location {
                                            col: col_idx,
                                            row: row_idx,
                                            recipe_id: *recipe_id,
                                        });
                                        self.context_menu_pos = pos;
                                    }
                                    self.show_context_menu = true;
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
                                            recipe_id: *recipe_id,
                                        });
                                    }
                                }
                            }
                        });

                        if self.show_context_menu {
                            // Draw an `Area` on top (foreground) at the stored position
                            egui::Area::new("my_context_menu_area".into())
                                .order(egui::Order::Foreground)
                                .fixed_pos(self.context_menu_pos)
                                .show(ui.ctx(), |ui| {
                                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                                        if ui.button(format!("{} Edit", ICON_MONITOR_COG)).clicked()
                                        {
                                        }
                                        if ui.button(format!("{} Remove", ICON_TRASH_2)).clicked() {
                                            if let Some(payload) = self.context_menu_payload {
                                                meal_planner.remove_planner_recipe(
                                                    payload.col,
                                                    payload.row,
                                                );
                                                self.context_menu_payload = None;
                                                self.show_context_menu = false;
                                            }
                                        }
                                    });
                                });

                            // --- 4) Close the menu if the user clicks elsewhere ---
                            if pointer_any_pressed && !pointer_is_over_area {
                                self.show_context_menu = false;
                            }
                        }

                        if let Some(dragged_payload) = dropped_payload {
                            // The user dropped onto the column, but not on any one item.
                            let recipe_id = dragged_payload.recipe_id;
                            from = Some(dragged_payload);
                            to = Some(Location {
                                col: col_idx,
                                row: usize::MAX, // Inset last
                                recipe_id,
                            });
                        }

                        // footer
                        let mut total_daily = Recipe::default();
                        for recipe_id in column {
                            total_daily = meal_planner
                                .get_recipe_by_id(recipe_id)
                                .unwrap()
                                .merge(&total_daily);
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

            let recipe_id = if from.row == usize::MAX {
                from.recipe_id
            } else {
                meal_planner.remove_planner_recipe(from.col,from.row)
            };

            meal_planner.add_recipe_to_planner(to.col, to.row, recipe_id);
        }
    }
    // ui.interact(ui.clip_rect(), ui.id(), Sense::click_and_drag())
}
