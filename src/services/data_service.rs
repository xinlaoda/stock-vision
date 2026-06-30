use stock_vision_data_model::*;
use stock_vision_data_source::{DataSource, TencentSource, EastMoneySource};
use stock_vision_storage::Storage;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DataService {
    kline_source: TencentSource,
    search_source: EastMoneySource,
    storage: Arc<RwLock<Storage>>,
}

impl DataService {
    pub fn new(storage: Arc<RwLock<Storage>>) -> Self {
        Self {
            kline_source: TencentSource::new(),
            search_source: EastMoneySource::new(),
            storage,
        }
    }

    pub async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>> {
        self.search_source.search_stocks(keyword).await
    }

    pub async fn load_daily_bars(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<Vec<DailyBar>> {
        let bars = self.kline_source.get_daily_bars(code, exchange.clone(), None, None, None).await?;
        if !bars.is_empty() {
            let storage = self.storage.write().await;
            storage.save_daily_bars(&bars)?;
        }
        Ok(bars)
    }

    pub async fn load_financial_data(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<(Vec<FinancialReport>, ValuationRatios)> {
        let reports = self.kline_source.get_financial_reports(code, exchange.clone(), None).await?;
        let valuation = self.kline_source.get_valuation_ratios(code, exchange.clone()).await?;
        Ok((reports, valuation))
    }
}
