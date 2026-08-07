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

    pub fn record_receipt(&self, store_id: Option<bmp_domain::StoreId>, total: Decimal, datetime: DateTime<Utc>) -> Result<String, String> {
        self.storage.insert_receipt(store_id, total, datetime).map_err(|e| e.to_string())
    }

    pub fn get_summary(&self, _start: DateTime<Utc>, _end: DateTime<Utc>) -> Result<AnalyticsSummary, String> {
        Ok(AnalyticsSummary {
            projected_cost: Decimal::ZERO,
            actual_expenditure: Decimal::ZERO,
            variance: Decimal::ZERO,
        })
    }
}
