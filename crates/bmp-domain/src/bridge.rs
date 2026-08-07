use crate::density::Density;
use crate::error::DomainError;
use crate::id::ItemId;
use crate::units::{Quantity, Unit};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitBridge {
    pub item_id: ItemId,
    pub from_qty: Quantity,
    pub to_qty: Quantity,
}

impl UnitBridge {
    pub fn new(item_id: ItemId, from_qty: Quantity, to_qty: Quantity) -> Result<Self, DomainError> {
        if from_qty.amount <= Decimal::ZERO || to_qty.amount <= Decimal::ZERO {
            return Err(DomainError::NegativeQuantity(from_qty.amount));
        }
        Ok(Self {
            item_id,
            from_qty,
            to_qty,
        })
    }

    /// Try converting a quantity using this bridge and optional item density.
    pub fn convert(
        &self,
        qty: &Quantity,
        target_unit: &Unit,
        density: Option<Density>,
    ) -> Result<Quantity, DomainError> {
        // Direct match with bridge source unit
        let from_scaled_amount = if qty.unit == self.from_qty.unit {
            qty.amount
        } else if let Ok(converted) = qty.convert_direct(&self.from_qty.unit) {
            converted.amount
        } else if let Some(d) = density {
            d.convert(qty, &self.from_qty.unit)?.amount
        } else {
            return Err(DomainError::IncompatibleUnits {
                from: format!("{}", qty.unit),
                to: format!("{}", self.from_qty.unit),
            });
        };

        let bridge_multiplier = from_scaled_amount / self.from_qty.amount;
        let target_base_qty = Quantity {
            amount: self.to_qty.amount * bridge_multiplier,
            unit: self.to_qty.unit.clone(),
        };

        if target_base_qty.unit == *target_unit {
            Ok(target_base_qty)
        } else if let Ok(direct) = target_base_qty.convert_direct(target_unit) {
            Ok(direct)
        } else if let Some(d) = density {
            d.convert(&target_base_qty, target_unit)
        } else {
            Err(DomainError::IncompatibleUnits {
                from: format!("{}", target_base_qty.unit),
                to: format!("{}", target_unit),
            })
        }
    }
}
