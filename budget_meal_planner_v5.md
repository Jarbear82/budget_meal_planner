# **Budget Meal Planner – Software Requirements Specification v5**  
*(Rust Multi-Crate Edition – Unified Domain Model)*

**Developer:** Jarom Anderson  
**Stakeholders:** Julianna Johnson, Nikara Haroldson, Mikayla Chambers, & Doug Anderson  

**Description:** A cost-aware, local-first meal planning application that provides tools to plan and cook meals, estimate and track daily/weekly/monthly/annual costs, and track and determine the best price for items at local stores. Version 5 establishes a single, density-centric domain model as the source of truth, fully specifies core workflows and their tests, and prioritizes the core crates plus a mandatory interactive desktop UI.

---

# 1.0 Introduction

## 1.1 Purpose
To provide meal planning, tracking, and cost analysis tools for budget-conscious users in a safe, high-performance, local-first application. This revision unifies the data model, makes the highest-value tests explicit, and locks down the core + desktop crates as the primary focus.

## 1.2 Scope
This project is a **local-first, client-only** application. It does not require user accounts. Initial scope does not provide cloud sync or online features.

The project is organized as a Cargo workspace with multiple crates. The **primary deliverable** is the local desktop application (`bmp-local`) with at least one interactive UI (GPUI preferred). Secondary crates (CLI, TUI, mobile shell, optional server) are supported but not required for completeness.

To be considered complete, the project must fully implement the meal planning, shopping, inventory, and cost analysis tools listed in Section 2.0, using the unified domain model defined in Section 5. Time permitting, stretch features from Section 3.0 may be added.

**Non-negotiable:** Fully offline operation. No network access is required for any core feature.

## 1.3 Technologies Used

### 1.3.1 Software (Core)
- **Language:** Rust (Edition 2024 or later)
- **Build system:** Cargo workspace
- **Domain & Services:** Pure Rust (no I/O in domain)
- **Persistence:** SQLite via `rusqlite` (primary) or `sqlx`
- **Serialization:** `serde` + `serde_json`
- **Decimal arithmetic:** `rust_decimal`
- **Async runtime (where needed):** `tokio`
- **Error handling:** `thiserror` / `anyhow` (layered appropriately)
- **Testing:** `cargo test`, `proptest`, `tempfile`

### 1.3.2 Front-end / Adapter Crates
| Crate                   | Purpose                              | UI / Interface Technology      | Priority      |
|-------------------------|--------------------------------------|--------------------------------|---------------|
| `bmp-local`             | Primary desktop application          | GPUI (preferred)               | **Mandatory** |
| `bmp-cli`               | Command-line interface               | clap                           | Secondary     |
| `bmp-tui`               | Terminal user interface              | ratatui + crossterm            | Secondary     |
| `bmp-common-ingredients`| Optional curated common items        | Static data                    | Optional      |
| `bmp-mobile`            | Mobile shell                         | Dioxus / Tauri / bridge        | Secondary     |
| `bmp-server`            | Optional future server               | axum / tonic                   | Secondary     |

### 1.3.3 Hardware
- Linux / macOS / Windows laptop (primary development and desktop testing)
- Android phone / tablet (mobile testing – secondary)

## 1.4 Standards

### 1.4.1 Coding
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Enforce `rustfmt` and `clippy` (pedantic where practical)
- Prefer explicit error handling with `Result`; avoid `unwrap`/`expect` in library code
- Document public APIs with rustdoc
- Domain crate must remain dependency-free of I/O

### 1.4.2 Design
Prototype → test → refine. Domain logic is written and unit-tested first. Integration tests exercise the storage and services layers. UI crates are thin adapters over the services layer.

### 1.4.3 Architecture
- Multi-crate Cargo workspace
- Pure domain model (single source of truth)
- Repository pattern for persistence
- Application services / use-case layer
- Event-driven message passing between front-end and services
- State-driven in-memory UI rendering with cached view-model state (no synchronous DB queries in high-frequency `render()` passes)
- Native programmatic window dialog management (`window.open_dialog` via root overlay layers)
- Unidirectional data flow
- Local-first; server is explicitly secondary

---

# 2.0 Core Requirements

## 2.1 Ingredients, Recipes & Meal Planning
1. The system shall allow users to create, edit, and delete Items (ingredients).
2. The system shall allow users to add Packages (price/quantity/unit) to Items at specific Stores.
3. The system shall allow users to create, edit, and delete Recipes consisting of ingredients (Items or other Recipe yields), instructions, and one or more yields.
4. The system shall allow users to add existing Recipe yields as ingredients within other Recipes, automatically scaling according to the parent Recipe’s required yield.
5. The system shall perform cycle detection at Recipe creation time and set cycle flags on ingredient edges as needed. Expansion shall terminate correctly when a cycle flag is present (critical for starters such as sourdough or yogurt).
6. The system shall allow users to create PrePlannedMeals from Recipes and/or raw Items.
7. The system shall allow users to schedule meals (from PrePlannedMeals, one-offs, or restaurant meals) on a calendar.
8. The system shall support a “Make Recipe” pre-configuration flow that lets the user set batch count (scaling), choose substitutes, include/exclude optional ingredients, and select which yield Item(s) will be produced.
9. The system shall support optional ingredients and substitutes (global preference + per-recipe override).

## 2.2 Shopping List & Inventory Management
1. The system shall automatically generate a shopping list based on scheduled meals, dynamically resolving nested Recipes down to base Items, applying the buy-finished vs expand preference, subtracting Pantry quantities, rounding up to whole packages, and selecting the best price.
2. The system shall allow users to manually move items from one store to another.
3. The system shall allow users to manually add independent Items or non-ingredient custom items to the shopping list.
4. The system shall allow users to mark items on the shopping list as purchased.
5. The system shall track already-owned Items derived from purchased shopping-list entries in a Pantry (ItemId + Quantity + optional expiration).
6. The system shall allow users to define bridge conversions between units. Bridges are applied on-the-fly; density is always normalized to g/ml.
7. The system shall automatically perform standard mass and volume conversions (US Customary, Metric, SI) without user configuration.
8. The system shall automatically decrement Pantry quantities upon user confirmation that a meal was consumed (form auto-filled with expected values; user may edit).
9. The system shall allow users to manually adjust Pantry quantities independently of the shopping list.
10. Shopping is per-store: user selects a store, the list is filtered, items can be checked off or added, and a confirmation form captures the actual total (or subtotal).

## 2.3 Financials & Analytics
1. The system shall calculate the subtotal (and optional total with sales tax) of the Shopping List based on selected Package prices.
2. The system shall allow users to input a final receipt total after shopping and calculate the difference between projected and actual costs.
3. When projected and actual costs differ, the system shall offer the user the option to update Package prices.
4. The system shall provide Analytics views for Daily, Weekly, Monthly, Annual, and Custom cost breakdowns.
5. The system shall calculate and display:
   - Price per batch (Recipe, including nested, respecting buy-finished vs expand toggle)
   - Price per serving
6. The system shall display projected vs actual costs for the supported time ranges.
7. Sales tax rate is optional; if unset, only subtotals are shown.

## 2.4 System Architecture & Notifications
1. The system shall persist all user data locally using an SQLite database.
2. The system shall trigger a notification (default 30 minutes after scheduled meal time) prompting the user to verify whether the meal was consumed. The notification is fire-and-forget; later confirmation is handled correctly.
3. The primary product (`bmp-local`) shall run fully offline and require no network access or user accounts.
4. All core domain logic shall reside in a pure, I/O-free crate (`bmp-domain`).
5. Front-end and services communicate via straightforward, low-overhead event-driven message passing (commands / queries → view-models / events).
6. The desktop UI (`bmp-local`) shall render dialog overlays programmatically using native window management (`window.open_dialog`) and explicit root view layer composition (`Root::render_dialog_layer`), bypassing inline view tree modal wrappers.
7. The desktop UI views shall maintain cached struct fields for domain entities and render strictly from memory in `Render::render()`, reloading state asynchronously or via mutation listeners (`reload_data()`). No database disk queries or recursive cost calculations shall execute inside `render()`.

---

# 3.0 Stretch Requirements

## 3.1 Advanced Inventory Features
1. Sort Items by category or alphabetically.
2. Sort Recipes alphabetically or by meal type.
3. “What can I make now” view based on current Pantry.
4. “Can make now” filter for Recipes and PrePlannedMeals.
5. Dashboard shortcut to available Recipes/Meals.

## 3.2 Advanced Input Methods
1. Input/update Items via receipt scanning (OCR).
2. Scan physical recipes to add them to the database.
3. Barcode support on Packages (scan to update price/last_seen).

## 3.3 UI/UX & Customization
1. Customize the dashboard.
2. Customize the app theme.
3. Configure the delay time for the “Meal Consumed” notification.

## 3.4 Data & Financial Enhancements
1. Export data to JSON for backup or transfer.
2. Cloud sync / peer-to-peer as future high-priority optional features (never required for core).

## 3.5 Advanced Meal Planning
1. Automatically calculate an ideal start time to cook a meal.
2. Trigger a notification reminding a user to start cooking.

## 3.6 Additional Front-ends & Server
1. Fully functional CLI (`bmp-cli`).
2. Fully functional TUI (`bmp-tui`).
3. Mobile shell (`bmp-mobile`) – best-effort.
4. Optional server crate (`bmp-server`) for future sync or shared recipe features (explicitly secondary).

---

# 4.0 Crate Architecture

```text
budget-meal-planner/                  # Cargo workspace
├── crates/
│   ├── bmp-domain/                   # Pure domain model & logic (single source of truth)
│   ├── bmp-storage/                  # SQLite repositories & migrations
│   ├── bmp-services/                 # Application / use-case layer
│   ├── bmp-local/                    # ★ Primary desktop application (GPUI preferred)
│   ├── bmp-common-ingredients/       # Optional static curated Items (can be disabled)
│   ├── bmp-cli/                      # CLI (secondary)
│   ├── bmp-tui/                      # Terminal UI (secondary)
│   ├── bmp-mobile/                   # Mobile shell (secondary)
│   └── bmp-server/                   # Optional server (secondary)
```

**Dependency direction (strict):**
- `bmp-domain` → nothing
- `bmp-storage` → `bmp-domain`
- `bmp-services` → `bmp-domain` + `bmp-storage`
- `bmp-common-ingredients` → `bmp-domain` (optional dependency)
- All front-end and server crates → `bmp-services` (and transitively the layers below)

---

# 5.0 Design Overview – Unified Domain Model

## 5.1 Core Types (Single Source of Truth)

```text
// Strongly-typed IDs
ItemId, RecipeId, PrePlannedMealId, ScheduledMealId, StoreId, PackageId, ...

Quantity { amount: Decimal, unit: Unit }

Unit  // Gram, Kilogram, Ounce, Pound, Milliliter, Liter,
      // Cup, Tbsp, Tsp, Each, and user-defined custom units.
      // Every mass unit implements to_grams().
      // Every volume unit implements to_ml().

Density  // Always stored normalized as g/ml.

PurchaseMode  // BuyFinished | PreferMake | AskEveryTime

Item {
    id: ItemId,
    name: String,
    density: Option<Density>,          // None → many calculations disabled
    preferred_purchase_mode: PurchaseMode,
    category: Option<String>,
    // optional mass-per-each bridge for count units
}

Package {
    id: PackageId,
    item_id: ItemId,
    store_id: StoreId,
    quantity: Quantity,
    price: Decimal,
    last_seen: Option<DateTime>,
    // barcode: Option<String>          // stretch
}

Store {
    id: StoreId,
    name: String,
    // additional metadata as needed
}

Recipe {
    id: RecipeId,
    name: String,
    yields: Vec<(ItemId, Quantity)>,   // one or more possible yields / variants
    ingredients: Vec<IngredientEdge>,
    instructions: String,
    servings: u32,                     // or Decimal
}

IngredientEdge {
    target: ItemOrRecipeId,            // Item or another Recipe’s yield
    quantity: Quantity,
    required: bool,                    // required vs optional
    cycle_flag: bool,                  // stops expansion (starters etc.)
    // substitute preferences live on Item or as per-recipe overrides
}

PrePlannedMeal {
    id: PrePlannedMealId,
    name: String,
    components: Vec<(RecipeId or ItemId, Quantity or servings)>,
}

ScheduledMeal {
    id: ScheduledMealId,
    source: PrePlannedMealId or one-off or Restaurant,
    datetime: DateTime,
    people: u32,
    consumed: bool,                    // or Option<DateTime>
    // restaurant meals carry only a cost (+ optional leftover yield)
}

PantryEntry {
    item_id: ItemId,
    quantity: Quantity,
    expiration: Option<Date>,
}
```

**Key rules**
- Density is always normalized to g/ml. Bridges are applied on-the-fly (not persisted as derived density).
- An Item may have zero or more Recipes. A Recipe declares the Item(s) it produces.
- Substitutes: global default + per-recipe override. Auto-considered for cost/shopping only when primary is missing/insufficient. Explicitly selectable on the Make Recipe screen.
- Best price = lowest price-per-unit-density after conversion to common base. User may pin a preferred Package that overrides it.
- Partial packages are always rounded up.
- Deletion: deleting an Item replaces references in Recipes with a “missing” flagged placeholder (original name preserved as string); the Recipe cannot be made until resolved. Deleting a Recipe does not delete the Item(s) it produces.

## 5.2 Useful Traits
- `HasDensity` / `Convertible`
- `Costable` (price-per-batch, price-per-serving)
- `Expandable` (nested recipe resolution, respecting cycle flags and purchase mode)
- `Purchaseable` (has packages)

## 5.3 Key Workflows

1. **Add Item (Ingredient)**  
   Create Item + optional density + one or more Packages + optional bridges (including mass-per-each).

2. **Add Recipe**  
   Ingredients (required/optional), instructions, one or more yields. Cycle detection runs at save time.

3. **Nest Recipe as Ingredient**  
   Reference another Recipe’s yield; quantities scale automatically.

4. **Create PrePlannedMeal**  
   Reusable template of Recipes and/or raw Items.

5. **Schedule Meal**  
   PrePlannedMeal, one-off, or restaurant meal + datetime + people count.

6. **Make Recipe (pre-configuration)**  
   Batches/scaling + substitutes + optionals + yield variant selection → then expand or consume.

7. **Go Shopping**  
   Generate list → select store → check off / add extras → finish → enter actual total → optional package updates.

8. **Toggle Buy-Finished vs Expand**  
   Per Item preference + per-line override on the shopping list.

9. **Manage Substitutes**  
   Global and/or per-recipe preferred substitutes.

10. **Confirm Meal Consumed**  
    Notification or manual → auto-filled decrement form → user confirms or edits → Pantry updated.

11. **Manage Stores**  
    Full CRUD.

12. **Manage Packages**  
    Add/edit (including last_seen); pin preferred package.

13. **Manual Pantry Adjustment**  
    Form-based quantity (and optional expiration) edit.

14. **Restaurant Meal**  
    Cost-only + optional leftover yield added to Pantry on consumption.

15. **Density / Bridge Management**  
    Supply bridges; system normalizes to g/ml on the fly. Missing density disables dependent calculations.

16. **View Analytics**  
    Daily / Weekly / Monthly / Annual / Custom projected vs actual, price-per-batch, price-per-serving.

## 5.4 Data Flow (Event-driven)
1. User interaction produces a Command / Message.
2. Front-end sends the command to the services layer.
3. Services call pure domain functions and storage repositories.
4. Storage persists changes and returns updated data.
5. Services produce a new View Model / State or Event.
6. Front-end renders the new state.  
Notifications are fire-and-forget; later confirmation is handled as a normal command.

## 5.5 Resources
- **Hardware:** Linux/macOS/Windows for development; desktop primary, Android secondary.
- **Software:** Pure Rust domain, SQLite storage, services layer, GPUI (preferred) for `bmp-local`.
- **Optional:** `bmp-common-ingredients` ships a static curated set; users may disable it and supply their own.

---

# 6.0 Verification

## 6.1 Demo
The demo shall exercise the core workflows (Add Item → Recipe → Nesting → PrePlannedMeal → Schedule → Make Recipe → Shopping → Pantry consumption → Analytics) using pre-populated data. Primary demo target is `bmp-local`.

## 6.2 Testing

All requirements in Section 2.0 have corresponding automated tests. Domain tests are pure unit + property-based (`proptest`). Storage tests use a temporary SQLite database. Services tests use in-memory or temp storage. Front-end tests are manual / snapshot / interaction as appropriate.

### 6.2.1 Explicit Core Tests (mapped to workflows)

**Add Item**
| Setup | Action | Success |
|-------|--------|---------|
| User is on Items view | Creates Item “Flour” with density | “Flour” appears; density stored as g/ml |
| Item has no density | Attempts cost or shopping calculation | Calculations for that Item are disabled until a bridge is supplied |
| Count-based Item | Adds mass-per-each bridge | Bridge applied on-the-fly for conversions |

**Add Recipe**
| Setup | Action | Success |
|-------|--------|---------|
| User is on Recipes view | Creates Recipe with ingredients, instructions, yield(s) | Recipe saved; yields correctly associated with Item(s) |
| Recipe with required + optional ingredients | Saves | Optional flag persisted; optional ingredients can be excluded later |
| Recipe produces multiple yield Items | Saves | All yields recorded |

**Nesting / Sub-Recipe**
| Setup | Action | Success |
|-------|--------|---------|
| Recipe A exists | Recipe B adds A’s yield as ingredient with quantity | Quantities scale correctly by parent yield |
| 3-level nesting | Expansion requested | Correctly resolves to base Items |
| Sourdough-style self-reference | Saves Recipe | Cycle flag set on edge; later expansion terminates and treats quantity as normal Item |
| Deep cycle (A→B→A) | Attempts to save | Detection runs; appropriate cycle flag(s) set |

**Create PrePlannedMeal**
| Setup | Action | Success |
|-------|--------|---------|
| Recipes and Items exist | Creates named PrePlannedMeal | Template appears and can be scheduled |
| Contains restaurant-style component | Saves | Cost-only component stored correctly |

**Schedule Meal**
| Setup | Action | Success |
|-------|--------|---------|
| PrePlannedMeal exists | Schedules with datetime + people | Appears on calendar |
| Restaurant meal | Schedules with cost only | Stored; optional leftover yield can later enter Pantry |
| People count = 4 | Later consumption | Scaling and pantry decrement respect the count |

**Make Recipe (pre-configuration)**
| Setup | Action | Success |
|-------|--------|---------|
| Recipe exists | Clicks “Make” | Pre-config screen appears |
| Sets batches = 2.5, chooses substitute, selects yield variant | Confirms | Correct scaled quantities, chosen substitute, and selected yield Item(s) produced |
| One Recipe yields multiple Items | Selects both | Both Items produced from single execution |

**Go Shopping**
| Setup | Action | Success |
|-------|--------|---------|
| Scheduled meals exist | Opens shopping list | List generated: nests expanded, Pantry subtracted, packages rounded up, best price selected, grouped by store |
| User selects Store “Walmart” | Enters shopping mode | Only Walmart items shown |
| While shopping | Adds extra item not on list | Extra item appears and is included in confirmation |
| Projected ≠ actual total | Finishes and enters actual | Optional package-price update flow offered |
| Preferred Package pinned | Generates list | Preferred Package used instead of computed best price |

**Toggle Buy-Finished vs Expand**
| Setup | Action | Success |
|-------|--------|---------|
| Item has both Packages and a Recipe | Views shopping list / cost | Default follows Item.preferred_purchase_mode |
| User overrides on a line | Toggles to Expand | That line expands to ingredients for the current list only |
| Toggle flipped | Cost view refreshed | Price-per-batch updates immediately |

**Manage Substitutes**
| Setup | Action | Success |
|-------|--------|---------|
| Two compatible Items | Sets global preferred substitute | Preference stored |
| Recipe ingredient | Sets per-recipe preferred substitute | Overrides global for that Recipe |
| Primary sufficient in Pantry | Generates shopping list / cost | Substitute ignored |
| Primary missing | Generates list | Substitute considered |
| Make Recipe screen | User forces a substitute | Substitute used for that execution only |
| No preference set | Uses Recipe | Original listed ingredient is used |

**Confirm Meal Consumed**
| Setup | Action | Success |
|-------|--------|---------|
| Scheduled meal time + 30 min | Notification fires | User can confirm |
| Confirmation form shown | Accepts defaults | Pantry decremented by expected quantities |
| User edits quantities on form | Commits | Edited quantities applied |
| Partial consumption | Edits some items lower | Only the edited amounts are decremented |

**Manage Stores**
| Setup | Action | Success |
|-------|--------|---------|
| User on Stores view | Creates / edits / deletes Store | Changes persisted |
| Store has Packages | Deletes Store | Packages handled according to documented cleanup rule (unassigned or cascade – exact rule stated in implementation) |

**Manage Packages**
| Setup | Action | Success |
|-------|--------|---------|
| Item + Store exist | Adds Package with qty, unit, price, last_seen | Package appears |
| Same Item + same Store | Adds second Package | Both packages coexist |
| User pins preferred Package | Generates shopping list | Preferred overrides best-price calculation |

**Manual Pantry Adjustment**
| Setup | Action | Success |
|-------|--------|---------|
| Item in Pantry | Edits quantity (and optional expiration) via form | New values persisted |
| Attempts negative quantity | Submits | Rejected with clear error |
| Expiration in the past | Saves | Allowed but flagged |

**Restaurant Meal**
| Setup | Action | Success |
|-------|--------|---------|
| User schedules restaurant meal | Supplies cost only | Stored correctly |
| Optional leftover yield | Confirms consumption | Leftover quantity added to Pantry |

**Density / Bridge Management**
| Setup | Action | Success |
|-------|--------|---------|
| User supplies bridge (e.g. 1 lb = 3.6 cups) | Saves | Applied on-the-fly; density normalized to g/ml |
| Item missing density | Any cost or shopping calculation | Disabled for that Item until usable bridge exists |
| Conflicting bridges | Supplies second path to g/ml | Most recent wins or clear error raised (rule documented) |

**Analytics**
| Setup | Action | Success |
|-------|--------|---------|
| History of purchases and consumptions exists | Opens Analytics | Daily/Weekly/Monthly/Annual/Custom views render projected vs actual |
| Nested Recipe + buy-finished toggle | Views cost | Price-per-batch and price-per-serving respect nesting and toggle |
| Sales tax rate set / unset | Views totals | Tax applied only when rate is present; otherwise subtotals only |

**Persistence & Architecture**
| Setup | Action | Success |
|-------|--------|---------|
| Any data created | App restarted | All data present (SQLite) |
| Domain functions called | Inspect dependencies | Domain crate has no I/O; only services talk to storage |
| User triggers modal dialog | Clicks action button | Modal opens via native `window.open_dialog` and renders on `Root::render_dialog_layer` |
| View `render()` passes fire | Inspect execution trace | View renders purely from struct-level cached memory state; 0 disk DB queries inside `render()` |

### 6.2.2 Stretch Tests
Tests for Section 3.0 features are required only when those features are implemented.

---

# 7.0 Resources

| Resource                          | URL / Notes |
|-----------------------------------|-------------|
| Rust API Guidelines               | https://rust-lang.github.io/api-guidelines/ |
| Rust Edition Guide                | https://doc.rust-lang.org/edition-guide/ |
| rusqlite                          | https://crates.io/crates/rusqlite |
| sqlx                              | https://crates.io/crates/sqlx |
| rust_decimal                      | https://crates.io/crates/rust_decimal |
| GPUI (Zed)                        | https://github.com/zed-industries/zed |
| ratatui                           | https://crates.io/crates/ratatui |
| Cargo Workspaces                  | https://doc.rust-lang.org/cargo/reference/workspaces.html |
| Original SRS v3 / v4              | Previous documents in project history |

---

**Document Status:** SRS v5 – Rust Multi-Crate Edition with Unified Density-Centric Domain Model  
**Primary Product:** `bmp-local` (local-first desktop application, GPUI preferred)  
**Core Principle:** Pure domain as single source of truth + explicit services + thin front-ends. Local-first remains non-negotiable; server, mobile, and cloud/P2P are secondary/future.  
**Key Change from v4:** Fully specified density model (g/ml normalized, on-the-fly bridges), Item/Recipe ownership and yield rules, substitute system, cycle-flag semantics, Make Recipe pre-configuration flow, per-store shopping, native window dialog management (`window.open_dialog`), state-driven cached in-memory UI rendering, and explicit tests covering every core workflow and the major edge cases.
