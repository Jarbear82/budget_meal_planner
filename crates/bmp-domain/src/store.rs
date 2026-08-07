use crate::id::StoreId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Store {
    pub id: StoreId,
    pub name: String,
}

impl Store {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: StoreId::new(),
            name: name.into(),
        }
    }
}
