use crate::id::{ItemOrRecipeId, RecipeId};
use crate::recipe::Recipe;
use std::collections::{HashMap, HashSet};

/// Detects cycles in recipe dependencies and sets `cycle_flag = true` on any edge that creates a loop.
pub fn update_cycle_flags(recipes: &mut HashMap<RecipeId, Recipe>) {
    let recipe_ids: Vec<RecipeId> = recipes.keys().copied().collect();
    let mut global_visited = HashSet::new();
    let mut path_set = HashSet::new();
    let mut path_stack = Vec::new();

    for start_id in recipe_ids {
        if !global_visited.contains(&start_id) {
            dfs_cycle_check(
                start_id,
                recipes,
                &mut global_visited,
                &mut path_set,
                &mut path_stack,
            );
        }
    }
}

fn dfs_cycle_check(
    current_id: RecipeId,
    recipes: &mut HashMap<RecipeId, Recipe>,
    global_visited: &mut HashSet<RecipeId>,
    path_set: &mut HashSet<RecipeId>,
    path_stack: &mut Vec<RecipeId>,
) {
    if path_set.contains(&current_id) {
        // Cycle detected! Mark cycle_flag on all edges participating in the cycle
        if let Some(start_idx) = path_stack.iter().position(|&id| id == current_id) {
            let cycle_nodes = &path_stack[start_idx..];
            for i in 0..cycle_nodes.len() {
                let from_id = cycle_nodes[i];
                let to_id = if i + 1 < cycle_nodes.len() {
                    cycle_nodes[i + 1]
                } else {
                    current_id
                };
                if let Some(parent) = recipes.get_mut(&from_id) {
                    for edge in &mut parent.ingredients {
                        if edge.target == ItemOrRecipeId::Recipe(to_id) {
                            edge.cycle_flag = true;
                        }
                    }
                }
            }
        }
        return;
    }

    if global_visited.contains(&current_id) {
        return;
    }

    path_set.insert(current_id);
    path_stack.push(current_id);

    if let Some(recipe) = recipes.get(&current_id).cloned() {
        for edge in &recipe.ingredients {
            if let ItemOrRecipeId::Recipe(sub_id) = edge.target {
                dfs_cycle_check(sub_id, recipes, global_visited, path_set, path_stack);
            }
        }
    }

    path_stack.pop();
    path_set.remove(&current_id);
    global_visited.insert(current_id);
}
