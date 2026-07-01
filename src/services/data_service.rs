use stock_vision_data_model::*;
use stock_vision_data_source::{AdjustType, DataSource, TencentSource, EastMoneySource};
use stock_vision_storage::Storage;
use std::sync::Arc;
use tracing::info;

pub struct DataService {
    kline_source: TencentSource,
    search_source: EastMoneySource,
    fin_source: EastMoneySource,
    storage: Arc<Storage>,
}

impl DataService {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            kline_source: TencentSource::new(),
            search_source: EastMoneySource::new(),
            fin_source: EastMoneySource::new(),
            storage,
        }
    }

    pub async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>> {
        self.search_source.search_stocks(keyword).await
    }

    /// Load daily bars with SQLite cache.
    pub async fn load_daily_bars(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<Vec<DailyBar>> {
        // 1. Check local cache
        let cached = self.storage.get_daily_bars(code, 4000).unwrap_or_default();

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

    /// Load financial reports (uses EastMoney, cached)
    pub async fn load_financial_data(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<(Vec<FinancialReport>, ValuationRatios)> {
        let reports = self.fin_source.get_financial_reports(code, exchange.clone(), None).await?;

        // Cache reports
        if !reports.is_empty() {
            if let Err(e) = self.storage.save_financial_reports(&reports) {
                tracing::warn!("Failed to cache financial reports: {}", e);
            }
        }

        let valuation = self.fin_source.get_valuation_ratios(code, exchange.clone()).await?;
        Ok((reports, valuation))
    }

    /// Load all data for a stock (background sync)
    pub async fn load_all(&self, code: &str, exchange: Exchange) -> anyhow::Result<()> {
        // Load and cache daily bars
        let _ = self.load_daily_bars(code, exchange.clone()).await;

        // Load and cache financial reports
        let _ = self.load_financial_data(code, exchange.clone()).await;

        info!("Background sync complete for {} ({})", code, exchange.prefix());
        Ok(())
    }
}
