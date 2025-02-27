use std::collections::HashMap;

use egui::Layout;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::util::{hb, hh, hs, DEFAULT_PADDING};

const VITAMINS: [&str; 10] = [
    "VITA_RAE", "THIA", "RIBF", "NIA", "VITB6A", "VITB12", "VITC", "VITD", "TOCPHA", "VITK1",
];
const MINERALS: [&str; 8] = ["CA", "MG", "ZN", "FE", "P", "K", "NA", "FOLDFE"];

const FOOD_LABEL_CODES: [&str; 9] = [
    "FAT", "FASAT", "FATRN", "CHOLE", "NA", "CHOCDF", "FIBTG", "SUGAR", "PROCNT",
];

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ParsedNutrientSimple {
    pub qty: i32,
    pub measure: String,
    pub food_match: String,
    pub food_id: String,
    pub weight: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NutritionDataModel {
    pub uri: String,
    pub calories: i32,
    pub totalCO2Emissions: f32,
    pub co2EmissionsClass: String,
    pub totalWeight: f32,
    pub dietLabels: Vec<String>,
    pub healthLabels: Vec<String>,
    pub cautions: Vec<String>,
    pub totalNutrients: HashMap<String, Nutrient>,
    pub totalDaily: HashMap<String, Nutrient>,
    pub ingredients: Vec<Ingredient>,
    pub totalNutrientsKCal: HashMap<String, Nutrient>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Nutrient {
    pub label: String,
    pub quantity: f32,
    pub unit: String,
}

impl Nutrient {
    pub fn qty_per_serving(&self, servings: u32) -> u32 {
        (self.quantity.abs().ceil() as u32) / servings
    }
    pub fn qty_with_unit(&self) -> String {
        let qty = self.quantity.abs().ceil() as u32;
        format!("{}{}", qty, self.unit)
    }

    pub fn qty_with_unit_per_serving(&self, servings: u32) -> String {
        let qty = (self.quantity.abs().ceil() as u32) / servings;
        format!("{}{}", qty, self.unit)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Ingredient {
    pub text: String,
    pub parsed: Option<Vec<ParsedNutrient>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct ParsedNutrient {
    pub quantity: f32,
    pub measure: Option<String>,
    pub foodMatch: String,
    pub food: String,
    pub foodId: String,
    pub weight: f32,
    pub retainedWeight: f32,
    pub nutrients: HashMap<String, Nutrient>,
    pub measureURI: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AnalysisRequest {
    pub ingr: Vec<String>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct AnalysisResponse {
    pub uri: String,
    #[serde(alias = "yield")]
    pub _yield: f32,
    pub calories: i32,
    pub totalCO2Emissions: f32,
    pub co2EmissionsClass: String,
    pub totalWeight: f32,
    pub dietLabels: Vec<String>,
    pub healthLabels: Vec<String>,
    pub cautions: Vec<String>,
    pub totalNutrients: HashMap<String, Nutrient>,
    pub totalDaily: HashMap<String, Nutrient>,
    pub ingredients: Vec<Ingredient>,
    pub cuisineType: Vec<String>,
    pub mealType: Vec<String>,
    pub dishType: Vec<String>,
    pub totalNutrientsKCal: HashMap<String, Nutrient>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AnalysisResponseView;

impl AnalysisResponseView {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        response: &AnalysisResponse,
        servings: u32,
        servings_label: &str,
    ) {
        response.ui(ui, servings, servings_label);
        ui.separator();
        let id = format!("analysis_response_view_{}", ui.unique_id().value());

        let mut show_nutrients = ui.data_mut(|data| {
            if let Some(show) = data.get_temp::<bool>(id.clone().into()) {
                show
            } else {
                false
            }
        });

        if ui.button("Nutrients").clicked() {
            show_nutrients = !show_nutrients;
            ui.data_mut(|data| {
                data.insert_temp(id.into(), show_nutrients);
            });
        }
        if show_nutrients {
            let default_nutrient = Nutrient::default();
            for v in VITAMINS {
                let nutrient = response.totalDaily.get(v).unwrap_or(&default_nutrient);
                ui.horizontal_wrapped(|ui| {
                    ui.label(hs(&nutrient.label));
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.label(hs(&nutrient.qty_with_unit_per_serving(servings)));
                    });
                });
            }

            ui.separator();

            for m in MINERALS {
                let nutrient = response.totalDaily.get(m).unwrap_or(&default_nutrient);
                ui.horizontal_wrapped(|ui| {
                    ui.label(hs(&nutrient.label));
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.label(hs(&nutrient.qty_with_unit_per_serving(servings)));
                    });
                });
            }
        }
    }
}

impl AnalysisResponse {
    pub fn ui(&self, ui: &mut egui::Ui, servings: u32, servings_label: &str) {
        ui.vertical(|ui| {
            ui.group(|ui| {
                ui.label(hh("Nutrition Facts"));
                ui.separator();
                ui.label(hs(servings_label));
                let calories_per_serving = self.calories as u32 / servings;
                self.row(ui, "Calories", "", &calories_per_serving.to_string(), &[]);
                ui.separator();
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(hs("% Daily Value*"));
                });

                ui.separator();

                let default_nutrient = Nutrient::default();
                let fat_gr = self.totalNutrients.get("FAT").unwrap_or(&default_nutrient);
                let fat_percent = self.totalDaily.get("FAT").unwrap_or(&default_nutrient);
                let fat_sat_gr = self
                    .totalNutrients
                    .get("FASAT")
                    .unwrap_or(&default_nutrient);
                let fat_sat_percent = self.totalDaily.get("FASAT").unwrap_or(&default_nutrient);
                let trans_fat_gr = self
                    .totalNutrients
                    .get("FATRN")
                    .unwrap_or(&default_nutrient);

                self.row(
                    ui,
                    "Total Fat",
                    &fat_gr.qty_with_unit_per_serving(servings),
                    &fat_percent.qty_with_unit_per_serving(servings),
                    &[
                        (
                            "Saturated Fat",
                            &fat_sat_gr.qty_with_unit_per_serving(servings),
                            &fat_sat_percent.qty_with_unit_per_serving(servings),
                        ),
                        (
                            "Trans Fat",
                            &trans_fat_gr.qty_with_unit_per_serving(servings),
                            "",
                        ),
                    ],
                );

                let cholesterol_gr = self
                    .totalNutrients
                    .get("CHOLE")
                    .unwrap_or(&default_nutrient);
                let cholesterol_percent = self.totalDaily.get("CHOLE").unwrap_or(&default_nutrient);
                self.row(
                    ui,
                    "Cholesterol",
                    &cholesterol_gr.qty_with_unit_per_serving(servings),
                    &cholesterol_percent.qty_with_unit_per_serving(servings),
                    &[],
                );

                let sodium_gr = self.totalNutrients.get("NA").unwrap_or(&default_nutrient);
                let sodium_percent = self.totalDaily.get("NA").unwrap_or(&default_nutrient);
                self.row(
                    ui,
                    "Sodium",
                    &sodium_gr.qty_with_unit_per_serving(servings),
                    &sodium_percent.qty_with_unit_per_serving(servings),
                    &[],
                );

                let carbs_gr = self
                    .totalNutrients
                    .get("CHOCDF")
                    .unwrap_or(&default_nutrient);
                let carbs_percent = self.totalDaily.get("CHOCDF").unwrap_or(&default_nutrient);
                let fiber_gr = self
                    .totalNutrients
                    .get("FIBTG")
                    .unwrap_or(&default_nutrient);
                let fiber_percent = self.totalDaily.get("FIBTG").unwrap_or(&default_nutrient);
                let sugar_gr = self
                    .totalNutrients
                    .get("SUGAR")
                    .unwrap_or(&default_nutrient);

                self.row(
                    ui,
                    "Total Carbohydrate",
                    &carbs_gr.qty_with_unit_per_serving(servings),
                    &carbs_percent.qty_with_unit_per_serving(servings),
                    &[
                        (
                            "Dietary Fiber",
                            &fiber_gr.qty_with_unit_per_serving(servings),
                            &fiber_percent.qty_with_unit_per_serving(servings),
                        ),
                        (
                            "Total Sugars",
                            &sugar_gr.qty_with_unit_per_serving(servings),
                            "",
                        ),
                    ],
                );

                let protein_gr = self
                    .totalNutrients
                    .get("PROCNT")
                    .unwrap_or(&default_nutrient);
                let protein_percent = self.totalDaily.get("PROCNT").unwrap_or(&default_nutrient);
                self.row(
                    ui,
                    "Protein",
                    &protein_gr.qty_with_unit_per_serving(servings),
                    &protein_percent.qty_with_unit_per_serving(servings),
                    &[],
                );
            });
        });
    }

    fn row(
        &self,
        ui: &mut egui::Ui,
        key: &str,
        qty: &str,
        daily: &str,
        children: &[(&str, &str, &str)],
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(hb(key));
                ui.label(hs(qty));
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(hs(daily));
                });
            });

            if !children.is_empty() {
                for (key, qty, daily) in children {
                    ui.horizontal(|ui| {
                        ui.add_space(DEFAULT_PADDING);
                        ui.label(hs(key));
                        ui.label(hs(qty));
                        ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                            ui.label(hs(daily));
                        });
                    });
                }
            }
        });
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Recipe {
    pub id: Uuid,
    pub title: String,
    pub ingredients: String,
    pub instructions: String,
    pub image_url: String,
    pub macros: AnalysisResponse,
    pub servings: u32,
}

impl Default for Recipe {
    fn default() -> Self {
        Self {
            title: String::new(),
            ingredients: String::new(),
            instructions: String::new(),
            image_url: String::new(),
            macros: AnalysisResponse::default(),
            servings: 1,
            id: Uuid::new_v4(),
        }
    }
}

impl Recipe {
    pub fn ingredients_to_vec(&self) -> Vec<String> {
        self.ingredients
            .split('\n')
            .map(|line| line.trim().to_string())
            .filter(|line| line.ne(&"".to_string()))
            .collect()
    }

    // produces the sum of 1 serving from each recipe
    pub fn merge(&self, from: &Recipe) -> Recipe {
        let mut merged = Recipe::default();

        merged.macros.calories = self.macros.calories / self.servings as i32
            + from.macros.calories / from.servings as i32;

        let default_nutrient = Nutrient::default();

        let codes: Vec<&&str> = FOOD_LABEL_CODES
            .iter()
            .chain(VITAMINS.iter())
            .chain(MINERALS.iter())
            .collect();

        for code in codes {
            let self_nutrient_gr = self
                .macros
                .totalNutrients
                .get(*code)
                .unwrap_or(&default_nutrient);
            let self_daily_nutrient = self
                .macros
                .totalDaily
                .get(*code)
                .unwrap_or(&default_nutrient);

            let from_nutrient_gr = from
                .macros
                .totalNutrients
                .get(*code)
                .unwrap_or(&default_nutrient);

            let from_nutrient_daily = from
                .macros
                .totalDaily
                .get(*code)
                .unwrap_or(&default_nutrient);

            merged.macros.totalNutrients.insert(
                code.to_string(),
                Nutrient {
                    label: self_nutrient_gr.label.clone(),
                    quantity: self_nutrient_gr.quantity / self.servings as f32
                        + from_nutrient_gr.quantity / from.servings as f32,
                    unit: self_nutrient_gr.unit.clone(),
                },
            );

            merged.macros.totalDaily.insert(
                code.to_string(),
                Nutrient {
                    label: self_daily_nutrient.label.clone(),
                    quantity: self_daily_nutrient.quantity / self.servings as f32
                        + from_nutrient_daily.quantity / from.servings as f32,
                    unit: self_daily_nutrient.unit.clone(),
                },
            );
        }
        merged
    }
}

impl ToString for Recipe {
    fn to_string(&self) -> String {
        if self.title.is_empty() {
            return "Default Recipe".to_string();
        }
        self.title.to_string()
    }
}

impl ToString for &Recipe {
    fn to_string(&self) -> String {
        self.title.to_string()
    }
}
