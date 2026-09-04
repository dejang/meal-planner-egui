// healthyfitnessrecipes.com is retained as an alias for the requested domain;
// the live site and supplied example use healthyfitnessmeals.com.
pub(super) const DOMAINS: &[&str] = &["healthyfitnessmeals.com", "healthyfitnessrecipes.com"];

#[cfg(test)]
mod tests {
    use crate::{models::Recipe, recipe_import::extract_json_ld_recipe};

    const SOURCE_URL: &str = "https://healthyfitnessmeals.com/cottage-cheese-scrambled-eggs/";
    const PAGE: &str = r#"
        <script type="application/ld+json">
        {"@graph":[{"@type":"Recipe",
          "name":"Cottage Cheese Scrambled Eggs",
          "image":["https://healthyfitnessmeals.com/cottage-cheese-eggs.jpg"],
          "recipeYield":["2","2 servings"],
          "recipeIngredient":["5 large eggs","1/2 cup cottage cheese","1 tbsp olive oil"],
          "recipeInstructions":[
            {"@type":"HowToStep","text":"Whisk the eggs and cottage cheese."},
            {"@type":"HowToStep","text":"Heat the oil in a pan."}
          ]}]}
        </script>
    "#;

    #[test]
    fn extracts_editable_recipe_fields() {
        let recipe = extract_json_ld_recipe(PAGE).unwrap();

        assert_eq!(recipe.title, "Cottage Cheese Scrambled Eggs");
        assert_eq!(recipe.servings, 2);
        assert_eq!(recipe.ingredients.len(), 3);
        assert_eq!(recipe.instructions.len(), 2);
        assert_eq!(
            recipe.image_url,
            "https://healthyfitnessmeals.com/cottage-cheese-eggs.jpg"
        );
    }

    #[test]
    fn populates_the_existing_recipe_model_with_source_url() {
        let imported = extract_json_ld_recipe(PAGE).unwrap();
        let mut recipe = Recipe::default();

        imported.apply_to(&mut recipe, SOURCE_URL);

        assert_eq!(recipe.title, "Cottage Cheese Scrambled Eggs");
        assert_eq!(recipe.ingredients_to_vec().len(), 3);
        assert!(recipe.instructions.ends_with(&format!("\n\n{SOURCE_URL}")));
        assert_eq!(recipe.servings, 2);
    }
}
