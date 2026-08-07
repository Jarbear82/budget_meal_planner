use crate::id::{ItemId, RecipeId};
use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum DomainError {
    #[error("Missing density for item {0}")]
    MissingDensity(ItemId),

    #[error("Incompatible unit conversion from {from} to {to}")]
    IncompatibleUnits { from: String, to: String },

    #[error("Cycle detected in recipe hierarchy involving recipe {0}")]
    RecipeCycleDetected(RecipeId),

    #[error("Negative quantity specified: {0}")]
    NegativeQuantity(Decimal),

    #[error("Item reference missing for recipe target: {0}")]
    MissingItemReference(ItemId),

    #[error("Invalid yield configuration for recipe {0}")]
    InvalidYield(RecipeId),

    #[error("Unit conversion bridge conflict for item {0}")]
    BridgeConflict(ItemId),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
