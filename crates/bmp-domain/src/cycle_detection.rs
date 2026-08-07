use crate::id::{ItemOrRecipeId, RecipeId};
use crate::recipe::Recipe;
use std::collections::{HashMap, HashSet};

/// Detects cycles in recipe dependencies and sets `cycle_flag = true` on any edge that creates a loop.
pub fn update_cycle_flags(recipes: &mut HashMap<RecipeId, Recipe>) {
    let recipe_ids: Vec<RecipeId> = recipes.keys().copied().collect();

    for start_id in recipe_ids {
        let mut visited = HashSet::new();
        let mut path_set = HashSet::new();
        let mut path_stack = Vec::new();
        dfs_cycle_check(
            start_id,
            recipes,
            &mut visited,
            &mut path_set,
            &mut path_stack,
        );
    }
}

fn dfs_cycle_check(
    current_id: RecipeId,
    recipes: &mut HashMap<RecipeId, Recipe>,
    visited: &mut HashSet<RecipeId>,
    path_set: &mut HashSet<RecipeId>,
    path_stack: &mut Vec<RecipeId>,
) {
    if path_set.contains(&current_id) {
        // Cycle detected! Mark cycle_flag on the edge leading to current_id in parent
        if let Some(&parent_id) = path_stack.last() {
            if let Some(parent) = recipes.get_mut(&parent_id) {
                for edge in &mut parent.ingredients {
                    if edge.target == ItemOrRecipeId::Recipe(current_id) {
                        edge.cycle_flag = true;
                    }
                }
            }
        }
        return;
    }

    if visited.contains(&current_id) {
        return;
    }

    visited.insert(current_id);
    path_set.insert(current_id);
    path_stack.push(current_id);

    if let Some(recipe) = recipes.get(&current_id).cloned() {
        for edge in &recipe.ingredients {
            if let ItemOrRecipeId::Recipe(sub_id) = edge.target {
                dfs_cycle_check(sub_id, recipes, visited, path_set, path_stack);
            }
        }
    }

    path_stack.pop();
    path_set.remove(&current_id);
}
