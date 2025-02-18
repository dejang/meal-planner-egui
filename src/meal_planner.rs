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
pub struct RecipeWithoutId {
    pub title: String,
    pub ingredients: String,
    pub instructions: String,
    pub image_url: String,
    pub macros: AnalysisResponse,
    pub servings: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct IncomingState {
    pub recipies: Vec<RecipeWithoutId>,
    pub recipe: RecipeWithoutId,
    pub daily_plan: Vec<Vec<usize>>,
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
    pub fn connect(self, api_key: &str, app_id: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            app_id: app_id.to_string(),
            recipies: self.recipies,
            daily_plan: self.daily_plan,
            ..Default::default()
        }
    }

    pub fn load_from_bytes(&mut self, json: &[u8]) {
        // #[derive(Deserialize)]
        // struct SavedState {
        //     api_key: Option<String>,
        //     app_id: Option<String>,
        //     recipies: Vec<Recipe>,
        //     recipe: Recipe,
        //     daily_plan: Vec<Vec<usize>>,
        // }

        // if let Ok(saved) = serde_json::from_slice::<SavedState>(json) {
        //     self.recipies = from_recipes_vec(saved.recipies);
        //     self.daily_plan = saved.daily_plan;
        //     self.api_key = saved.api_key.unwrap_or_default();
        //     self.app_id = saved.app_id.unwrap_or_default();
        // }
    }

    pub fn from_json(&mut self, json: &str) -> bool {
        let result = serde_json::from_str::<IncomingJSON>(json);
        if let Ok(state) = result {
            self.recipies = from_recipes_vec(&state.meal_planner.recipies);
            // TODO: update this to read the serialized plan
            self.daily_plan = state.meal_planner.daily_plan.iter().map(|day| {
                let mut uuid_vec = Vec::with_capacity(day.len());
                for index in day {
                    let recipe_without_id = state.meal_planner.recipies.get(*index).unwrap();
                    let recipe = *self.search_recipe(&recipe_without_id.title).get(0).unwrap();
                    uuid_vec.push(recipe.id);
                    
                }
                uuid_vec
            }).collect();
            true
        } else {
            println!("{:?}", result.err().unwrap());
            false
        }
    }

    pub fn is_daily_plan_empty(&self) -> bool {
        let mut is_empty = 0;
        for day in &self.daily_plan {
            is_empty += day.len();
        }

        is_empty == 0
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }

    pub fn get_app_id(&self) -> &str {
        &self.app_id
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

fn from_recipes_vec(recipes: &Vec<RecipeWithoutId>) -> HashMap<Uuid, Recipe> {
    recipes
        .iter()
        .map(|f| {
            let id = Uuid::new_v4();
            let recipe = Recipe {
                id,
                title: f.title.to_owned(),
                ingredients: f.ingredients.to_owned(),
                instructions: f.instructions.to_owned(),
                image_url: f.image_url.to_owned(),
                macros: f.macros.to_owned(),
                servings: f.servings,
            };
            (id, recipe)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::models::Recipe;

    use super::MealPlanner;

    #[test]
    fn is_daily_plan_empty() {
        // TODO: re-enable test
        // let app = MealPlanner::default();
        // assert!(app.is_daily_plan_empty());

        // let app = MealPlanner::new(&[Recipe::default()], &[vec![0]]);
        // assert!(!app.is_daily_plan_empty());
    }

    #[test]
    fn is_api_configured() {
        let app = MealPlanner::default();
        assert!(!app.is_api_configured());

        let app = MealPlanner::default().connect("foo", "");
        assert!(!app.is_api_configured());

        let app = MealPlanner::default().connect("", "foo");
        assert!(!app.is_api_configured());

        let app = MealPlanner::default().connect("foo", "bar");
        assert!(app.is_api_configured());
    }

    #[test]
    fn duplace_day() {
        // TODO: re-enable test
        // let mut app = MealPlanner::new(&[Recipe::default()], &[vec![0], vec![]]);
        // app.duplicate_day(0, 1);
        // assert_eq!(**app.daily_plan.get(1).unwrap(), vec![0]);
    }
}
