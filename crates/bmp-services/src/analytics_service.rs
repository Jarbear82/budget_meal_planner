use bmp_storage::Storage;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct AnalyticsSummary {
    pub projected_cost: Decimal,
    pub actual_expenditure: Decimal,
    pub variance: Decimal,
}

pub struct AnalyticsService {
    storage: Storage,
}

impl AnalyticsService {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn record_receipt(
        &self,
        store_id: Option<bmp_domain::StoreId>,
        total: Decimal,
        datetime: DateTime<Utc>,
    ) -> Result<String, String> {
        self.storage.insert_receipt(store_id, total, datetime).map_err(|e| e.to_string())
    }

    pub fn get_summary(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<AnalyticsSummary, String> {
        let receipts = self.storage.get_all_receipts().map_err(|e| e.to_string())?;
        let actual_expenditure: Decimal = receipts
            .into_iter()
            .filter(|(_, _, _, dt)| *dt >= start && *dt <= end)
            .map(|(_, _, total, _)| total)
            .sum();

        let shopping_service = crate::ShoppingService::new(self.storage.clone());
        let projected_cost = match shopping_service.generate_shopping_list(Vec::new(), None, None) {
            Ok(list) => list.total,
            Err(_) => Decimal::ZERO,
        };

        let variance = actual_expenditure - projected_cost;

        Ok(AnalyticsSummary {
            projected_cost,
            actual_expenditure,
            variance,
        })
    }

    pub fn get_overall_summary(&self) -> Result<AnalyticsSummary, String> {
        let receipts = self.storage.get_all_receipts().map_err(|e| e.to_string())?;
        let actual_expenditure: Decimal = receipts.into_iter().map(|(_, _, total, _)| total).sum();

        let shopping_service = crate::ShoppingService::new(self.storage.clone());
        let projected_cost = match shopping_service.generate_shopping_list(Vec::new(), None, None) {
            Ok(list) => list.total,
            Err(_) => Decimal::ZERO,
        };

        let variance = actual_expenditure - projected_cost;

        Ok(AnalyticsSummary {
            projected_cost,
            actual_expenditure,
            variance,
        })
    }
}
