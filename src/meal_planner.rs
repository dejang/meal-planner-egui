use ehttp::Request;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use crate::models::{AnalysisRequest, AnalysisResponse, Recipe};

#[derive(Debug, Serialize, Deserialize)]
struct IncomingState {
    pub api_key: String,
    pub app_id: String,
    pub recipies: HashMap<Uuid, Recipe>,
    pub daily_plan: Vec<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IncomingJSON {
    meal_planner: IncomingState,
}

#[derive(Debug, PartialEq)]
enum ApiRequest {
    Idle,
    Requesting(Uuid),
    Complete(Uuid, AnalysisResponse),
    Error(Uuid, String),
}

impl Default for ApiRequest {
    fn default() -> Self {
        ApiRequest::Idle
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MealPlanner {
    pub api_key: String,
    pub app_id: String,
    recipies: HashMap<Uuid, Recipe>,
    daily_plan: Vec<Vec<Uuid>>,
    #[serde(skip)]
    api_request: Arc<Mutex<ApiRequest>>,
    draft_recipe: Option<Uuid>,
}

impl Default for MealPlanner {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            app_id: String::new(),
            recipies: HashMap::new(),
            daily_plan: vec![vec![], vec![], vec![], vec![], vec![], vec![]],
            api_request: Arc::new(Mutex::new(ApiRequest::Idle)),
            draft_recipe: None,
        }
    }
}

impl MealPlanner {
    pub fn from_json(&mut self, json: &str) -> bool {
        let result = serde_json::from_str::<IncomingJSON>(json);
        if let Ok(state) = result {
            self.api_key = state.meal_planner.api_key;
            self.app_id = state.meal_planner.app_id;
            self.recipies = state.meal_planner.recipies;
            self.daily_plan = state.meal_planner.daily_plan;
            true
        } else {
            println!("{:?}", result.err().unwrap());
            false
        }
    }

    pub fn is_api_configured(&self) -> bool {
        !self.api_key.is_empty() && !self.app_id.is_empty()
    }

    pub fn duplicate_day(&mut self, src_day: usize, dst_day: usize) {
        let src_day_recipies = self.daily_plan.get(src_day);
        let dst_day_recipies = src_day_recipies.cloned();
        self.daily_plan[dst_day] = dst_day_recipies.unwrap();
    }

    pub fn remove_recipe(&mut self, recipe_id: &Uuid) {
        // Remove the recipe
        self.recipies.remove(recipe_id);

        // Update daily plan: remove references and adjust indices
        for day in &mut self.daily_plan {
            day.retain(|&meal_id| meal_id != *recipe_id);
        }
    }

    pub fn lookup_nutrients_for_recipe_id(&mut self, ctx: &egui::Context, id: Uuid) {
        let request = self.api_request.clone();
        if ApiRequest::Idle != *request.lock().unwrap() {
            warn!("Pending request");
            return;
        }
        *request.lock().unwrap() = ApiRequest::Requesting(id.clone());

        let recipe = self.recipies.get(&id).unwrap();
        self.request(ctx, id, recipe.ingredients_to_vec());
    }

    pub fn poll_analysis(&mut self) {
        if let Ok(mut lock) = self.api_request.clone().try_lock() {
            match &*lock {
                ApiRequest::Complete(uuid, analysis_response) => {
                    self.recipies.get_mut(&uuid).unwrap().macros = analysis_response.to_owned();
                    *lock = ApiRequest::Idle
                }
                ApiRequest::Error(uuid, err) => {
                    error!("{} - {}", uuid, err);
                    *lock = ApiRequest::Idle
                }
                _ => {}
            }
        }
    }

    pub fn create_draft_recipe(&mut self) -> Option<&mut Recipe> {
        if self.draft_recipe.is_some() {
            return None;
        }
        let recipe = Recipe::default();
        let id = recipe.id.clone();
        self.recipies.insert(recipe.id, recipe);
        self.draft_recipe = Some(id);
        self.recipies.get_mut(&id)
    }

    pub fn delete_draft_recipe(&mut self) {
        if let Some(id) = self.draft_recipe {
            let recipe = self.recipies.get(&id).unwrap();
            if recipe.title.trim().is_empty() {
                self.recipies.remove(&id);
            }
            self.draft_recipe = None;
        }
    }

    pub fn search_recipe(&self, arg: &str) -> Vec<&Recipe> {
        let result = self
            .recipies
            .iter()
            .filter(|(_key, value)| value.title.to_lowercase().contains(&arg.to_lowercase()))
            .map(|(_key, value)| value)
            .collect::<Vec<&Recipe>>();
        result
    }

    pub fn get_recipes(&self) -> Vec<&Recipe> {
        self.recipies.iter().map(|(_key, value)| value).collect()
    }

    pub fn get_daily_plan(&self) -> &Vec<Vec<Uuid>> {
        self.daily_plan.as_ref()
    }

    pub fn clear_planner_day(&mut self, day: usize) {
        self.daily_plan.get_mut(day).unwrap().clear();
    }

    pub fn remove_planner_recipe(&mut self, day: usize, recipe_position: usize) -> Uuid {
        self.daily_plan
            .get_mut(day)
            .unwrap()
            .remove(recipe_position)
    }

    pub fn add_recipe_to_planner(&mut self, day: usize, recipe_position: usize, recipe_id: Uuid) {
        let day_plan = self.daily_plan.get_mut(day).unwrap();

        let insert_position = if day_plan.len() == 0 {
            day_plan.len()
        } else if recipe_position > day_plan.len() - 1 {
            day_plan.len()
        } else {
            recipe_position
        };

        day_plan.insert(insert_position, recipe_id);
    }

    pub fn get_recipe_by_id(&self, id: &Uuid) -> Option<&Recipe> {
        self.recipies.get(id)
    }
    pub fn get_recipe_by_id_mut(&mut self, id: &Uuid) -> Option<&mut Recipe> {
        self.recipies.get_mut(id)
    }

    fn request(&mut self, ctx: &egui::Context, recipe_id: Uuid, ingr: Vec<String>) {
        let analysis_request = AnalysisRequest { ingr };

        let url = format!(
            "https://api.edamam.com/api/nutrition-details?app_id={}&app_key={}",
            self.app_id, self.api_key
        );

        let ctx = ctx.clone();
        let request = self.api_request.clone();
        ehttp::fetch(
            Request::json(url, &analysis_request).unwrap(),
            move |response| {
                if let Ok(response) = response {
                    let raw_text = response.text().unwrap();
                    if response.status == 200 {
                        let analysis = serde_json::from_str(raw_text).unwrap();
                        *request.lock().unwrap() = ApiRequest::Complete(recipe_id, analysis);
                    } else {
                        *request.lock().unwrap() =
                            ApiRequest::Error(recipe_id, raw_text.to_string());
                    }
                    ctx.request_repaint(); // Wake up UI thread
                } else {
                    let network_error = response.err().unwrap();
                    error!("Network Error: {}", network_error);
                    *request.lock().unwrap() = ApiRequest::Error(recipe_id, network_error);
                    ctx.request_repaint(); // Wake up UI thread
                }
            },
        );
    }
}
