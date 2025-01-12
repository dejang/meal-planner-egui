use serde::{Deserialize, Serialize};

use crate::models::Recipe;

#[derive(Debug, Serialize, Deserialize)]
struct OldMealPlanner {
    api_key: Option<String>,
    app_id: Option<String>,
    recipies: Vec<Recipe>,
    recipe: Recipe,
    daily_plan: Vec<Vec<usize>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MealPlanner {
    pub api_key: String,
    pub app_id: String,

    pub recipies: Vec<Recipe>,
    pub recipe: Recipe,
    pub daily_plan: Vec<Vec<usize>>,
}

impl Default for MealPlanner {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            app_id: String::new(),
            recipies: vec![],
            recipe: Recipe::default(),
            daily_plan: vec![vec![], vec![], vec![], vec![], vec![], vec![]],
        }
    }
}

impl MealPlanner {
    pub fn new(recipies: &[Recipe], daily_plan: &[Vec<usize>]) -> Self {
        Self {
            recipies: recipies.to_vec(),
            daily_plan: daily_plan.to_vec(),
            ..Default::default()
        }
    }

    pub fn connect(self, api_key: &str, app_id: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            app_id: app_id.to_string(),
            recipies: self.recipies,
            daily_plan: self.daily_plan,
            recipe: self.recipe,
        }
    }

    pub fn load_from_bytes(&mut self, json: &[u8]) {
        #[derive(Deserialize)]
        struct SavedState {
            api_key: Option<String>,
            app_id: Option<String>,
            recipies: Vec<Recipe>,
            recipe: Recipe,
            daily_plan: Vec<Vec<usize>>,
        }

        if let Ok(saved) = serde_json::from_slice::<SavedState>(json) {
            self.recipe = saved.recipe;
            self.recipies = saved.recipies;
            self.daily_plan = saved.daily_plan;
            self.api_key = saved.api_key.unwrap_or_default();
            self.app_id = saved.app_id.unwrap_or_default();
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
}

#[cfg(test)]
mod tests {
    use crate::models::Recipe;

    use super::MealPlanner;

    #[test]
    fn is_daily_plan_empty() {
        let app = MealPlanner::default();
        assert!(app.is_daily_plan_empty());

        let app = MealPlanner::new(&[Recipe::default()], &[vec![0]]);
        assert!(!app.is_daily_plan_empty());
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
        let mut app = MealPlanner::new(&[Recipe::default()], &[vec![0], vec![]]);
        app.duplicate_day(0, 1);
        assert_eq!(**app.daily_plan.get(1).unwrap(), vec![0]);
    }
}
