use stock_vision_data_model::*;
use stock_vision_data_source::{AdjustType, DataSource, TencentSource, EastMoneySource};
use stock_vision_storage::Storage;
use std::sync::Arc;
use tracing::info;

pub struct DataService {
    kline_source: TencentSource,
    search_source: EastMoneySource,
    storage: Arc<Storage>,
}

impl DataService {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            kline_source: TencentSource::new(),
            search_source: EastMoneySource::new(),
            storage,
        }
    }

    pub async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>> {
        self.search_source.search_stocks(keyword).await
    }

    /// Load daily bars with SQLite cache.
    /// Checks local cache first; if not enough data, fetches from API and caches.
    pub async fn load_daily_bars(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<Vec<DailyBar>> {
        // 1. Check local cache
        let cached = self.storage.get_daily_bars(code, 4000).unwrap_or_default();

        // If we have enough cached data (>= 200 bars), use it directly
        if cached.len() >= 100 {
            info!("Using cached data for {} ({} bars)", code, cached.len());
            return Ok(cached);
        }

        // 2. Fetch from API
        info!("Cache miss for {}, fetching from Tencent API", code);
        let bars = self.kline_source
            .get_daily_bars(code, exchange.clone(), None, None, Some(AdjustType::Forward))
            .await?;

        // 3. Save to cache
        if !bars.is_empty() {
            if let Err(e) = self.storage.save_daily_bars(&bars) {
                tracing::warn!("Failed to cache daily bars: {}", e);
            } else {
                info!("Cached {} bars for {}", bars.len(), code);
            }
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
