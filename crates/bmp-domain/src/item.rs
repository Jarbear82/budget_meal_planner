use crate::bridge::UnitBridge;
use crate::density::Density;
use crate::id::ItemId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DietaryFlag {
    GlutenFree,
    DairyFree,
    KetoFriendly,
    Carnivore,
    Vegetarian,
    Vegan,
    NutFree,
    LowFodmap,
}

impl DietaryFlag {
    pub fn as_str(&self) -> &'static str {
        match self {
            DietaryFlag::GlutenFree => "Gluten-Free",
            DietaryFlag::DairyFree => "Dairy-Free",
            DietaryFlag::KetoFriendly => "Keto-Friendly",
            DietaryFlag::Carnivore => "Carnivore",
            DietaryFlag::Vegetarian => "Vegetarian",
            DietaryFlag::Vegan => "Vegan",
            DietaryFlag::NutFree => "Nut-Free",
            DietaryFlag::LowFodmap => "Low-FODMAP",
        }
    }

    pub fn all() -> &'static [DietaryFlag] {
        &[
            DietaryFlag::GlutenFree,
            DietaryFlag::DairyFree,
            DietaryFlag::KetoFriendly,
            DietaryFlag::Carnivore,
            DietaryFlag::Vegetarian,
            DietaryFlag::Vegan,
            DietaryFlag::NutFree,
            DietaryFlag::LowFodmap,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NutritionalInfo {
    pub calories: Option<Decimal>,
    pub protein_g: Option<Decimal>,
    pub net_carbs_g: Option<Decimal>,
    pub fat_g: Option<Decimal>,
    pub fiber_g: Option<Decimal>,
    pub sodium_mg: Option<Decimal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum PurchaseMode {
    #[default]
    BuyFinished,
    PreferMake,
    AskEveryTime,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub density: Option<Density>,
    pub preferred_purchase_mode: PurchaseMode,
    pub category: Option<String>,
    pub count_bridge: Option<UnitBridge>,
    pub nutrition: Option<NutritionalInfo>,
    pub dietary_flags: Vec<DietaryFlag>,
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
            nutrition: None,
            dietary_flags: Vec::new(),
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

    pub fn with_nutrition(mut self, nutrition: NutritionalInfo) -> Self {
        self.nutrition = Some(nutrition);
        self
    }

    pub fn with_dietary_flag(mut self, flag: DietaryFlag) -> Self {
        if !self.dietary_flags.contains(&flag) {
            self.dietary_flags.push(flag);
        }
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
        if let Some(ref bridge) = self.count_bridge
            && let Ok(bridged) = bridge.convert(qty, target_unit, self.density) {
                return Ok(bridged);
            }
        if let Some(ref density) = self.density
            && let Ok(dense) = density.convert(qty, target_unit) {
                return Ok(dense);
            }
        Err(crate::error::DomainError::IncompatibleUnits {
            from: format!("{}", qty.unit),
            to: format!("{}", target_unit),
        })
    }
}
