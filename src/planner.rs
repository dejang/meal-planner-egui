use egui::*;
use uuid::Uuid;

use crate::{
    icon,
    meal_planner::MealPlanner,
    models::{AnalysisResponseView, Recipe},
    theme::{on_forest, palette},
    typography::icons::{ICON_CLIPBOARD_PASTE, ICON_PENCIL, ICON_TRASH_2},
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
}

impl Default for Planner {
    fn default() -> Self {
        Self {
            search_term: String::new(),
            collapsible_nutrients: (0..7).map(|_| AnalysisResponseView).collect(),
        }
    }
}

impl Planner {
    pub fn ui(&mut self, ui: &mut egui::Ui, meal_planner: &mut MealPlanner) -> Option<Uuid> {
        // If there is a drop, store the location of the item being dragged, and the destination for the drop.
        let mut edit_recipe = None;
        let mut remove_recipe = None;
        let mut from = None;
        let mut to = None;
        ui.columns(meal_planner.get_daily_plan().len(), |uis| {
            for (col_idx, column) in meal_planner.get_daily_plan().clone().iter().enumerate() {
                let ui = &mut uis[col_idx];
                Frame::new()
                    .fill(palette::FOREST)
                    .corner_radius(10)
                    .inner_margin(Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        on_forest(ui);
                        ui.spacing_mut().button_padding = vec2(6.0, 5.0);
                        ui.horizontal(|ui| {
                            ui.heading(
                                RichText::new(format!("Day {}", col_idx + 1))
                                    .color(palette::CITRUS),
                            );
                            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                                let clear_btn = Button::new(icon(ICON_TRASH_2));
                                if ui.add(clear_btn).on_hover_text("Clear").clicked() {
                                    meal_planner.clear_planner_day(col_idx);
                                };

                                if col_idx > 0 {
                                    let duplicate_btn = Button::new(icon(ICON_CLIPBOARD_PASTE));
                                    if ui
                                        .add(duplicate_btn)
                                        .on_hover_text("Copy from previous day")
                                        .clicked()
                                    {
                                        meal_planner.duplicate_day(col_idx - 1, col_idx);
                                    };
                                }
                            });
                        });
                    });
                ui.add_space(6.0);
                ScrollArea::new([false, true])
                    .id_salt(format!("scroll_area{}", col_idx))
                    .show(ui, |ui| {
                        let frame = Frame::default()
                            .inner_margin(6.0)
                            .fill(palette::SAGE)
                            .corner_radius(10);

                        let (_, dropped_payload) = ui.dnd_drop_zone::<Location, ()>(frame, |ui| {
                            ui.set_min_size(vec2(ui.available_width(), 100.0));
                            if column.is_empty() {
                                ui.painter().text(
                                    ui.min_rect().center(),
                                    Align2::CENTER_CENTER,
                                    "Drop a recipe here",
                                    TextStyle::Small.resolve(ui.style()),
                                    palette::MUTED,
                                );
                            }
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
                                            .fill(palette::SURFACE)
                                            .stroke(Stroke::new(1.0, palette::BORDER))
                                            .corner_radius(8)
                                            .inner_margin(8)
                                            .show(ui, |ui| {
                                                ui.style_mut().interaction.selectable_labels =
                                                    false;
                                                ui.set_width(ui.available_width());
                                                let title = WidgetText::from(
                                                    ls(&meal_planner
                                                        .get_recipe_by_id(recipe_id)
                                                        .unwrap()
                                                        .title)
                                                    .size(16.)
                                                    .color(palette::FOREST),
                                                );
                                                let title = title.into_galley(
                                                    ui,
                                                    Some(TextWrapMode::Wrap),
                                                    ui.available_width(),
                                                    TextStyle::Body,
                                                );
                                                ui.label(title);
                                            });
                                    })
                                    .response
                                    // Use one interaction for clicks and drags so the drag
                                    // source does not steal secondary clicks from the menu.
                                    .interact(Sense::click_and_drag());

                                response.context_menu(|ui| {
                                    if ui.button((icon(ICON_PENCIL), "Edit")).clicked() {
                                        edit_recipe = Some(*recipe_id);
                                        ui.close();
                                    }
                                    if ui.button((icon(ICON_TRASH_2), "Remove")).clicked() {
                                        remove_recipe = Some(item_location);
                                        ui.close();
                                    }
                                });
                                // Detect drops onto this item:
                                if let (Some(pointer), Some(hovered_payload)) = (
                                    ui.input(|i| i.pointer.interact_pos()),
                                    response.dnd_hover_payload::<Location>(),
                                ) {
                                    let rect = response.rect;

                                    // Preview insertion:
                                    let stroke = egui::Stroke::new(3.0, palette::GREEN);
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
                        ui.add_space(8.0);
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

        if let Some(location) = remove_recipe {
            meal_planner.remove_planner_recipe(location.col, location.row);
        }

        if let (Some(from), Some(mut to)) = (from, to) {
            if from.col == to.col {
                // Dragging within the same column.
                // Adjust row index if we are re-ordering:
                to.row -= (from.row < to.row) as usize;
            }

            let recipe_id = if from.row == usize::MAX {
                from.recipe_id
            } else {
                meal_planner.remove_planner_recipe(from.col, from.row)
            };

            meal_planner.add_recipe_to_planner(to.col, to.row, recipe_id);
        }
        edit_recipe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planned_recipe_supports_context_menu_and_drag() {
        let ctx = Context::default();
        crate::set_theme(&ctx, crate::Theme::default());
        let mut meals = MealPlanner::default();
        let recipe = meals.create_draft_recipe().unwrap();
        recipe.title = "Test recipe".to_owned();
        let recipe_id = recipe.id;
        meals.add_recipe_to_planner(0, 0, recipe_id);
        let mut planner = Planner::default();
        let mut render = |events| {
            let mut output = ctx.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1400.0, 900.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    planner.ui(ui, &mut meals);
                },
            );
            output.textures_delta.clear();
        };
        render(vec![]);
        render(vec![]);
        let row_id = Id::new(("my_drag_and_drop_demo", 0_usize, 0_usize));
        let pos = ctx.read_response(row_id).unwrap().rect.center();
        let pointer = |button, pressed| Event::PointerButton {
            pos,
            button,
            pressed,
            modifiers: Modifiers::NONE,
        };

        render(vec![Event::PointerMoved(pos)]);
        render(vec![pointer(PointerButton::Secondary, true)]);
        render(vec![pointer(PointerButton::Secondary, false)]);
        assert!(ctx.read_response(row_id).unwrap().context_menu_opened());

        for pressed in [true, false] {
            render(vec![Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: Modifiers::NONE,
            }]);
        }
        assert!(!ctx.read_response(row_id).unwrap().context_menu_opened());

        render(vec![pointer(PointerButton::Primary, true)]);
        render(vec![Event::PointerMoved(pos + vec2(40.0, 40.0))]);
        render(vec![]);
        assert_eq!(
            DragAndDrop::payload::<Location>(&ctx).as_deref(),
            Some(&Location {
                col: 0,
                row: 0,
                recipe_id,
            })
        );
    }
}
