#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod icons;
mod meal_planner;
mod models;
mod planner;
mod recipe_editor;
mod recipe_gallery;
mod shopping_list;
mod theme;
mod util;
pub use app::MealPlannerApp;
pub use theme::*;
