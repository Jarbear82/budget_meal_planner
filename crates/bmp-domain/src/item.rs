use crate::bridge::UnitBridge;
use crate::density::Density;
use crate::id::ItemId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PurchaseMode {
    BuyFinished,
    PreferMake,
    AskEveryTime,
}

impl Default for PurchaseMode {
    fn default() -> Self {
        PurchaseMode::BuyFinished
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub density: Option<Density>,
    pub preferred_purchase_mode: PurchaseMode,
    pub category: Option<String>,
    pub count_bridge: Option<UnitBridge>,
}

impl Item {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ItemId::new(),
            name: name.into(),
            density: None,
            preferred_purchase_mode: PurchaseMode::BuyFinished,
            category: None,
            count_bridge: None,
        }
    }

    pub fn with_density(mut self, density: Density) -> Self {
        self.density = Some(density);
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn with_purchase_mode(mut self, mode: PurchaseMode) -> Self {
        self.preferred_purchase_mode = mode;
        self
    }

    pub fn with_count_bridge(mut self, bridge: UnitBridge) -> Self {
        self.count_bridge = Some(bridge);
        self
    }

    pub fn convert_quantity(
        &self,
        qty: &crate::units::Quantity,
        target_unit: &crate::units::Unit,
    ) -> Result<crate::units::Quantity, crate::error::DomainError> {
        if qty.unit == *target_unit {
            return Ok(qty.clone());
        }
        if let Ok(direct) = qty.convert_direct(target_unit) {
            return Ok(direct);
        }
        if let Some(ref bridge) = self.count_bridge {
            if let Ok(bridged) = bridge.convert(qty, target_unit, self.density) {
                return Ok(bridged);
            }
        }
        if let Some(ref density) = self.density {
            if let Ok(dense) = density.convert(qty, target_unit) {
                return Ok(dense);
            }
        }
        Err(crate::error::DomainError::IncompatibleUnits {
            from: format!("{}", qty.unit),
            to: format!("{}", target_unit),
        })
    }
}
