#![warn(clippy::all)]

mod app;
mod meal_planner;
mod models;
mod planner;
mod recipe_editor;
mod recipe_gallery;
#[cfg(not(target_arch = "wasm32"))]
mod recipe_import;
mod shopping_list;
mod theme;
mod util;
pub use app::MealPlannerApp;
pub use theme::*;
