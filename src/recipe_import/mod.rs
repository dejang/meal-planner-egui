mod healthy_fitness_meals;
mod spanish_sabores;

use ehttp::Request;
use scraper::{Html, Selector};
use serde_json::{Map, Value};
use std::sync::{Arc, Mutex};
use url::Url;

use crate::models::Recipe;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedRecipe {
    pub title: String,
    pub ingredients: Vec<String>,
    pub instructions: Vec<String>,
    pub image_url: String,
    pub servings: u32,
}

impl ImportedRecipe {
    pub fn apply_to(self, recipe: &mut Recipe, source_url: &str) {
        recipe.title = self.title;
        recipe.ingredients = self.ingredients.join("\n");
        recipe.instructions = format!("{}\n\n{source_url}", self.instructions.join("\n"));
        recipe.image_url = self.image_url;
        recipe.servings = self.servings;
    }
}

type Extractor = fn(&str) -> Result<ImportedRecipe, String>;

struct ExtractorRegistration {
    domains: &'static [&'static str],
    extract: Extractor,
}

const EXTRACTORS: &[ExtractorRegistration] = &[
    ExtractorRegistration {
        domains: healthy_fitness_meals::DOMAINS,
        extract: extract_json_ld_recipe,
    },
    ExtractorRegistration {
        domains: spanish_sabores::DOMAINS,
        extract: extract_json_ld_recipe,
    },
];

#[derive(Debug, Default)]
enum ImportState {
    #[default]
    Idle,
    Loading,
    Ready(ImportedRecipe),
    Failed(String),
}

#[derive(Clone, Debug, Default)]
pub struct RecipeImporter {
    state: Arc<Mutex<ImportState>>,
}

impl RecipeImporter {
    pub fn start(&self, ctx: &egui::Context, recipe_url: &str) -> Result<(), String> {
        let recipe_url = recipe_url.trim();
        let extractor = extractor_for_url(recipe_url)?;
        {
            let mut state = self.state.lock().unwrap();
            if matches!(*state, ImportState::Loading) {
                return Err("A recipe import is already in progress".to_string());
            }
            *state = ImportState::Loading;
        }

        let state = self.state.clone();
        let ctx = ctx.clone();
        ehttp::fetch(Request::get(recipe_url), move |response| {
            let result = response
                .map_err(|error| format!("Could not download recipe: {error}"))
                .and_then(|response| {
                    if response.status != 200 {
                        return Err(format!(
                            "Recipe site returned HTTP status {}",
                            response.status
                        ));
                    }
                    let html = response
                        .text()
                        .ok_or_else(|| "Recipe page was not valid UTF-8".to_string())?;
                    extractor(html)
                });

            *state.lock().unwrap() = match result {
                Ok(recipe) => ImportState::Ready(recipe),
                Err(error) => ImportState::Failed(error),
            };
            ctx.request_repaint();
        });

        Ok(())
    }

    pub fn is_loading(&self) -> bool {
        matches!(*self.state.lock().unwrap(), ImportState::Loading)
    }

    pub fn take_result(&self) -> Option<Result<ImportedRecipe, String>> {
        let mut state = self.state.lock().unwrap();
        match std::mem::take(&mut *state) {
            ImportState::Ready(recipe) => Some(Ok(recipe)),
            ImportState::Failed(error) => Some(Err(error)),
            other => {
                *state = other;
                None
            }
        }
    }
}

fn extractor_for_url(recipe_url: &str) -> Result<Extractor, String> {
    let url = Url::parse(recipe_url.trim()).map_err(|_| "Enter a valid recipe URL".to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("Recipe URLs must use HTTP or HTTPS".to_string());
    }

    let host = url
        .host_str()
        .ok_or_else(|| "Recipe URL does not contain a domain".to_string())?
        .trim_start_matches("www.");

    EXTRACTORS
        .iter()
        .find(|registration| registration.domains.contains(&host))
        .map(|registration| registration.extract)
        .ok_or_else(|| format!("Recipes from {host} are not supported yet"))
}

pub(super) fn extract_json_ld_recipe(html: &str) -> Result<ImportedRecipe, String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();

    for script in document.select(&selector) {
        let Ok(json) = serde_json::from_str::<Value>(&script.inner_html()) else {
            continue;
        };
        if let Some(recipe) = find_recipe(&json) {
            return recipe_from_schema(recipe);
        }
    }

    Err("The page did not contain structured recipe data".to_string())
}

fn find_recipe(value: &Value) -> Option<&Map<String, Value>> {
    match value {
        Value::Object(object) => {
            if has_type(object, "Recipe") {
                return Some(object);
            }
            object.values().find_map(find_recipe)
        }
        Value::Array(values) => values.iter().find_map(find_recipe),
        _ => None,
    }
}

fn has_type(object: &Map<String, Value>, expected: &str) -> bool {
    match object.get("@type") {
        Some(Value::String(value)) => value == expected,
        Some(Value::Array(values)) => values.iter().any(|value| value.as_str() == Some(expected)),
        _ => false,
    }
}

fn recipe_from_schema(schema: &Map<String, Value>) -> Result<ImportedRecipe, String> {
    let title = schema
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .ok_or_else(|| "Recipe title was missing".to_string())?
        .to_string();

    let ingredients = schema
        .get("recipeIngredient")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|ingredient| !ingredient.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|ingredients| !ingredients.is_empty())
        .ok_or_else(|| "Recipe ingredients were missing".to_string())?;

    let mut instructions = Vec::new();
    if let Some(value) = schema.get("recipeInstructions") {
        collect_instructions(value, &mut instructions);
    }
    if instructions.is_empty() {
        return Err("Recipe instructions were missing".to_string());
    }

    let image_url = schema
        .get("image")
        .and_then(first_image_url)
        .ok_or_else(|| "Recipe image was missing".to_string())?
        .to_string();

    Ok(ImportedRecipe {
        title,
        ingredients,
        instructions,
        image_url,
        servings: schema
            .get("recipeYield")
            .and_then(first_positive_integer)
            .unwrap_or(1),
    })
}

fn collect_instructions(value: &Value, instructions: &mut Vec<String>) {
    match value {
        Value::String(text) => push_non_empty(text, instructions),
        Value::Array(values) => {
            for value in values {
                collect_instructions(value, instructions);
            }
        }
        Value::Object(object) => {
            if let Some(text) = object.get("text").and_then(Value::as_str) {
                push_non_empty(text, instructions);
            } else if let Some(items) = object.get("itemListElement") {
                collect_instructions(items, instructions);
            }
        }
        _ => {}
    }
}

fn push_non_empty(text: &str, values: &mut Vec<String>) {
    let text = text.trim();
    if !text.is_empty() {
        values.push(text.to_string());
    }
}

fn first_image_url(value: &Value) -> Option<&str> {
    match value {
        Value::String(url) => Some(url),
        Value::Array(values) => values.iter().find_map(first_image_url),
        Value::Object(object) => object
            .get("url")
            .or_else(|| object.get("contentUrl"))
            .and_then(first_image_url),
        _ => None,
    }
}

fn first_positive_integer(value: &Value) -> Option<u32> {
    match value {
        Value::Number(number) => number.as_u64().and_then(|value| u32::try_from(value).ok()),
        Value::String(text) => text
            .split(|character: char| !character.is_ascii_digit())
            .find(|part| !part.is_empty())
            .and_then(|part| part.parse().ok()),
        Value::Array(values) => values.iter().find_map(first_positive_integer),
        _ => None,
    }
    .filter(|value| *value > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_dispatch_routes_registered_domains() {
        assert!(extractor_for_url(
            "https://healthyfitnessmeals.com/cottage-cheese-scrambled-eggs/"
        )
        .is_ok());
        assert!(extractor_for_url(
            "https://www.healthyfitnessmeals.com/cottage-cheese-scrambled-eggs/"
        )
        .is_ok());
        assert!(
            extractor_for_url("https://spanishsabores.com/cod-and-potato-stew/#recipe").is_ok()
        );
    }

    #[test]
    fn shared_dispatch_rejects_unsupported_and_lookalike_domains() {
        assert!(extractor_for_url("https://example.com/recipe").is_err());
        assert!(extractor_for_url("https://spanishsabores.com.evil.test/recipe").is_err());
    }
}
