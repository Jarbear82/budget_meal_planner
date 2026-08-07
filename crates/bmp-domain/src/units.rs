use crate::error::DomainError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    Mass,
    Volume,
    Count,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unit {
    // Mass
    Gram,
    Kilogram,
    Ounce,
    Pound,
    // Volume
    Milliliter,
    Liter,
    Cup,
    Tablespoon,
    Teaspoon,
    // Count
    Each,
    // Custom
    Custom(String),
}

impl Unit {
    pub fn unit_type(&self) -> UnitType {
        match self {
            Unit::Gram | Unit::Kilogram | Unit::Ounce | Unit::Pound => UnitType::Mass,
            Unit::Milliliter | Unit::Liter | Unit::Cup | Unit::Tablespoon | Unit::Teaspoon => {
                UnitType::Volume
            }
            Unit::Each => UnitType::Count,
            Unit::Custom(_) => UnitType::Custom,
        }
    }

    /// Converts mass units to Grams.
    pub fn to_grams(&self, amount: Decimal) -> Result<Decimal, DomainError> {
        match self {
            Unit::Gram => Ok(amount),
            Unit::Kilogram => Ok(amount * dec!(1000)),
            Unit::Ounce => Ok(amount * dec!(28.349523125)),
            Unit::Pound => Ok(amount * dec!(453.59237)),
            _ => Err(DomainError::IncompatibleUnits {
                from: format!("{:?}", self),
                to: "Gram".to_string(),
            }),
        }
    }

    /// Converts Grams to target mass unit.
    pub fn from_grams(&self, grams: Decimal) -> Result<Decimal, DomainError> {
        match self {
            Unit::Gram => Ok(grams),
            Unit::Kilogram => Ok(grams / dec!(1000)),
            Unit::Ounce => Ok(grams / dec!(28.349523125)),
            Unit::Pound => Ok(grams / dec!(453.59237)),
            _ => Err(DomainError::IncompatibleUnits {
                from: "Gram".to_string(),
                to: format!("{:?}", self),
            }),
        }
    }

    /// Converts volume units to Milliliters.
    pub fn to_ml(&self, amount: Decimal) -> Result<Decimal, DomainError> {
        match self {
            Unit::Milliliter => Ok(amount),
            Unit::Liter => Ok(amount * dec!(1000)),
            Unit::Cup => Ok(amount * dec!(236.5882365)),
            Unit::Tablespoon => Ok(amount * dec!(14.78676478125)),
            Unit::Teaspoon => Ok(amount * dec!(4.92892159375)),
            _ => Err(DomainError::IncompatibleUnits {
                from: format!("{:?}", self),
                to: "Milliliter".to_string(),
            }),
        }
    }

    /// Converts Milliliters to target volume unit.
    pub fn from_ml(&self, ml: Decimal) -> Result<Decimal, DomainError> {
        match self {
            Unit::Milliliter => Ok(ml),
            Unit::Liter => Ok(ml / dec!(1000)),
            Unit::Cup => Ok(ml / dec!(236.5882365)),
            Unit::Tablespoon => Ok(ml / dec!(14.78676478125)),
            Unit::Teaspoon => Ok(ml / dec!(4.92892159375)),
            _ => Err(DomainError::IncompatibleUnits {
                from: "Milliliter".to_string(),
                to: format!("{:?}", self),
            }),
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Gram => write!(f, "g"),
            Unit::Kilogram => write!(f, "kg"),
            Unit::Ounce => write!(f, "oz"),
            Unit::Pound => write!(f, "lb"),
            Unit::Milliliter => write!(f, "ml"),
            Unit::Liter => write!(f, "L"),
            Unit::Cup => write!(f, "cup"),
            Unit::Tablespoon => write!(f, "tbsp"),
            Unit::Teaspoon => write!(f, "tsp"),
            Unit::Each => write!(f, "ea"),
            Unit::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quantity {
    pub amount: Decimal,
    pub unit: Unit,
}

impl Quantity {
    pub fn new(amount: Decimal, unit: Unit) -> Result<Self, DomainError> {
        if amount < Decimal::ZERO {
            return Err(DomainError::NegativeQuantity(amount));
        }
        Ok(Self { amount, unit })
    }

    pub fn zero(unit: Unit) -> Self {
        Self {
            amount: Decimal::ZERO,
            unit,
        }
    }

    /// Convert to same-category unit (mass <-> mass, volume <-> volume).
    pub fn convert_direct(&self, target_unit: &Unit) -> Result<Quantity, DomainError> {
        if &self.unit == target_unit {
            return Ok(self.clone());
        }

        match (self.unit.unit_type(), target_unit.unit_type()) {
            (UnitType::Mass, UnitType::Mass) => {
                let grams = self.unit.to_grams(self.amount)?;
                let target_amount = target_unit.from_grams(grams)?;
                Ok(Quantity {
                    amount: target_amount,
                    unit: target_unit.clone(),
                })
            }
            (UnitType::Volume, UnitType::Volume) => {
                let ml = self.unit.to_ml(self.amount)?;
                let target_amount = target_unit.from_ml(ml)?;
                Ok(Quantity {
                    amount: target_amount,
                    unit: target_unit.clone(),
                })
            }
            (UnitType::Count, UnitType::Count) => Ok(Quantity {
                amount: self.amount,
                unit: Unit::Each,
            }),
            _ => Err(DomainError::IncompatibleUnits {
                from: format!("{}", self.unit),
                to: format!("{}", target_unit),
            }),
        }
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount.normalize(), self.unit)
    }
}
