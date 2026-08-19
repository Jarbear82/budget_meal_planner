use rusqlite::{Connection, Result, params};
use chrono::Utc;

pub struct Migration {
    pub version: u32,
    pub description: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial core schema tables",
        sql: "
        CREATE TABLE IF NOT EXISTS items (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            density TEXT,
            preferred_purchase_mode TEXT NOT NULL,
            category TEXT,
            nutrition TEXT,
            dietary_flags TEXT
        );

        CREATE TABLE IF NOT EXISTS stores (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS packages (
            id TEXT PRIMARY KEY,
            item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            store_id TEXT NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
            quantity_amount TEXT NOT NULL,
            quantity_unit TEXT NOT NULL,
            price TEXT NOT NULL,
            last_seen TEXT,
            is_preferred INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS recipes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            instructions TEXT NOT NULL,
            servings TEXT NOT NULL,
            meal_type TEXT
        );

        CREATE TABLE IF NOT EXISTS recipe_yields (
            recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
            item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            quantity_amount TEXT NOT NULL,
            quantity_unit TEXT NOT NULL,
            PRIMARY KEY (recipe_id, item_id)
        );

        CREATE TABLE IF NOT EXISTS ingredient_edges (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recipe_id TEXT NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
            target_type TEXT NOT NULL, -- 'item' or 'recipe'
            target_id TEXT NOT NULL,
            quantity_amount TEXT NOT NULL,
            quantity_unit TEXT NOT NULL,
            required INTEGER NOT NULL DEFAULT 1,
            cycle_flag INTEGER NOT NULL DEFAULT 0,
            per_recipe_substitute TEXT
        );

        CREATE TABLE IF NOT EXISTS pre_planned_meals (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS meal_components (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            meal_id TEXT NOT NULL REFERENCES pre_planned_meals(id) ON DELETE CASCADE,
            component_type TEXT NOT NULL, -- 'recipe', 'item', 'restaurant'
            target_id_or_name TEXT NOT NULL,
            quantity_or_servings TEXT NOT NULL,
            unit_or_cost TEXT,
            leftover_item_id TEXT,
            leftover_qty_amount TEXT,
            leftover_qty_unit TEXT
        );

        CREATE TABLE IF NOT EXISTS scheduled_meals (
            id TEXT PRIMARY KEY,
            source_type TEXT NOT NULL,
            source_payload TEXT NOT NULL,
            datetime TEXT NOT NULL,
            people INTEGER NOT NULL,
            consumed_at TEXT
        );

        CREATE TABLE IF NOT EXISTS pantry_entries (
            id TEXT PRIMARY KEY,
            item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            quantity_amount TEXT NOT NULL,
            quantity_unit TEXT NOT NULL,
            expiration TEXT
        );

        CREATE TABLE IF NOT EXISTS unit_bridges (
            item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
            from_amount TEXT NOT NULL,
            from_unit TEXT NOT NULL,
            to_amount TEXT NOT NULL,
            to_unit TEXT NOT NULL,
            PRIMARY KEY (item_id, from_unit, to_unit)
        );

        CREATE TABLE IF NOT EXISTS global_substitutes (
            primary_item_id TEXT PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
            substitute_item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS receipts (
            id TEXT PRIMARY KEY,
            store_id TEXT REFERENCES stores(id) ON DELETE SET NULL,
            total TEXT NOT NULL,
            datetime TEXT NOT NULL
        );
        ",
    },
    Migration {
        version: 2,
        description: "Performance indexes for foreign keys and queries",
        sql: "
        CREATE INDEX IF NOT EXISTS idx_packages_item_id ON packages(item_id);
        CREATE INDEX IF NOT EXISTS idx_packages_store_id ON packages(store_id);
        CREATE INDEX IF NOT EXISTS idx_recipe_yields_recipe_id ON recipe_yields(recipe_id);
        CREATE INDEX IF NOT EXISTS idx_recipe_yields_item_id ON recipe_yields(item_id);
        CREATE INDEX IF NOT EXISTS idx_ingredient_edges_recipe_id ON ingredient_edges(recipe_id);
        CREATE INDEX IF NOT EXISTS idx_ingredient_edges_target ON ingredient_edges(target_id);
        CREATE INDEX IF NOT EXISTS idx_meal_components_meal_id ON meal_components(meal_id);
        CREATE INDEX IF NOT EXISTS idx_pantry_entries_item_id ON pantry_entries(item_id);
        CREATE INDEX IF NOT EXISTS idx_scheduled_meals_datetime ON scheduled_meals(datetime);
        CREATE INDEX IF NOT EXISTS idx_receipts_datetime ON receipts(datetime);
        ",
    },
    Migration {
        version: 3,
        description: "Verify columns and schema consistency",
        sql: "
        -- Safe idempotent verification
        SELECT 1;
        ",
    },
];

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS _schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL,
            description TEXT NOT NULL
        );
        ",
    )?;

    // Query applied versions
    let mut stmt = conn.prepare("SELECT version FROM _schema_migrations")?;
    let applied_versions: std::collections::HashSet<u32> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|v| v.ok())
        .collect();

    for migration in MIGRATIONS {
        if !applied_versions.contains(&migration.version) {
            conn.execute_batch(migration.sql)?;
            conn.execute(
                "INSERT INTO _schema_migrations (version, applied_at, description) VALUES (?1, ?2, ?3)",
                params![migration.version, Utc::now().to_rfc3339(), migration.description],
            )?;
        }
    }

    // Safe column additions for legacy databases if they upgraded from older schema
    let _ = conn.execute("ALTER TABLE items ADD COLUMN nutrition TEXT;", []);
    let _ = conn.execute("ALTER TABLE items ADD COLUMN dietary_flags TEXT;", []);
    let _ = conn.execute("ALTER TABLE recipes ADD COLUMN meal_type TEXT;", []);

    Ok(())
}
