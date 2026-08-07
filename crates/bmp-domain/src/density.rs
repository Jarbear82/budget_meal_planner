use crate::error::DomainError;
use crate::units::{Quantity, Unit, UnitType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Density {
    /// Normalized density in grams per milliliter (g/ml).
    pub g_per_ml: Decimal,
}

impl Density {
    pub fn new(g_per_ml: Decimal) -> Result<Self, DomainError> {
        if g_per_ml <= Decimal::ZERO {
            return Err(DomainError::NegativeQuantity(g_per_ml));
        }
        Ok(Self { g_per_ml })
    }

    /// Convert mass quantity to volume quantity using density.
    pub fn mass_to_volume(&self, mass: &Quantity, target_unit: &Unit) -> Result<Quantity, DomainError> {
        let grams = mass.unit.to_grams(mass.amount)?;
        let ml = grams / self.g_per_ml;
        let target_amount = target_unit.from_ml(ml)?;
        Ok(Quantity {
            amount: target_amount,
            unit: target_unit.clone(),
        })
    }

    /// Convert volume quantity to mass quantity using density.
    pub fn volume_to_mass(&self, volume: &Quantity, target_unit: &Unit) -> Result<Quantity, DomainError> {
        let ml = volume.unit.to_ml(volume.amount)?;
        let grams = ml * self.g_per_ml;
        let target_amount = target_unit.from_grams(grams)?;
        Ok(Quantity {
            amount: target_amount,
            unit: target_unit.clone(),
        })
    }

    /// Convert between any mass and volume quantities.
    pub fn convert(&self, qty: &Quantity, target_unit: &Unit) -> Result<Quantity, DomainError> {
        match (qty.unit.unit_type(), target_unit.unit_type()) {
            (UnitType::Mass, UnitType::Mass) | (UnitType::Volume, UnitType::Volume) => {
                qty.convert_direct(target_unit)
            }
            (UnitType::Mass, UnitType::Volume) => self.mass_to_volume(qty, target_unit),
            (UnitType::Volume, UnitType::Mass) => self.volume_to_mass(qty, target_unit),
            _ => Err(DomainError::IncompatibleUnits {
                from: format!("{}", qty.unit),
                to: format!("{}", target_unit),
            }),
        }
    }
}
