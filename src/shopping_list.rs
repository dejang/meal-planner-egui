use std::{collections::HashMap, sync::Arc};

use egui::{
    pos2, vec2, Align, Align2, Button, CentralPanel, FontFamily, FontId, Frame, Layout, Margin,
    Panel, Rect, RichText, ScrollArea, Sense, Stroke, Ui,
};
use uuid::Uuid;

use crate::{icon, meal_planner::MealPlanner, theme::palette, typography::icons::ICON_X};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShoppingList {}

impl ShoppingList {
    fn shopping_list(&self, plan: &[Vec<Uuid>], meal_planner: &MealPlanner) -> Vec<(String, f32)> {
        let mut list = HashMap::new();
        plan.iter().for_each(|day| {
            for r_id in day {
                let recipe = meal_planner.get_recipe_by_id(r_id).unwrap();
                for ingr in &recipe.macros.ingredients {
                    if ingr.parsed.is_none() {
                        continue;
                    }
                    let model = ingr.parsed.as_ref().unwrap();

                    if model.is_empty() {
                        continue;
                    }

                    let model = model.first().unwrap();
                    if !list.contains_key(&model.foodId) {
                        list.insert(model.foodId.clone(), (model.food.clone(), 0.0));
                    }
                    let value = list.get_mut(&model.foodId).unwrap();
                    value.1 += model.weight / recipe.servings as f32;
                }
            }
        });

        let mut ids = list.keys().collect::<Vec<&String>>();
        ids.sort_unstable();
        let mut ingredients = ids
            .iter()
            .map(|id| list.get(id.as_str()).unwrap().clone())
            .collect::<Vec<(String, f32)>>();
        ingredients.sort_by_cached_key(|(name, _)| name.to_lowercase());
        ingredients
    }

    pub fn show(&self, ui: &mut Ui, meal_planner: &MealPlanner) -> bool {
        let plan = meal_planner.get_daily_plan();
        let list = self.shopping_list(plan, meal_planner);
        let mut close = false;

        Panel::top("shopping_sheet_header")
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(palette::PAPER)
                    .inner_margin(Margin::symmetric(24, 14)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Shopping list")
                            .font(FontId::new(32.0, FontFamily::Name("recipe_display".into())))
                            .color(palette::FOREST),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        close = ui
                            .add(Button::new(icon(ICON_X).size(16.0)).frame(false))
                            .on_hover_text("Close shopping list")
                            .clicked();
                    });
                });
            });

        CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(palette::PAPER)
                    .inner_margin(Margin::symmetric(24, 8)),
            )
            .show(ui, |ui| {
                let page_height = (ui.available_height() - 40.0).max(80.0);
                Frame::new()
                    .fill(palette::SURFACE)
                    .corner_radius(6)
                    .inner_margin(20)
                    .show(ui, |ui| {
                        ui.spacing_mut().scroll = egui::style::ScrollStyle::solid();
                        ScrollArea::vertical()
                            .id_salt("shopping_sheet_rows")
                            .auto_shrink([false, false])
                            .max_height(page_height)
                            .show(ui, |ui| {
                                if list.is_empty() {
                                    ui.add_space(24.0);
                                    ui.label(
                                        RichText::new("Your shopping list is empty.")
                                            .size(20.0)
                                            .color(palette::FOREST),
                                    );
                                    ui.label(caption("Add meals to your planner to get started."));
                                    return;
                                }
                                SheetLayout::measure(ui, &list).paint(ui, &list);
                            });
                    });
            });
        close
    }
}

struct MeasuredRow {
    name: Arc<egui::Galley>,
    amount: Arc<egui::Galley>,
    height: f32,
}

struct SheetLayout {
    columns: usize,
    column_gap: f32,
    header_height: f32,
    rows_per_column: usize,
    rows: Vec<MeasuredRow>,
    height: f32,
}

impl SheetLayout {
    fn measure(ui: &Ui, list: &[(String, f32)]) -> Self {
        let width = ui.available_width();
        let font_size = 18.0;
        let column_gap = 32.0;
        let header_height = 34.0;
        // Columns follow the available reading width. List length and window
        // height never change the type size or trigger a search for a tighter fit.
        let columns = (((width + column_gap) / (320.0 + column_gap)) as usize)
            .max(1)
            .min(list.len().max(1));
        let column_width = (width - column_gap * (columns - 1) as f32) / columns as f32;
        let rows_per_column = list.len().div_ceil(columns).max(1);
        let rows = ui.fonts_mut(|fonts| {
            let amount_font = FontId::new(font_size, FontFamily::Name("heading".into()));
            let amounts = list
                .iter()
                .map(|(_, weight)| format_weight(*weight))
                .collect::<Vec<_>>();
            let amount_width = amounts
                .iter()
                .map(|amount| {
                    fonts
                        .layout_no_wrap(amount.clone(), amount_font.clone(), palette::FOREST)
                        .size()
                        .x
                })
                .fold(0.0, f32::max)
                .min(column_width * 0.4);
            let name_width = (column_width - amount_width - 18.0).max(1.0);
            list.iter()
                .zip(amounts)
                .map(|((name, _), amount)| {
                    let name = fonts.layout(
                        name.clone(),
                        FontId::proportional(font_size),
                        palette::INK,
                        name_width,
                    );
                    // Large quantities wrap inside their own column instead of
                    // covering ingredient names on narrow windows.
                    let amount = fonts.layout(
                        amount,
                        amount_font.clone(),
                        palette::FOREST,
                        amount_width.max(1.0),
                    );
                    let height = name.size().y.max(amount.size().y) + 8.0;
                    MeasuredRow {
                        name,
                        amount,
                        height,
                    }
                })
                .collect::<Vec<_>>()
        });
        let height = if rows.is_empty() {
            0.0
        } else {
            header_height
                + rows
                    .chunks(rows_per_column)
                    .map(|chunk| chunk.iter().map(|row| row.height).sum::<f32>())
                    .fold(0.0, f32::max)
        };
        Self {
            columns,
            column_gap,
            header_height,
            rows_per_column,
            rows,
            height,
        }
    }

    fn paint(&self, ui: &mut Ui, list: &[(String, f32)]) {
        let width = ui.available_width();
        let column_width =
            (width - self.column_gap * (self.columns - 1) as f32) / self.columns as f32;
        let (sheet, _) = ui.allocate_exact_size(vec2(width, self.height), Sense::hover());
        for (column, rows) in self.rows.chunks(self.rows_per_column).enumerate() {
            let x = sheet.left() + column as f32 * (column_width + self.column_gap);
            let right = x + column_width;
            let header_y = sheet.top() + 8.0;
            ui.painter().text(
                pos2(x, header_y),
                Align2::LEFT_CENTER,
                "INGREDIENT",
                FontId::proportional(11.0),
                palette::MUTED,
            );
            ui.painter().text(
                pos2(right, header_y),
                Align2::RIGHT_CENTER,
                "AMOUNT",
                FontId::proportional(11.0),
                palette::MUTED,
            );
            ui.painter().hline(
                x..=right,
                sheet.top() + 25.0,
                Stroke::new(1.0, palette::FOREST),
            );

            let mut y = sheet.top() + self.header_height;
            for (row_index, row) in rows.iter().enumerate() {
                let rect = Rect::from_min_size(pos2(x, y), vec2(column_width, row.height));
                let response = ui.interact(
                    rect,
                    ui.id().with(("shopping_row", column, row_index)),
                    Sense::hover(),
                );
                let entry = &list[column * self.rows_per_column + row_index];
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::Label,
                        true,
                        format!("{}, {}", entry.0, format_weight(entry.1)),
                    )
                });
                ui.painter().galley(
                    pos2(x, y + (row.height - row.name.size().y) * 0.5),
                    row.name.clone(),
                    palette::INK,
                );
                ui.painter().galley(
                    pos2(
                        right - row.amount.size().x,
                        y + (row.height - row.amount.size().y) * 0.5,
                    ),
                    row.amount.clone(),
                    palette::FOREST,
                );
                y += row.height;
                ui.painter()
                    .hline(x..=right, y, Stroke::new(0.6, palette::BORDER));
            }
        }
    }
}

fn caption(text: &str) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(13.0))
        .color(palette::MUTED)
}

fn format_weight(weight: f32) -> String {
    // Round only the displayed amount; the aggregated/model weights retain precision.
    format!("{:.0} g", weight.round())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(width: f32, height: f32, entries: &[(String, f32)]) -> SheetLayout {
        let ctx = egui::Context::default();
        crate::set_theme(&ctx, crate::Theme::default());
        let mut layout = None;
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(width, height))),
                ..Default::default()
            },
            |ui| layout = Some(SheetLayout::measure(ui, entries)),
        );
        output.textures_delta.clear();
        layout.unwrap()
    }

    #[test]
    fn arbitrary_lists_keep_all_text_readable_and_in_separate_columns() {
        for count in [0, 1, 4, 17, 63, 240] {
            let entries = (0..count)
                .map(|index| {
                    let name = match index % 4 {
                        0 => format!("Ingredient {index}"),
                        1 => format!(
                            "Ingredient {index}, roasted and unsalted, with a long descriptive name"
                        ),
                        2 => format!("Ingredient{index}{}", "x".repeat(90)),
                        _ => format!("Crème fraîche {index}, whole-grain rye & barley"),
                    };
                    let weight = [0.3, 12.5, 99.9, 123_456_792.0, f32::MAX][index % 5];
                    (name, weight)
                })
                .collect::<Vec<_>>();

            for width in [200.0, 360.0, 700.0, 1100.0, 2000.0] {
                let sheet = layout(width, 800.0, &entries);
                assert_eq!(sheet.rows.len(), entries.len());
                let column_width =
                    (width - sheet.column_gap * (sheet.columns - 1) as f32) / sheet.columns as f32;
                for (row, (name, weight)) in sheet.rows.iter().zip(&entries) {
                    assert_eq!(row.name.text(), name);
                    assert_eq!(row.amount.text(), format_weight(*weight));
                    assert!(
                        row.name.size().x + 18.0 + row.amount.size().x <= column_width + 0.5,
                        "overlapping text for {count} ingredients at width {width}"
                    );
                    assert!(row.height >= row.name.size().y.max(row.amount.size().y));
                    for galley in [&row.name, &row.amount] {
                        assert!(galley
                            .job
                            .sections
                            .iter()
                            .all(|section| { section.format.font_id.size == 18.0 }));
                    }
                }
            }
        }
    }

    #[test]
    fn changing_window_height_does_not_reflow_or_shrink_the_list() {
        let entries = (0..100)
            .map(|index| (format!("Ingredient {index}"), index as f32 + 0.25))
            .collect::<Vec<_>>();
        for width in [360.0, 700.0, 1100.0] {
            let short = layout(width, 200.0, &entries);
            let tall = layout(width, 1000.0, &entries);
            assert_eq!(short.columns, tall.columns);
            assert_eq!(short.rows_per_column, tall.rows_per_column);
            assert_eq!(short.height, tall.height);
            assert!(short.height > 200.0, "long lists must remain scrollable");
        }
    }
}
