use stock_vision_data_model::*;
use stock_vision_data_source::{AdjustType, DataSource, FallbackSource, TencentSource, EastMoneySource, YahooSource, FinnhubSource};
use stock_vision_storage::Storage;
use std::sync::{Arc, Mutex};
use tracing::info;

pub struct DataService {
    /// Primary data source for A-share (with fallback chain)
    source: FallbackSource,
    /// Data source for US stocks (Yahoo Finance - free, no key needed)
    us_source: YahooSource,
    /// Data source for US stocks (Finnhub - richer, needs API key, hot-swappable)
    finnhub_source: Mutex<FinnhubSource>,
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
            finnhub_source: Mutex::new(FinnhubSource::with_optional_key(stored_key.as_deref())),
            search_source: EastMoneySource::new(),
            fin_source: EastMoneySource::new(),
            storage,
        }
    }

    /// Whether Finnhub is configured and available (check env var + stored key)
    pub fn is_finnhub_available(&self) -> bool {
        // Check env var
        if let Ok(key) = std::env::var("FINNHUB_API_KEY") {
            if !key.is_empty() && key != "your_key_here" {
                return true;
            }
        }
        // Check stored key
        if let Some(key) = self.storage.get_config("finnhub_api_key") {
            if !key.is_empty() && key != "your_key_here" {
                return true;
            }
        }
        false
    }

    /// Set Finnhub API key dynamically — saves to SQLite + hot-swaps the client immediately.
    pub fn set_finnhub_api_key(&self, key: &str) {
        // Store in SQLite
        let _ = self.storage.set_config("finnhub_api_key", key);
        // Hot-swap the Finnhub client immediately (no restart needed)
        let mut finnhub = self.finnhub_source.lock().unwrap();
        *finnhub = FinnhubSource::with_optional_key(Some(key));
    }

    /// Get the current Finnhub API key (env var > SQLite)
    pub fn get_finnhub_api_key(&self) -> String {
        if let Ok(key) = std::env::var("FINNHUB_API_KEY") {
            if !key.is_empty() {
                return key;
            }
        }
        self.storage.get_config("finnhub_api_key").unwrap_or_default()
    }

    /// Select the best data source for a given exchange.
    /// Returns a Box since the source type varies at runtime.
    fn source_for_exchange(&self, exchange: &Exchange) -> Box<dyn DataSource> {
        if exchange.is_us() {
            let finnhub = self.finnhub_source.lock().unwrap();
            if finnhub.is_available() {
                // Drop lock, then create a fresh FinnhubSource with the key
                // (reqwest::Client is Arc internally so this is cheap)
                drop(finnhub);
                Box::new(FinnhubSource::new())
            } else {
                drop(finnhub);
                Box::new(YahooSource::new())
            }
        } else {
            Box::new(TencentSource::new())
        }
    }

    pub async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>> {
        // Try EastMoney first (A-share), then Finnhub (if available), then Yahoo (US stocks)
        match self.search_source.search_stocks(keyword).await {
            Ok(stocks) if !stocks.is_empty() => Ok(stocks),
            _ => {
                let has_finnhub = self.finnhub_source.lock().unwrap().is_available();
                if has_finnhub {
                    match FinnhubSource::new().search_stocks(keyword).await {
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
    pub async fn load_daily_bars(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<Vec<DailyBar>> {
        let is_us = exchange.is_us();
        let min_bars = if is_us { 10 } else { 100 };

        let cached = self.storage.get_daily_bars(code, 4000).unwrap_or_default();

        if cached.len() >= min_bars {
            info!("Using cached K-line for {} ({} bars)", code, cached.len());
            return Ok(cached);
        }

        info!("Cache miss for {} K-line, fetching from {}", 
            code, if is_us { "Yahoo/Finnhub" } else { "Tencent API" });
        
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
            let has_finnhub_cache = is_us && self.finnhub_source.lock().unwrap().is_available();
            let valuation = if is_us {
                if has_finnhub_cache {
                    FinnhubSource::new().get_valuation_ratios(code, exchange.clone()).await?
                } else {
                    self.fin_source.get_valuation_ratios(code, exchange.clone()).await?
                }
            } else {
                self.fin_source.get_valuation_ratios(code, exchange.clone()).await?
            };
            return Ok((cached, valuation));
        }

        // Fetch from API
        let (reports, valuation) = if is_us {
            let has_finnhub = self.finnhub_source.lock().unwrap().is_available();
            if has_finnhub {
                info!("Cache miss for {} financial reports, fetching from Finnhub", code);
                let fh = FinnhubSource::new();
                let r = fh.get_financial_reports(code, exchange.clone(), None).await?;
                let v = fh.get_valuation_ratios(code, exchange.clone()).await?;
                (r, v)
            } else {
                info!("Finnhub not available, financial reports for US stocks limited");
                (Vec::new(), self.fin_source.get_valuation_ratios(code, exchange.clone()).await?)
            }
        } else {
            info!("Cache miss for {} financial reports, fetching from EastMoney", code);
            let r = self.fin_source.get_financial_reports(code, exchange.clone(), None).await?;
            let v = self.fin_source.get_valuation_ratios(code, exchange.clone()).await?;
            (r, v)
        };

        if !reports.is_empty() {
            if let Err(e) = self.storage.save_financial_reports(&reports) {
                tracing::warn!("Failed to cache financial reports: {}", e);
            } else {
                info!("Cached {} financial reports for {}", reports.len(), code);
            }
        }

        Ok((reports, valuation))
    }

    /// Smart background sync: only fetches data not yet in cache.
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

        // Financial reports: only for A-share or if Finnhub available
        if !is_us || self.is_finnhub_available() {
            let cached_reports = self.storage.get_financial_reports(code).unwrap_or_default();
            if cached_reports.len() < 4 {
                info!("Background: fetching financial reports for {} (cached: {} reports)", code, cached_reports.len());
                let fin_source: Box<dyn DataSource> = if is_us {
                    Box::new(FinnhubSource::new())
                } else {
                    Box::new(EastMoneySource::new())
                };
                if let Ok(reports) = fin_source.get_financial_reports(code, exchange.clone(), None).await {
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
    pub async fn load_market_indices(&self) -> anyhow::Result<Vec<crate::state::MarketIndexData>> {
        let indices: Vec<(&str, &str, Exchange)> = vec![
            ("上证指数", "000001", Exchange::SH),
            ("深证成指", "399001", Exchange::SZ),
            ("创业板指", "399006", Exchange::SZ),
            ("科创50",   "000688", Exchange::SH),
        ];

        let mut result = Vec::new();
        for (name, code, exchange) in indices {
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
    pub async fn load_intraday_bars(
        &self,
        code: &str,
        exchange: Exchange,
        period: IntradayPeriod,
    ) -> anyhow::Result<Vec<IntradayBar>> {
        let cached = self.storage.get_intraday_bars(code, 4000).unwrap_or_default();
        if cached.len() >= 10 {
            info!("Using cached intraday {} for {} ({} bars)", period.tencent_param(), code, cached.len());
            return Ok(cached);
        }

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
