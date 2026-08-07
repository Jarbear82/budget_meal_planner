use bmp_domain::*;
use bmp_services::AppServices;
use bmp_storage::Storage;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[test]
fn test_explicit_workflow_1_add_item_and_density() {
    let storage = Storage::in_memory().unwrap();
    let services = AppServices::new(storage);

    // 1. Create Item "Flour" with density 0.53 g/ml
    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();

    assert_eq!(flour.name, "Flour");
    assert_eq!(flour.density.unwrap().g_per_ml, dec!(0.53));

    // Convert 2 Cups of Flour to Grams using density
    let cups = Quantity::new(dec!(2), Unit::Cup).unwrap();
    let grams = flour.density.unwrap().convert(&cups, &Unit::Gram).unwrap();
    assert_eq!(grams.amount, dec!(250.78353069));
}

#[test]
fn test_explicit_workflow_2_recipe_creation_and_nesting_expansion() {
    let storage = Storage::in_memory().unwrap();
    let services = AppServices::new(storage.clone());

    let dough_item = Item::new("Dough")
        .with_density(Density::new(dec!(0.8)).unwrap())
        .with_category("Baking")
        .with_purchase_mode(PurchaseMode::PreferMake);
    let dough_id = dough_item.id;
    storage.insert_item(&dough_item).unwrap();

    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();
    let water = services
        .items
        .create_item("Water", Some(dec!(1.0)), Some("Pantry"))
        .unwrap();

    // Create Sub-Recipe "Dough Base"
    let mut dough_recipe = Recipe::new("Dough Base", dec!(1));
    dough_recipe = dough_recipe.add_yield(
        dough_id,
        Quantity::new(dec!(500), Unit::Gram).unwrap(),
    );
    dough_recipe = dough_recipe.add_ingredient(IngredientEdge::item(
        flour.id,
        Quantity::new(dec!(300), Unit::Gram).unwrap(),
    ));
    dough_recipe = dough_recipe.add_ingredient(IngredientEdge::item(
        water.id,
        Quantity::new(dec!(200), Unit::Milliliter).unwrap(),
    ));

    let saved_dough = services.recipes.save_recipe(dough_recipe).unwrap();

    // Create Parent Recipe "Pizza Crust" referencing "Dough Base"
    let mut pizza_recipe = Recipe::new("Pizza Crust", dec!(2));
    pizza_recipe = pizza_recipe.add_ingredient(IngredientEdge::recipe(
        saved_dough.id,
        Quantity::new(dec!(1), Unit::Each).unwrap(),
    ));

    let saved_pizza = services.recipes.save_recipe(pizza_recipe).unwrap();

    // Verify expansion resolves to base ingredients
    let recipes_list = services.recipes.list_recipes().unwrap();
    let recipe_map: HashMap<RecipeId, Recipe> =
        recipes_list.into_iter().map(|r| (r.id, r)).collect();
    let items_list = services.items.list_items().unwrap();
    let item_map: HashMap<ItemId, Item> = items_list.into_iter().map(|i| (i.id, i)).collect();

    let mut visited = std::collections::HashSet::new();
    let requirements = expand_recipe(
        saved_pizza.id,
        dec!(1),
        &recipe_map,
        &item_map,
        &mut visited,
    )
    .unwrap();

    assert_eq!(requirements.len(), 2);
}

#[test]
fn test_explicit_workflow_3_shopping_list_pantry_subtraction_and_package_rounding() {
    let storage = Storage::in_memory().unwrap();
    let services = AppServices::new(storage);

    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();
    let store = services.items.add_store("Walmart").unwrap();

    // Package: 5 lb Flour for $3.48
    let _pkg = services
        .items
        .add_package(flour.id, store.id, dec!(5), Unit::Pound, dec!(3.48))
        .unwrap();

    // Add 1 lb Flour to Pantry
    let _pantry = services
        .pantry
        .add_pantry_entry(flour.id, dec!(1), Unit::Pound, None)
        .unwrap();

    // Required: 8 lb Flour. After subtracting 1 lb in pantry = 7 lb remaining required.
    // 7 lb / 5 lb per package = 1.4 packages -> rounds UP to 2 whole packages ($6.96 total)
    let requirements = vec![(
        flour.id,
        Quantity::new(dec!(8), Unit::Pound).unwrap(),
    )];

    let shopping_list = services
        .shopping
        .generate_shopping_list(requirements, Some(store.id), None)
        .unwrap();

    assert_eq!(shopping_list.items.len(), 1);
    assert_eq!(shopping_list.items[0].package_count, 2);
    assert_eq!(shopping_list.subtotal, dec!(6.96));
}
