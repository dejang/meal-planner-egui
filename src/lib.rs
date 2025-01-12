#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod icons;
mod meal_planner;
mod models;
mod planner;
mod recipe_editor;
mod recipe_viewer;
mod shopping_list;
mod util;
mod theme;
pub use theme::*;
pub use app::MealPlannerApp;
