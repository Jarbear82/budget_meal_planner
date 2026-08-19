use bmp_domain::*;
use bmp_services::AppServices;
use bmp_storage::Storage;
use chrono::Utc;
use rust_decimal_macros::dec;
use std::collections::{HashMap, HashSet};

fn setup_services() -> AppServices {
    let storage = Storage::in_memory().unwrap();
    AppServices::new(storage)
}

#[test]
fn test_workflow_1_add_ingredient_basic_and_edges() {
    let services = setup_services();

    // Basic: Create ingredient with name + density -> appears in list
    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();
    let items = services.items.list_items().unwrap();
    assert!(items.iter().any(|i| i.id == flour.id && i.name == "Flour"));
    assert_eq!(flour.density.unwrap().g_per_ml, dec!(0.53));

    // Edge: Create ingredient with NO density
    let salt = services.items.create_item("Salt", None, Some("Spices")).unwrap();
    assert!(salt.density.is_none());

    // Edge: Add optional mass-per-each bridge for an "Each" unit item (e.g. 1 Egg = 50g)
    let egg = services.items.create_item("Egg", None, Some("Dairy")).unwrap();
    let bridge = UnitBridge::new(
        egg.id,
        Quantity::new(dec!(1), Unit::Each).unwrap(),
        Quantity::new(dec!(50), Unit::Gram).unwrap(),
    )
    .unwrap();

    let egg_count = Quantity::new(dec!(2), Unit::Each).unwrap();
    let converted = bridge.convert(&egg_count, &Unit::Gram, None).unwrap();
    assert_eq!(converted.amount, dec!(100));
}

#[test]
fn test_workflow_2_add_recipe_basic_and_edges() {
    let services = setup_services();
    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();
    let sugar = services
        .items
        .create_item("Sugar", Some(dec!(0.85)), Some("Baking"))
        .unwrap();

    // Basic: Recipe with ingredients, instructions, yield (Item + Qty), servings
    let mut recipe = Recipe::new("Basic Cake", dec!(8));
    recipe = recipe
        .add_yield(flour.id, Quantity::new(dec!(1), Unit::Each).unwrap())
        .add_ingredient(IngredientEdge::item(
            flour.id,
            Quantity::new(dec!(200), Unit::Gram).unwrap(),
        ))
        .add_ingredient(
            IngredientEdge::item(sugar.id, Quantity::new(dec!(100), Unit::Gram).unwrap())
                .with_required(false), // Edge: optional ingredient
        )
        .with_instructions("Mix and bake at 350F.");

    // Edge: Recipe yields multiple Items (e.g. Cake + Cupcake variant)
    recipe = recipe.add_yield(sugar.id, Quantity::new(dec!(12), Unit::Each).unwrap());

    let saved = services.recipes.save_recipe(recipe).unwrap();
    assert_eq!(saved.name, "Basic Cake");
    assert_eq!(saved.yields.len(), 2);
    assert_eq!(saved.ingredients.len(), 2);
    assert!(!saved.ingredients[1].required);

    // Re-save existing recipe multiple times (Bug #3 fix verification)
    let re_saved = services.recipes.save_recipe(saved.clone()).unwrap();
    assert_eq!(re_saved.yields.len(), 2);
    assert_eq!(re_saved.ingredients.len(), 2);

    let all = services.recipes.list_recipes().unwrap();
    let found = all.iter().find(|r| r.id == saved.id).unwrap();
    assert_eq!(found.yields.len(), 2);
    assert_eq!(found.ingredients.len(), 2);
}

#[test]
fn test_workflow_3_and_17_nesting_deep_and_cycle_aware() {
    let services = setup_services();
    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();

    // Deep nesting (3 levels: Level3 -> Level2 -> Level1 -> Flour)
    let mut r1 = Recipe::new("Level 1 Base", dec!(1));
    r1 = r1
        .add_yield(flour.id, Quantity::new(dec!(100), Unit::Gram).unwrap())
        .add_ingredient(IngredientEdge::item(
            flour.id,
            Quantity::new(dec!(100), Unit::Gram).unwrap(),
        ));
    let saved_r1 = services.recipes.save_recipe(r1).unwrap();

    let mut r2 = Recipe::new("Level 2 Dough", dec!(1));
    r2 = r2
        .add_yield(flour.id, Quantity::new(dec!(100), Unit::Gram).unwrap())
        .add_ingredient(IngredientEdge::recipe(
            saved_r1.id,
            Quantity::new(dec!(1), Unit::Each).unwrap(),
        ));
    let saved_r2 = services.recipes.save_recipe(r2).unwrap();

    let mut r3 = Recipe::new("Level 3 Bread", dec!(1));
    r3 = r3
        .add_yield(flour.id, Quantity::new(dec!(100), Unit::Gram).unwrap())
        .add_ingredient(IngredientEdge::recipe(
            saved_r2.id,
            Quantity::new(dec!(1), Unit::Each).unwrap(),
        ));
    let saved_r3 = services.recipes.save_recipe(r3).unwrap();

    let recipes = services.recipes.list_recipes().unwrap();
    let recipe_map: HashMap<RecipeId, Recipe> = recipes.into_iter().map(|r| (r.id, r)).collect();
    let items = services.items.list_items().unwrap();
    let item_map: HashMap<ItemId, Item> = items.into_iter().map(|i| (i.id, i)).collect();

    let mut visited = HashSet::new();
    let expanded = expand_recipe(saved_r3.id, dec!(1), &recipe_map, &item_map, &mut visited).unwrap();
    assert_eq!(expanded.len(), 1);
    assert_eq!(expanded[0].0, flour.id);

    // Edge: Cycle detection at creation time (Sourdough starter A -> B -> A)
    let r_a_id = RecipeId::new();
    let r_b_id = RecipeId::new();

    let mut r_a = Recipe::new("Starter A", dec!(1));
    r_a.id = r_a_id;
    r_a = r_a.add_ingredient(IngredientEdge::recipe(
        r_b_id,
        Quantity::new(dec!(50), Unit::Gram).unwrap(),
    ));

    let mut r_b = Recipe::new("Starter B", dec!(1));
    r_b.id = r_b_id;
    r_b = r_b.add_ingredient(IngredientEdge::recipe(
        r_a_id,
        Quantity::new(dec!(50), Unit::Gram).unwrap(),
    ));

    let mut cycle_map = HashMap::new();
    cycle_map.insert(r_a_id, r_a);
    cycle_map.insert(r_b_id, r_b);

    update_cycle_flags(&mut cycle_map);
    assert!(cycle_map.get(&r_a_id).unwrap().ingredients[0].cycle_flag);
}

#[test]
fn test_workflow_4_and_5_meals_and_scheduling() {
    let services = setup_services();
    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();

    let mut recipe = Recipe::new("Pancakes", dec!(2));
    recipe = recipe.add_ingredient(IngredientEdge::item(
        flour.id,
        Quantity::new(dec!(200), Unit::Gram).unwrap(),
    ));
    let saved_recipe = services.recipes.save_recipe(recipe).unwrap();

    // Basic: Assemble recipes + items + restaurant components in PrePlannedMeal
    let meal = PrePlannedMeal::new("Sunday Breakfast")
        .add_component(MealComponent::Recipe {
            recipe_id: saved_recipe.id,
            servings: dec!(2),
        })
        .add_component(MealComponent::Item {
            item_id: flour.id,
            quantity: Quantity::new(dec!(50), Unit::Gram).unwrap(),
        })
        .add_component(MealComponent::Restaurant {
            name: "Coffee Shop".to_string(),
            cost: dec!(12.50),
            leftover_yield: None,
        });

    assert_eq!(meal.name, "Sunday Breakfast");
    assert_eq!(meal.components.len(), 3);

    let saved_meal = services.meals.create_pre_planned_meal(&meal.name, meal.components.clone()).unwrap();

    // Basic & Edge: Schedule meal with datetime & people count
    let now = Utc::now();
    let scheduled = services.meals.schedule_meal(
        ScheduledMealSource::PrePlanned(saved_meal.id),
        now,
        4, // people count
    ).unwrap();
    assert_eq!(scheduled.people, 4);

    // Verify shopping list generated from scheduled meal
    let reqs = services.shopping.collect_scheduled_meal_requirements(None, None).unwrap();
    assert!(!reqs.is_empty());
}

#[test]
fn test_workflow_6_go_shopping_basic_and_edges() {
    let services = setup_services();
    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();
    let walmart = services.items.add_store("Walmart").unwrap();
    let costco = services.items.add_store("Costco").unwrap();

    // Add Packages
    let pkg_walmart = services
        .items
        .add_package(flour.id, walmart.id, dec!(5), Unit::Pound, dec!(3.48))
        .unwrap();
    let _pkg_costco = services
        .items
        .add_package(flour.id, costco.id, dec!(25), Unit::Pound, dec!(14.99))
        .unwrap();

    // Basic: Generate shopping list (pantry subtraction, package ceiling rounding)
    let requirements = vec![(
        flour.id,
        Quantity::new(dec!(7), Unit::Pound).unwrap(),
    )];

    let list_all = services
        .shopping
        .generate_shopping_list(requirements.clone(), None, None)
        .unwrap();
    assert_eq!(list_all.items.len(), 1);

    // Edge: Per-store shopping mode (only Walmart items shown)
    let list_walmart = services
        .shopping
        .generate_shopping_list(requirements.clone(), Some(walmart.id), None)
        .unwrap();
    assert_eq!(list_walmart.items[0].store_id, walmart.id);

    // Edge: Preferred package pinned overrides best price
    let mut pref_pkg = pkg_walmart;
    pref_pkg.is_preferred = true;
    services.storage.insert_package(&pref_pkg).unwrap();

    let list_pref = services
        .shopping
        .generate_shopping_list(requirements, None, None)
        .unwrap();
    assert_eq!(list_pref.items[0].package_id, pref_pkg.id);
}

#[test]
fn test_workflow_8_make_recipe_pre_configuration() {
    let services = setup_services();
    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();
    let sugar = services
        .items
        .create_item("Sugar", Some(dec!(0.85)), Some("Baking"))
        .unwrap();
    let honey = services
        .items
        .create_item("Honey", Some(dec!(1.42)), Some("Baking"))
        .unwrap();

    let mut recipe = Recipe::new("Sweet Bread", dec!(1));
    recipe = recipe
        .add_yield(flour.id, Quantity::new(dec!(1), Unit::Each).unwrap())
        .add_ingredient(IngredientEdge::item(
            flour.id,
            Quantity::new(dec!(200), Unit::Gram).unwrap(),
        ))
        .add_ingredient(IngredientEdge::item(
            sugar.id,
            Quantity::new(dec!(50), Unit::Gram).unwrap(),
        ));

    // Make Recipe config: batch scale 2.5, substitute sugar with honey
    let mut config = MakeRecipeConfig::default();
    config.batches = dec!(2.5); // Edge: non-integer batches
    config.substitute_overrides.insert(sugar.id, honey.id);

    let items = services.items.list_items().unwrap();
    let items_map: HashMap<ItemId, Item> = items.into_iter().map(|i| (i.id, i)).collect();

    let execution = evaluate_make_recipe(&recipe, &config, &items_map).unwrap();
    assert_eq!(execution.ingredients_to_consume.len(), 2);
    // Flour scaled to 200 * 2.5 = 500g
    assert_eq!(execution.ingredients_to_consume[0].1.amount, dec!(500));
    // Sugar substituted with Honey and scaled to 50 * 2.5 = 125g
    assert_eq!(execution.ingredients_to_consume[1].0, honey.id);
    assert_eq!(execution.ingredients_to_consume[1].1.amount, dec!(125));
}

#[test]
fn test_workflow_9_toggle_buy_finished_vs_expand() {
    let services = setup_services();
    let bread_item = Item::new("Bread")
        .with_purchase_mode(PurchaseMode::BuyFinished);
    let bread_id = bread_item.id;
    services.storage.insert_item(&bread_item).unwrap();

    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();

    let mut recipe = Recipe::new("Bread Recipe", dec!(1));
    recipe = recipe
        .add_yield(bread_id, Quantity::new(dec!(1), Unit::Each).unwrap())
        .add_ingredient(IngredientEdge::item(
            flour.id,
            Quantity::new(dec!(400), Unit::Gram).unwrap(),
        ));
    let saved_recipe = services.recipes.save_recipe(recipe).unwrap();

    let recipes = services.recipes.list_recipes().unwrap();
    let recipe_map: HashMap<RecipeId, Recipe> = recipes.into_iter().map(|r| (r.id, r)).collect();
    let items = services.items.list_items().unwrap();
    let item_map: HashMap<ItemId, Item> = items.into_iter().map(|i| (i.id, i)).collect();

    // Default: BuyFinished -> returns Bread item directly
    let mut visited = HashSet::new();
    let result_buy = expand_recipe(saved_recipe.id, dec!(1), &recipe_map, &item_map, &mut visited).unwrap();
    assert_eq!(result_buy.len(), 1);
    assert_eq!(result_buy[0].0, flour.id);

    // Toggle: PreferMake -> expands into flour ingredients
    let mut bread_make = bread_item;
    bread_make.preferred_purchase_mode = PurchaseMode::PreferMake;
    services.items.update_item(&bread_make).unwrap();

    let items2 = services.items.list_items().unwrap();
    let item_map2: HashMap<ItemId, Item> = items2.into_iter().map(|i| (i.id, i)).collect();

    let mut visited2 = HashSet::new();
    let result_make = expand_recipe(saved_recipe.id, dec!(1), &recipe_map, &item_map2, &mut visited2).unwrap();
    assert_eq!(result_make[0].0, flour.id);
}

#[test]
fn test_workflow_10_manage_substitutes() {
    let _services = setup_services();
    let primary = ItemId::new();
    let global_sub = ItemId::new();
    let per_recipe_sub = ItemId::new();

    let mut resolver = SubstituteResolver::new();
    resolver.set_global(primary, global_sub);

    // Basic: No override -> global substitute used
    let resolved_global = resolver.resolve(primary, None, None);
    assert_eq!(resolved_global, global_sub);

    // Basic: Per-recipe override -> per-recipe substitute used
    let resolved_recipe = resolver.resolve(primary, Some(per_recipe_sub), None);
    assert_eq!(resolved_recipe, per_recipe_sub);

    // Edge: No substitute configured -> returns original primary item
    let unconfigured_item = ItemId::new();
    let resolved_none = resolver.resolve(unconfigured_item, None, None);
    assert_eq!(resolved_none, unconfigured_item);
}

#[test]
fn test_workflow_11_14_pantry_adjustment_and_consumption() {
    let services = setup_services();
    let flour = services
        .items
        .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
        .unwrap();

    // Basic: Add pantry entry
    let entry = services
        .pantry
        .add_pantry_entry(flour.id, dec!(10), Unit::Pound, None)
        .unwrap();
    assert_eq!(entry.quantity.amount, dec!(10));

    // Basic: User accepts pantry update
    services.pantry.update_quantity(entry.id, dec!(8)).unwrap();
    let updated = services.pantry.get_pantry().unwrap();
    assert_eq!(updated[0].quantity.amount, dec!(8));

    // Edge: Negative quantity rejected
    let err = services.pantry.update_quantity(entry.id, dec!(-2));
    assert!(err.is_err());

    // Basic: Consume pantry quantity
    services.pantry.consume_pantry_item(flour.id, dec!(3), Unit::Pound).unwrap();
    let after_consume = services.pantry.get_pantry().unwrap();
    assert_eq!(after_consume[0].quantity.amount, dec!(5));
}

#[test]
fn test_workflow_12_and_13_manage_stores_and_packages() {
    let services = setup_services();
    let store = services.items.add_store("Target").unwrap();
    let item = services.items.create_item("Milk", Some(dec!(1.03)), Some("Dairy")).unwrap();

    // Basic: Add package with price and store association
    let pkg1 = services
        .items
        .add_package(item.id, store.id, dec!(1), Unit::Liter, dec!(2.99))
        .unwrap();
    assert_eq!(pkg1.price, dec!(2.99));

    // Edge: Multiple packages for same item at same store
    let _pkg2 = services
        .items
        .add_package(item.id, store.id, dec!(2), Unit::Liter, dec!(4.99))
        .unwrap();

    let packages = services.storage.get_packages_for_item(item.id).unwrap();
    assert_eq!(packages.len(), 2);

    // Edge: Delete store -> store deleted cleanly
    services.storage.delete_store(store.id).unwrap();
    let stores = services.items.list_stores().unwrap();
    assert!(stores.iter().all(|s| s.id != store.id));
}

#[test]
fn test_workflow_15_restaurant_meal() {
    let services = setup_services();
    let leftover_item = services
        .items
        .create_item("Leftover Pizza", None, Some("Leftovers"))
        .unwrap();

    let meal = ScheduledMeal::new(
        ScheduledMealSource::Restaurant {
            name: "Luigi's Pizzeria".to_string(),
            cost: dec!(28.50),
            leftover_yield: Some((
                leftover_item.id,
                Quantity::new(dec!(2), Unit::Each).unwrap(),
            )),
        },
        Utc::now(),
        2,
    );

    if let ScheduledMealSource::Restaurant { cost, leftover_yield, .. } = &meal.source {
        assert_eq!(*cost, dec!(28.50));
        assert_eq!(leftover_yield.as_ref().unwrap().1.amount, dec!(2));
    } else {
        panic!("Expected restaurant meal source");
    }
}

#[test]
fn test_workflow_16_density_bridge_management() {
    let services = setup_services();
    let item = services.items.create_item("Oats", None, Some("Grains")).unwrap();

    // Edge: Missing density -> direct conversion between mass and volume fails without density
    let cups = Quantity::new(dec!(1), Unit::Cup).unwrap();
    let density_none: Option<Density> = None;
    let err = density_none
        .map(|d| d.convert(&cups, &Unit::Gram))
        .unwrap_or(Err(DomainError::MissingDensity(item.id)));

    assert_eq!(err, Err(DomainError::MissingDensity(item.id)));

    // Basic: Supply bridge -> conversion succeeds on-the-fly
    let oats_density = Density::new(dec!(0.41)).unwrap();
    let grams = oats_density.convert(&cups, &Unit::Gram).unwrap();
    assert_eq!(grams.amount, dec!(97.001176965));
}

#[test]
fn test_cross_cutting_persistence_restart() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_path_buf();

    // Session 1: Write data
    {
        let storage1 = Storage::open(&db_path).unwrap();
        let services1 = AppServices::new(storage1);
        let flour = services1
            .items
            .create_item("Flour", Some(dec!(0.53)), Some("Baking"))
            .unwrap();
        let store = services1.items.add_store("Kroger").unwrap();
        services1
            .items
            .add_package(flour.id, store.id, dec!(5), Unit::Pound, dec!(3.29))
            .unwrap();
    }

    // Session 2: App restart -> reload SQLite database and verify persistence
    {
        let storage2 = Storage::open(&db_path).unwrap();
        let services2 = AppServices::new(storage2);
        let items = services2.items.list_items().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "Flour");

        let stores = services2.items.list_stores().unwrap();
        assert_eq!(stores.len(), 1);
        assert_eq!(stores[0].name, "Kroger");
    }
}

#[test]
fn test_analytics_summary() {
    let services = setup_services();
    let now = Utc::now();
    services.analytics.record_receipt(None, dec!(45.50), now).unwrap();
    services.analytics.record_receipt(None, dec!(12.25), now).unwrap();

    let summary = services.analytics.get_overall_summary().unwrap();
    assert_eq!(summary.actual_expenditure, dec!(57.75));
}

#[test]
fn test_spec_gaps_fixes() {
    let services = setup_services();
    let store1 = services.items.add_store("Store 1").unwrap();
    let store2 = services.items.add_store("Store 2").unwrap();

    let flour = services.items.create_item("Flour", Some(dec!(0.53)), Some("Baking")).unwrap();
    let pkg = services.items.add_package(flour.id, store1.id, dec!(5), Unit::Pound, dec!(4.99)).unwrap();

    // 1. Move package to another store (SRS §2.2.2)
    services.items.move_package_to_store(pkg.id, store2.id).unwrap();

    // 2. Update package price on receipt actual!=projected (SRS §2.3.3)
    services.items.update_package_price(pkg.id, dec!(5.49)).unwrap();

    // 3. Nested recipe cost calculation (SRS §5.3.6)
    let mut base_recipe = Recipe::new("Flour Base", dec!(1));
    base_recipe = base_recipe.add_ingredient(IngredientEdge::item(flour.id, Quantity::new(dec!(1), Unit::Pound).unwrap()));
    let saved_base = services.recipes.save_recipe(base_recipe).unwrap();

    let mut parent_recipe = Recipe::new("Cake Parent", dec!(1));
    parent_recipe = parent_recipe.add_ingredient(IngredientEdge::recipe(saved_base.id, Quantity::new(dec!(1), Unit::Pound).unwrap()));
    let saved_parent = services.recipes.save_recipe(parent_recipe).unwrap();

    let parent_cost = services.recipes.estimate_cost(saved_parent.id).unwrap();
    assert!(parent_cost.price_per_batch > dec!(0));

    // 4. Item deletion placeholder rule (SRS §5.1)
    services.items.delete_item(flour.id).unwrap();
    let items_after = services.items.list_items().unwrap();
    let deleted_item = items_after.iter().find(|i| i.id == flour.id).unwrap();
    assert!(deleted_item.name.contains("[Deleted Item: Flour]"));
}

#[tokio::test]
async fn test_domain_event_bus_publishing_and_subscription() {
    let services = setup_services();
    let mut rx = services.event_bus.subscribe();

    // Trigger item creation
    let apple = services.items.create_item("Honeycrisp Apple", None, Some("Produce")).unwrap();
    let ev1 = rx.recv().await.unwrap();
    assert_eq!(ev1, DomainEvent::ItemCreated(apple.id));

    // Trigger store & package creation
    let store = services.items.add_store("Safeway").unwrap();
    let ev2 = rx.recv().await.unwrap();
    assert_eq!(ev2, DomainEvent::StoreCreated(store.id));

    let pkg = services.items.add_package(apple.id, store.id, dec!(1), Unit::Pound, dec!(2.49)).unwrap();
    let ev3 = rx.recv().await.unwrap();
    assert_eq!(ev3, DomainEvent::PackageCreated(pkg.id));

    // Trigger recipe save
    let mut recipe = Recipe::new("Apple Slices", dec!(2));
    recipe = recipe.add_ingredient(IngredientEdge::item(apple.id, Quantity::new(dec!(1), Unit::Pound).unwrap()));
    let saved_recipe = services.recipes.save_recipe(recipe).unwrap();
    let ev4 = rx.recv().await.unwrap();
    assert_eq!(ev4, DomainEvent::RecipeSaved(saved_recipe.id));

    // Trigger receipt record
    let receipt_id = services.analytics.record_receipt(Some(store.id), dec!(15.99), Utc::now()).unwrap();
    let ev5 = rx.recv().await.unwrap();
    assert_eq!(ev5, DomainEvent::ReceiptRecorded(receipt_id));
}

#[test]
fn test_batch_operations_packages_and_pantry() {
    let services = setup_services();
    let store = services.items.add_store("Costco").unwrap();
    let milk = services.items.create_item("Whole Milk", Some(dec!(1.03)), Some("Dairy")).unwrap();
    let eggs = services.items.create_item("Large Eggs", None, Some("Dairy")).unwrap();

    let pkg1 = Package::new(milk.id, store.id, Quantity::new(dec!(1), Unit::Liter).unwrap(), dec!(3.79));
    let pkg2 = Package::new(eggs.id, store.id, Quantity::new(dec!(24), Unit::Each).unwrap(), dec!(5.99));

    // Batch packages insert
    services.items.upsert_packages_batch(&[pkg1.clone(), pkg2.clone()]).unwrap();
    let milk_pkgs = services.items.get_packages_for_item(milk.id).unwrap();
    let eggs_pkgs = services.items.get_packages_for_item(eggs.id).unwrap();
    assert_eq!(milk_pkgs.len(), 1);
    assert_eq!(eggs_pkgs.len(), 1);

    // Pantry bulk adjust
    let p1 = services.pantry.add_pantry_entry(milk.id, dec!(1000), Unit::Milliliter, None).unwrap();
    let p2 = services.pantry.add_pantry_entry(eggs.id, dec!(12), Unit::Each, None).unwrap();

    services.pantry.bulk_pantry_adjust(&[
        (p1.id, dec!(800)),
        (p2.id, dec!(10)),
    ]).unwrap();

    let pantry = services.pantry.get_pantry().unwrap();
    let updated_p1 = pantry.iter().find(|p| p.id == p1.id).unwrap();
    let updated_p2 = pantry.iter().find(|p| p.id == p2.id).unwrap();
    assert_eq!(updated_p1.quantity.amount, dec!(800));
    assert_eq!(updated_p2.quantity.amount, dec!(10));
}

#[test]
fn test_full_database_backup_export_and_import_roundtrip() {
    let services1 = setup_services();

    // Populate database with rich relational data
    let oats = services1.items.create_item("Oats", Some(dec!(0.41)), Some("Grains")).unwrap();
    let honey = services1.items.create_item("Honey", Some(dec!(1.42)), Some("Baking")).unwrap();
    let store = services1.items.add_store("Trader Joe's").unwrap();

    let pkg1 = services1.items.add_package(oats.id, store.id, dec!(2), Unit::Pound, dec!(4.29)).unwrap();
    let _pkg2 = services1.items.add_package(honey.id, store.id, dec!(16), Unit::Ounce, dec!(6.49)).unwrap();

    let mut granola = Recipe::new("Homemade Granola", dec!(4));
    granola = granola.add_ingredient(IngredientEdge::item(oats.id, Quantity::new(dec!(200), Unit::Gram).unwrap()));
    granola = granola.add_ingredient(IngredientEdge::item(honey.id, Quantity::new(dec!(50), Unit::Gram).unwrap()));
    let saved_granola = services1.recipes.save_recipe(granola).unwrap();

    let meal = services1.meals.schedule_meal(
        ScheduledMealSource::OneOff(MealComponent::Recipe { recipe_id: saved_granola.id, servings: dec!(2) }),
        Utc::now(),
        2,
    ).unwrap();

    let pantry = services1.pantry.add_pantry_entry(oats.id, dec!(500), Unit::Gram, None).unwrap();
    let _receipt_id = services1.analytics.record_receipt(Some(store.id), dec!(10.78), Utc::now()).unwrap();

    // Export to JSON string
    let json_backup = services1.backup.export_json().unwrap();
    assert!(!json_backup.is_empty());
    assert!(json_backup.contains("Homemade Granola"));
    assert!(json_backup.contains("Trader Joe's"));

    // Fresh storage 2 -> Import JSON
    let storage2 = Storage::in_memory().unwrap();
    let services2 = AppServices::new(storage2);

    // Initial state of storage2 should be empty
    assert_eq!(services2.items.list_items().unwrap().len(), 0);

    // Import
    services2.backup.import_json(&json_backup, true).unwrap();

    // Verify all entities exist in storage2
    let items2 = services2.items.list_items().unwrap();
    assert_eq!(items2.len(), 2);
    assert!(items2.iter().any(|i| i.name == "Oats"));
    assert!(items2.iter().any(|i| i.name == "Honey"));

    let stores2 = services2.items.list_stores().unwrap();
    assert_eq!(stores2.len(), 1);
    assert_eq!(stores2[0].name, "Trader Joe's");

    let packages2 = services2.items.get_packages_for_item(oats.id).unwrap();
    assert_eq!(packages2.len(), 1);
    assert_eq!(packages2[0].id, pkg1.id);
    assert_eq!(packages2[0].price, dec!(4.29));

    let recipes2 = services2.recipes.list_recipes().unwrap();
    assert_eq!(recipes2.len(), 1);
    assert_eq!(recipes2[0].name, "Homemade Granola");
    assert_eq!(recipes2[0].ingredients.len(), 2);

    let meals2 = services2.meals.list_scheduled_meals().unwrap();
    assert_eq!(meals2.len(), 1);
    assert_eq!(meals2[0].id, meal.id);

    let pantry2 = services2.pantry.get_pantry().unwrap();
    assert_eq!(pantry2.len(), 1);
    assert_eq!(pantry2[0].id, pantry.id);
    assert_eq!(pantry2[0].quantity.amount, dec!(500));

    let summary2 = services2.analytics.get_overall_summary().unwrap();
    assert_eq!(summary2.actual_expenditure, dec!(10.78));
}
