use std::collections::HashMap;

use egui_extras::{Column, TableBuilder};

use crate::models::Recipe;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShoppingList {}

impl ShoppingList {
    fn shopping_list(&self, plan: &[Vec<usize>], recipe_list: &[Recipe]) -> Vec<(String, f32)> {
        let mut list = HashMap::new();
        plan.iter().for_each(|day| {
            for r_id in day {
                let recipe = recipe_list.get(*r_id).unwrap();
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
        ids.iter()
            .map(|id| list.get(id.as_str()).unwrap().clone())
            .collect::<Vec<(String, f32)>>()
    }

    pub fn show(&self, ui: &mut egui::Ui, plan: &[Vec<usize>], recipies: &[Recipe]) {
        let list = self.shopping_list(plan, recipies);

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .column(Column::remainder())
            .column(Column::auto())
            .header(30., |mut header| {
                header.col(|ui| {
                    ui.heading("Ingredient");
                });
                header.col(|ui| {
                    ui.heading("Quantity");
                });
            })
            .body(|mut body| {
                for (name, weight) in list {
                    body.row(20., |mut row| {
                        row.col(|ui| {
                            ui.label(name);
                        });
                        row.col(|ui| {
                            ui.label(format!("{}g", weight));
                        });
                    })
                }
            });
    }
}
