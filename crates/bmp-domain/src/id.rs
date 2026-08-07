use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(ItemId);
define_id!(RecipeId);
define_id!(StoreId);
define_id!(PackageId);
define_id!(PrePlannedMealId);
define_id!(ScheduledMealId);
define_id!(PantryEntryId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemOrRecipeId {
    Item(ItemId),
    Recipe(RecipeId),
}

impl fmt::Display for ItemOrRecipeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemOrRecipeId::Item(id) => write!(f, "Item({})", id),
            ItemOrRecipeId::Recipe(id) => write!(f, "Recipe({})", id),
        }
    }
}
