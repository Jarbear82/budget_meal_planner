use crate::error::ServiceResult;
use crate::event_bus::EventBus;
use bmp_domain::DomainEvent;
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
    event_bus: EventBus,
}

impl AnalyticsService {
    pub fn new(storage: Storage, event_bus: EventBus) -> Self {
        Self { storage, event_bus }
    }

    pub fn new_with_storage(storage: Storage) -> Self {
        Self::new(storage, EventBus::default())
    }

    pub fn record_receipt(
        &self,
        store_id: Option<bmp_domain::StoreId>,
        total: Decimal,
        datetime: DateTime<Utc>,
    ) -> ServiceResult<String> {
        let id = self.storage.insert_receipt(store_id, total, datetime)?;
        self.event_bus.publish(DomainEvent::ReceiptRecorded(id.clone()));
        Ok(id)
    }

    pub fn get_summary(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> ServiceResult<AnalyticsSummary> {
        let receipts = self.storage.get_all_receipts()?;
        let actual_expenditure: Decimal = receipts
            .into_iter()
            .filter(|(_, _, _, dt)| *dt >= start && *dt <= end)
            .map(|(_, _, total, _)| total)
            .sum();

        let shopping_service = crate::ShoppingService::new(self.storage.clone(), self.event_bus.clone());
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

    pub fn get_overall_summary(&self) -> ServiceResult<AnalyticsSummary> {
        let receipts = self.storage.get_all_receipts()?;
        let actual_expenditure: Decimal = receipts.into_iter().map(|(_, _, total, _)| total).sum();

        let shopping_service = crate::ShoppingService::new(self.storage.clone(), self.event_bus.clone());
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
