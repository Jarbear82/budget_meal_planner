use bmp_domain::*;
use bmp_services::AppServices;
use bmp_storage::Storage;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn bench_unit_conversions_throughput() {
    let density = Density::new(dec!(0.85)).unwrap();
    let qty = Quantity::new(dec!(2.5), Unit::Cup).unwrap();

    let start = Instant::now();
    let iterations = 100_000;

    for _ in 0..iterations {
        let _ = density.convert(&qty, &Unit::Gram).unwrap();
    }

    let elapsed = start.elapsed();
    let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();

    println!(
        "\n--- PERFORMANCE METRIC: Unit Conversion Throughput ---"
    );
    println!("Total Iterations: {}", iterations);
    println!("Time Elapsed: {:?}", elapsed);
    println!("Throughput: {:.2} ops/sec", ops_per_sec);
    assert!(ops_per_sec > 100_000.0);
}

#[test]
fn bench_recipe_cycle_detection_scaling() {
    let mut recipes = HashMap::new();
    let num_recipes = 1_000;

    let recipe_ids: Vec<RecipeId> = (0..num_recipes).map(|_| RecipeId::new()).collect();

    for i in 0..num_recipes {
        let mut r = Recipe::new(format!("Recipe {}", i), dec!(1));
        r.id = recipe_ids[i];
        let next_idx = (i + 1) % num_recipes;
        r = r.add_ingredient(IngredientEdge::recipe(
            recipe_ids[next_idx],
            Quantity::new(dec!(1), Unit::Each).unwrap(),
        ));
        recipes.insert(r.id, r);
    }

    let start = Instant::now();
    update_cycle_flags(&mut recipes);
    let elapsed = start.elapsed();

    println!(
        "\n--- PERFORMANCE METRIC: 1,000-Node Cycle Detection ---"
    );
    println!("Graph Nodes: {}", num_recipes);
    println!("Cycle Detection Time: {:?}", elapsed);
    assert!(elapsed.as_millis() < 3000);
}

#[test]
fn bench_sqlite_transaction_throughput() {
    let storage = Storage::in_memory().unwrap();
    let services = AppServices::new(storage);

    let start = Instant::now();
    let count = 1_000;

    for i in 0..count {
        let _ = services
            .items
            .create_item(&format!("Bench Item {}", i), Some(dec!(1.0)), Some("Pantry"))
            .unwrap();
    }

    let elapsed = start.elapsed();
    let writes_per_sec = (count as f64) / elapsed.as_secs_f64();

    println!(
        "\n--- PERFORMANCE METRIC: SQLite Writes Throughput ---"
    );
    println!("Items Inserted: {}", count);
    println!("Time Elapsed: {:?}", elapsed);
    println!("Throughput: {:.2} writes/sec", writes_per_sec);
    assert!(writes_per_sec > 100.0);
}
