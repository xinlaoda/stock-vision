pub mod eastmoney;
pub mod tencent;
pub mod mock;

use async_trait::async_trait;
use chrono::NaiveDate;
use stock_vision_data_model::*;

// Re-export structs so they can be `use`d from outside
pub use eastmoney::EastMoneySource;
pub use tencent::TencentSource;
pub use mock::MockSource;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdjustType {
    None,
    Forward,
    Backward,
}

#[async_trait]
pub trait DataSource: Send + Sync {
    async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>>;
    async fn get_daily_bars(
        &self,
        code: &str,
        exchange: Exchange,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        adjust: Option<AdjustType>,
    ) -> anyhow::Result<Vec<DailyBar>>;
    async fn get_financial_reports(
        &self,
        code: &str,
        exchange: Exchange,
        report_type: Option<ReportType>,
    ) -> anyhow::Result<Vec<FinancialReport>>;
    async fn get_valuation_ratios(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<ValuationRatios>;

    /// Get intraday/minute-level K-line data.
    /// Default implementation returns NotSupported error.
    async fn get_intraday_bars(
        &self,
        _code: &str,
        _exchange: Exchange,
        _period: IntradayPeriod,
    ) -> Result<Vec<IntradayBar>, DataSourceError> {
        Err(DataSourceError::NotSupported)
    }
}

/// A data source that tries multiple backends in order (fallback strategy).
/// Each source is tried in order; if one returns an error or empty data,
/// the next is attempted. This ensures redundancy since historical data
/// is the same across sources.
pub struct FallbackSource {
    sources: Vec<Box<dyn DataSource>>,
}

impl FallbackSource {
    pub fn new(sources: Vec<Box<dyn DataSource>>) -> Self {
        Self { sources }
    }
}

#[async_trait]
impl DataSource for FallbackSource {
    async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>> {
        let keyword = keyword.to_string();
        let mut last_err = None;
        for source in &self.sources {
            match source.search_stocks(&keyword).await {
                Ok(data) if !data.is_empty() => return Ok(data),
                Ok(_) => { last_err = None; continue; },
                Err(e) => { tracing::warn!("Source failed: {}", e); last_err = Some(e); }
            }
        }
        Err(last_err.unwrap_or_else(|| panic!("All data sources exhausted")))
    }

    async fn get_daily_bars(
        &self,
        code: &str,
        exchange: Exchange,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        adjust: Option<AdjustType>,
    ) -> anyhow::Result<Vec<DailyBar>> {
        let code = code.to_string();
        let mut last_err = None;
        for source in &self.sources {
            match source.get_daily_bars(&code, exchange.clone(), start_date, end_date, adjust).await {
                Ok(data) if !data.is_empty() => return Ok(data),
                Ok(_) => { last_err = None; continue; },
                Err(e) => { tracing::warn!("Source failed: {}", e); last_err = Some(e); }
            }
        }
        Err(last_err.unwrap_or_else(|| panic!("All data sources exhausted")))
    }

    async fn get_financial_reports(
        &self,
        code: &str,
        exchange: Exchange,
        report_type: Option<ReportType>,
    ) -> anyhow::Result<Vec<FinancialReport>> {
        let code = code.to_string();
        let mut last_err = None;
        for source in &self.sources {
            match source.get_financial_reports(&code, exchange.clone(), report_type.clone()).await {
                Ok(data) if !data.is_empty() => return Ok(data),
                Ok(_) => { last_err = None; continue; },
                Err(e) => { tracing::warn!("Source failed: {}", e); last_err = Some(e); }
            }
        }
        Err(last_err.unwrap_or_else(|| panic!("All data sources exhausted")))
    }

    async fn get_valuation_ratios(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<ValuationRatios> {
        let code = code.to_string();
        let mut last_err: Option<anyhow::Error> = None;
        for source in &self.sources {
            match source.get_valuation_ratios(&code, exchange.clone()).await {
                Ok(data) => return Ok(data),
                Err(e) => { tracing::warn!("Source failed: {}", e); last_err = Some(e); }
            }
        }
        Err(last_err.unwrap_or_else(|| panic!("All data sources exhausted")))
    }

    async fn get_intraday_bars(
        &self,
        code: &str,
        exchange: Exchange,
        period: IntradayPeriod,
    ) -> Result<Vec<IntradayBar>, DataSourceError> {
        let code = code.to_string();
        let mut last_err = None;
        for source in &self.sources {
            match source.get_intraday_bars(&code, exchange.clone(), period).await {
                Ok(data) if !data.is_empty() => return Ok(data),
                Ok(_) => { last_err = None; continue; },
                Err(e) => {
                    match &e {
                        DataSourceError::NotSupported => { /* try next */ },
                        _ => tracing::warn!("Source failed: {}", e),
                    }
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| panic!("All data sources exhausted")))
    }
}
