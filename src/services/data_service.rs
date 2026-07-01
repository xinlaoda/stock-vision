use stock_vision_data_model::*;
use stock_vision_data_source::{AdjustType, DataSource, FallbackSource, TencentSource, EastMoneySource, YahooSource, FinnhubSource};
use stock_vision_storage::Storage;
use std::sync::Arc;
use tracing::info;

pub struct DataService {
    /// Primary data source for A-share (with fallback chain)
    source: FallbackSource,
    /// Data source for US stocks (Yahoo Finance - free, no key needed)
    us_source: YahooSource,
    /// Data source for US stocks (Finnhub - richer, needs API key)
    finnhub_source: FinnhubSource,
    search_source: EastMoneySource,
    fin_source: EastMoneySource,
    storage: Arc<Storage>,
}

impl DataService {
    pub fn new(storage: Arc<Storage>) -> Self {
        // Fallback chain for A-share: try Tencent first, then EastMoney
        let source = FallbackSource::new(vec![
            Box::new(TencentSource::new()),
            Box::new(EastMoneySource::new()),
        ]);

        // Try loading Finnhub API key from storage (env var takes priority)
        let stored_key = storage.get_config("finnhub_api_key");

        Self {
            source,
            us_source: YahooSource::new(),
            finnhub_source: FinnhubSource::with_optional_key(stored_key.as_deref()),
            search_source: EastMoneySource::new(),
            fin_source: EastMoneySource::new(),
            storage,
        }
    }

    /// Whether Finnhub is configured and available
    pub fn is_finnhub_available(&self) -> bool {
        self.finnhub_source.is_available()
    }

    /// Set Finnhub API key dynamically (from settings UI).
    /// Stores in SQLite and re-initializes the Finnhub client.
    /// Environment variable FINNHUB_API_KEY takes priority.
    pub fn set_finnhub_api_key(&mut self, key: &str) {
        // Store in SQLite
        let _ = self.storage.set_config("finnhub_api_key", key);
        // Re-initialize Finnhub source with the stored key
        // with_optional_key handles priority: explicit key > env var > disabled
        self.finnhub_source = FinnhubSource::with_optional_key(Some(key));
    }

    /// Get the current Finnhub API key (from env var or SQLite)
    pub fn get_finnhub_api_key(&self) -> String {
        // Env var takes priority
        if let Ok(key) = std::env::var("FINNHUB_API_KEY") {
            if !key.is_empty() {
                return key;
            }
        }
        // Fall back to SQLite config
        self.storage.get_config("finnhub_api_key").unwrap_or_default()
    }

    /// Select data source based on exchange
    /// For US stocks: prefers Finnhub (richer data) if available, falls back to Yahoo
    fn source_for_exchange(&self, exchange: &Exchange) -> &dyn DataSource {
        if exchange.is_us() {
            if self.finnhub_source.is_available() {
                &self.finnhub_source as &dyn DataSource
            } else {
                &self.us_source as &dyn DataSource
            }
        } else {
            &self.source as &dyn DataSource
        }
    }

    pub async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>> {
        // Try EastMoney first (A-share), then Finnhub (if available), then Yahoo (US stocks)
        match self.search_source.search_stocks(keyword).await {
            Ok(stocks) if !stocks.is_empty() => Ok(stocks),
            _ => {
                if self.finnhub_source.is_available() {
                    match self.finnhub_source.search_stocks(keyword).await {
                        Ok(stocks) if !stocks.is_empty() => Ok(stocks),
                        _ => self.us_source.search_stocks(keyword).await
                    }
                } else {
                    self.us_source.search_stocks(keyword).await
                }
            }
        }
    }

    /// Load daily bars with SQLite cache.
    /// Returns cached data if available (>= 100 bars), otherwise fetches from API.
    /// For US stocks, skip cache threshold (fewer bars available) and use Yahoo source.
    pub async fn load_daily_bars(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<Vec<DailyBar>> {
        let is_us = exchange.is_us();
        let min_bars = if is_us { 10 } else { 100 };

        // Check local cache first
        let cached = self.storage.get_daily_bars(code, 4000).unwrap_or_default();

        if cached.len() >= min_bars {
            info!("Using cached K-line for {} ({} bars)", code, cached.len());
            return Ok(cached);
        }

        // Fetch from API (use different source for US vs A-share)
        info!("Cache miss for {} K-line, fetching from {}", 
            code, if is_us { "Yahoo Finance" } else { "Tencent API" });
        
        let source = self.source_for_exchange(&exchange);
        let bars = source
            .get_daily_bars(code, exchange.clone(), None, None, Some(AdjustType::Forward))
            .await?;

        if !bars.is_empty() {
            if let Err(e) = self.storage.save_daily_bars(&bars) {
                tracing::warn!("Failed to cache daily bars: {}", e);
            } else {
                info!("Cached {} K-line bars for {}", bars.len(), code);
            }
        }

        Ok(bars)
    }

    /// Load financial reports with SQLite cache.
    /// Returns cached data if available, otherwise fetches from API.
    /// Uses Finnhub for US stocks (if available), EastMoney for A-share.
    pub async fn load_financial_data(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<(Vec<FinancialReport>, ValuationRatios)> {
        let is_us = exchange.is_us();

        // Check local cache first
        let cached = self.storage.get_financial_reports(code).unwrap_or_default();
        if cached.len() >= 4 {
            info!("Using cached financial reports for {} ({} reports)", code, cached.len());
            let valuation = if is_us && self.finnhub_source.is_available() {
                self.finnhub_source.get_valuation_ratios(code, exchange.clone()).await?
            } else {
                self.fin_source.get_valuation_ratios(code, exchange.clone()).await?
            };
            return Ok((cached, valuation));
        }

        // Fetch from API
        let fin_source: &dyn DataSource = if is_us && self.finnhub_source.is_available() {
            info!("Cache miss for {} financial reports, fetching from Finnhub", code);
            &self.finnhub_source as &dyn DataSource
        } else {
            info!("Cache miss for {} financial reports, fetching from EastMoney", code);
            &self.fin_source as &dyn DataSource
        };

        let reports = fin_source.get_financial_reports(code, exchange.clone(), None).await?;

        if !reports.is_empty() {
            if let Err(e) = self.storage.save_financial_reports(&reports) {
                tracing::warn!("Failed to cache financial reports: {}", e);
            } else {
                info!("Cached {} financial reports for {}", reports.len(), code);
            }
        }

        let valuation = fin_source.get_valuation_ratios(code, exchange.clone()).await?;
        Ok((reports, valuation))
    }

    /// Smart background sync: only fetches data not yet in cache.
    /// Uses correct data source based on exchange (Yahoo for US, Tencent/EastMoney for A-share).
    pub async fn load_all(&self, code: &str, exchange: Exchange) -> anyhow::Result<()> {
        let is_us = exchange.is_us();
        let min_bars = if is_us { 10 } else { 100 };
        let mut fetched_anything = false;

        // Daily bars: check cache
        let cached_bars = self.storage.get_daily_bars(code, 4000).unwrap_or_default();
        if cached_bars.len() < min_bars {
            info!("Background: fetching K-line for {} (cached: {} bars)", code, cached_bars.len());
            let source = self.source_for_exchange(&exchange);
            if let Ok(bars) = source
                .get_daily_bars(code, exchange.clone(), None, None, Some(AdjustType::Forward))
                .await
            {
                if !bars.is_empty() {
                    let _ = self.storage.save_daily_bars(&bars);
                    fetched_anything = true;
                    info!("Background: cached {} K-line bars for {}", bars.len(), code);
                }
            }
        } else {
            info!("Background: K-line for {} already cached ({} bars), skipping", code, cached_bars.len());
        }

        // Financial reports: only for A-share (Yahoo doesn't support financial reports)
        if !is_us {
            let cached_reports = self.storage.get_financial_reports(code).unwrap_or_default();
            if cached_reports.len() < 4 {
                info!("Background: fetching financial reports for {} (cached: {} reports)", code, cached_reports.len());
                if let Ok(reports) = self.fin_source.get_financial_reports(code, exchange.clone(), None).await {
                    if !reports.is_empty() {
                        let _ = self.storage.save_financial_reports(&reports);
                        fetched_anything = true;
                        info!("Background: cached {} financial reports for {}", reports.len(), code);
                    }
                }
            } else {
                info!("Background: financial reports for {} already cached, skipping", code);
            }
        }

        if fetched_anything {
            info!("Background sync complete for {} ({})", code, exchange.prefix());
        } else {
            info!("Background sync: all data for {} already in cache, nothing to fetch", code);
        }
        Ok(())
    }

    /// Fetch real-time market indices data for the home page.
    /// Uses Tencent's realtime quote API + K-line API.
    pub async fn load_market_indices(&self) -> anyhow::Result<Vec<crate::state::MarketIndexData>> {
        let indices: Vec<(&str, &str, Exchange)> = vec![
            ("上证指数", "000001", Exchange::SH),
            ("深证成指", "399001", Exchange::SZ),
            ("创业板指", "399006", Exchange::SZ),
            ("科创50",   "000688", Exchange::SH),
        ];

        let mut result = Vec::new();
        for (name, code, exchange) in indices {
            // Get K-line data for sparkline
            let bars = self.source.get_daily_bars(code, exchange.clone(), None, None, Some(AdjustType::None)).await.unwrap_or_default();
            let bars_60: Vec<_> = bars.into_iter().rev().take(60).rev().collect();

            let (price, change, change_pct) = if let Some(last) = bars_60.last() {
                let first = bars_60.first().map(|b| b.close).unwrap_or(last.close);
                (last.close, last.close - first, (last.close - first) / first * 100.0)
            } else {
                (0.0, 0.0, 0.0)
            };

            result.push(crate::state::MarketIndexData {
                name: name.to_string(),
                code: code.to_string(),
                price,
                change,
                change_pct,
                bars: bars_60,
            });
        }

        info!("Loaded {} market indices", result.len());
        Ok(result)
    }

    /// Load intraday (minute-level) K-line bars with SQLite cache + fallback.
    /// Returns cached data if available (>= 10 bars), otherwise fetches from API
    /// (tries multiple sources with fallback).
    pub async fn load_intraday_bars(
        &self,
        code: &str,
        exchange: Exchange,
        period: IntradayPeriod,
    ) -> anyhow::Result<Vec<IntradayBar>> {
        // Check local cache first
        let cached = self.storage.get_intraday_bars(code, 4000).unwrap_or_default();
        if cached.len() >= 10 {
            info!("Using cached intraday {} for {} ({} bars)", period.tencent_param(), code, cached.len());
            return Ok(cached);
        }

        // Fetch from API with fallback
        info!("Cache miss for intraday {} {}, fetching from API", period.tencent_param(), code);
        let bars = self.source
            .get_intraday_bars(code, exchange, period)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch intraday data: {}", e))?;

        if !bars.is_empty() {
            if let Err(e) = self.storage.save_intraday_bars(&bars) {
                tracing::warn!("Failed to cache intraday bars: {}", e);
            } else {
                info!("Cached {} intraday bars for {} ({})", bars.len(), code, period.tencent_param());
            }
        }

        Ok(bars)
    }


    /// Load the latest daily bars (for realtime quote updates).
    /// Uses correct data source based on exchange.
    pub async fn load_latest_bars(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<Vec<DailyBar>> {
        let source = self.source_for_exchange(&exchange);
        let bars = source
            .get_daily_bars(code, exchange, None, None, Some(AdjustType::Forward))
            .await?;
        if !bars.is_empty() {
            let _ = self.storage.save_daily_bars(&bars);
        }
        Ok(bars)
    }
}