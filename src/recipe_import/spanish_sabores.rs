pub(super) const DOMAINS: &[&str] = &["spanishsabores.com"];

#[cfg(test)]
mod tests {
    use crate::recipe_import::extract_json_ld_recipe;

    const PAGE: &str = r#"
        <script type="application/ld+json">
        {"@graph":[{"@type":"Recipe",
          "name":"Spanish Cod and Potato Stew",
          "image":[
            "https://spanishsabores.com/wp-content/uploads/2024/12/Cod-and-Potato-Stew-Featured-01.jpg",
            "https://spanishsabores.com/cod-and-potato-stew-square.jpg"
          ],
          "recipeYield":["4","4 servings"],
          "recipeIngredient":[
            "1/4 cup extra virgin olive oil",
            "1 large onion (diced)",
            "1 pound dried salt cod (desalinated and chopped)",
            "4 medium red potatoes (peeled and cut in wedges)"
          ],
          "recipeInstructions":[
            {"@type":"HowToStep","text":"Sauté the onion and leek in the olive oil."},
            {"@type":"HowToStep","text":"Add the cod and potatoes."},
            {"@type":"HowToStep","text":"Simmer until the potatoes are tender."}
          ]}]}
        </script>
    "#;

    #[test]
    fn extracts_editable_recipe_fields() {
        let recipe = extract_json_ld_recipe(PAGE).unwrap();

        assert_eq!(recipe.title, "Spanish Cod and Potato Stew");
        assert_eq!(recipe.servings, 4);
        assert_eq!(recipe.ingredients.len(), 4);
        assert_eq!(
            recipe.instructions,
            [
                "Sauté the onion and leek in the olive oil.",
                "Add the cod and potatoes.",
                "Simmer until the potatoes are tender."
            ]
        );
        assert_eq!(
            recipe.image_url,
            "https://spanishsabores.com/wp-content/uploads/2024/12/Cod-and-Potato-Stew-Featured-01.jpg"
        );
    }
}
