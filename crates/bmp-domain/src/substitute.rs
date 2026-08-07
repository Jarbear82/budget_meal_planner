use crate::id::ItemId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalSubstitute {
    pub primary: ItemId,
    pub substitute: ItemId,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubstituteResolver {
    pub global_substitutes: HashMap<ItemId, ItemId>,
}

impl SubstituteResolver {
    pub fn new() -> Self {
        Self {
            global_substitutes: HashMap::new(),
        }
    }

    pub fn set_global(&mut self, primary: ItemId, substitute: ItemId) {
        self.global_substitutes.insert(primary, substitute);
    }

    pub fn resolve(
        &self,
        primary: ItemId,
        per_recipe_substitute: Option<ItemId>,
        forced_override: Option<ItemId>,
    ) -> ItemId {
        if let Some(forced) = forced_override {
            return forced;
        }
        if let Some(per_recipe) = per_recipe_substitute {
            return per_recipe;
        }
        if let Some(&global) = self.global_substitutes.get(&primary) {
            return global;
        }
        primary
    }
}
