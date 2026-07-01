use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use stock_vision_data_model::*;
use tracing::info;

use crate::{AdjustType, DataSource};

/// Finnhub data source for US stocks.
/// 
/// Requires FINNHUB_API_KEY environment variable to be set.
/// Provides K-line data, search, financial reports, and real-time quotes.
/// Falls back gracefully if API key is not configured.
/// 
/// Free tier: 60 requests/minute.
pub struct FinnhubSource {
    client: Option<finnhub::FinnhubClient>,
}

impl FinnhubSource {
    pub fn new() -> Self {
        let client = match std::env::var("FINNHUB_API_KEY") {
            Ok(key) if !key.is_empty() && key != "your_key_here" => {
                info!("FinnhubSource: API key found, enabling Finnhub");
                let mut config = finnhub::ClientConfig::default();
                config.rate_limit_strategy = finnhub::RateLimitStrategy::FifteenSecondWindow;
                Some(finnhub::FinnhubClient::with_config(key, config))
            }
            _ => {
                info!("FinnhubSource: no FINNHUB_API_KEY set, disabled");
                None
            }
        };
        Self { client }
    }

    /// Check if Finnhub is available (API key configured)
    pub fn is_available(&self) -> bool {
        self.client.is_some()
    }

    /// Get finnhub client reference
    fn c(&self) -> Result<&finnhub::FinnhubClient, anyhow::Error> {
        self.client.as_ref().ok_or_else(|| anyhow::anyhow!("Finnhub API key not configured"))
    }

    /// Map Finnhub exchange codes to our Exchange enum
    fn map_exchange(exchange_str: &str) -> Option<Exchange> {
        match exchange_str {
            "NYSE" | "NYQ" | "NYE" => Some(Exchange::NYSE),
            "NASDAQ" | "NAS" | "NMS" | "NCM" => Some(Exchange::NASDAQ),
            "SS" | "SHH" => Some(Exchange::SH),
            "SZ" | "SZN" | "SHE" => Some(Exchange::SZ),
            _ => None,
        }
    }
}

#[async_trait]
impl DataSource for FinnhubSource {
    async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>> {
        let client = self.c()?;
        info!("Finnhub search: {}", keyword);

        let result = client.misc().symbol_search(keyword, None).await?;

        let stocks: Vec<Stock> = result.result.into_iter()
            .filter_map(|item| {
                // Finnhub returns type like "Common Stock", "ETF" etc.
                // Exchange info comes from the symbol itself
                let exchange = Self::map_exchange(&item.security_type)?;
                Some(Stock {
                    code: item.symbol,
                    name: item.description,
                    exchange,
                    sector: None,
                    industry: None,
                    list_date: None,
                    total_shares: None,
                    float_shares: None,
                })
            })
            .collect();

        Ok(stocks)
    }

    async fn get_daily_bars(
        &self,
        code: &str,
        exchange: Exchange,
        _start_date: Option<NaiveDate>,
        _end_date: Option<NaiveDate>,
        _adjust: Option<AdjustType>,
    ) -> anyhow::Result<Vec<DailyBar>> {
        let client = self.c()?;

        // Only support US stocks
        if !exchange.is_us() {
            return Ok(Vec::new());
        }

        // Finnhub candles: from=unix seconds, to=unix seconds
        let to = Utc::now().timestamp();
        let from = to - 86400 * 365 * 5; // 5 years of data

        info!("Finnhub daily K-line: {} ({})", code, exchange.prefix());

        use finnhub::models::stock::CandleResolution;
        let resolution = CandleResolution::Daily;
        let candles = client.stock().candles(code, resolution, from, to).await?;

        if candles.status != "ok" {
            return Ok(Vec::new());
        }

        let bars: Vec<DailyBar> = candles.timestamp.iter().enumerate()
            .filter_map(|(i, &ts)| {
                let open = candles.open.get(i).copied().unwrap_or(0.0);
                let high = candles.high.get(i).copied().unwrap_or(0.0);
                let low = candles.low.get(i).copied().unwrap_or(0.0);
                let close = candles.close.get(i).copied().unwrap_or(0.0);
                let volume = candles.volume.get(i).copied().unwrap_or(0.0);

                if open == 0.0 && high == 0.0 && low == 0.0 && close == 0.0 {
                    return None;
                }

                // Convert unix seconds to NaiveDate
                let date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()
                    + chrono::Duration::seconds(ts);

                Some(DailyBar {
                    code: code.to_string(),
                    date,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    amount: 0.0,
                    change_pct: None,
                })
            })
            .collect();

        Ok(bars)
    }

    async fn get_financial_reports(
        &self,
        code: &str,
        _exchange: Exchange,
        _report_type: Option<ReportType>,
    ) -> anyhow::Result<Vec<FinancialReport>> {
        let client = self.c()?;

        info!("Finnhub financials: {}", code);

        // Get basic financial metrics
        let metrics = client.stock().metrics(code).await?;

        // Get income statement (annual)
        use finnhub::models::stock::{StatementFrequency, StatementType};
        let income = client.stock()
            .financials(code, StatementType::IncomeStatement, StatementFrequency::Annual)
            .await?;

        let mut reports: Vec<FinancialReport> = Vec::new();

        // Parse financial statements
        for fin in income.financials {
            let report_date_str = fin.get("endDate")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let report_date = NaiveDate::parse_from_str(report_date_str, "%Y-%m-%d").ok()
                .unwrap_or_else(|| Utc::now().date_naive());

            let revenue = fin.get("revenue").and_then(|v| v.as_f64());
            let net_profit = fin.get("netIncome").and_then(|v| v.as_f64());
            let eps = fin.get("basicEarningsPerShare").and_then(|v| v.as_f64());
            let operating_cf = fin.get("operatingCashFlow").and_then(|v| v.as_f64());
            let total_assets = fin.get("totalAssets").and_then(|v| v.as_f64());
            let total_liabilities = fin.get("totalLiabilities").and_then(|v| v.as_f64());
            let equity = fin.get("totalShareholderEquity").and_then(|v| v.as_f64());

            // Determine report type from month
            let report_type = match report_date.month() {
                1..=3 => ReportType::Q1,
                4..=6 => ReportType::Mid,
                7..=9 => ReportType::Q3,
                10..=12 => ReportType::Annual,
                _ => ReportType::Annual,
            };

            let roe = equity.and_then(|e| net_profit.map(|np| if e != 0.0 { np / e * 100.0 } else { 0.0 }));
            let gross_margin = revenue.and_then(|r| {
                fin.get("grossProfit").and_then(|v| v.as_f64())
                    .map(|gp| if r != 0.0 { gp / r * 100.0 } else { 0.0 })
            });
            let net_margin = revenue.and_then(|r| net_profit.map(|np| if r != 0.0 { np / r * 100.0 } else { 0.0 }));

            reports.push(FinancialReport {
                code: code.to_string(),
                report_date,
                report_type,
                revenue,
                net_profit,
                operating_profit: None,
                total_assets,
                total_liabilities,
                equity,
                cash_equivalent: None,
                operating_cf,
                eps,
                bvps: None,
                roe,
                gross_margin,
                net_margin,
            });
        }

        // If no reports, create a basic one from metrics
        if reports.is_empty() {
            let eps_val = metrics.metric.get("epsTTM").and_then(|v| v.as_f64());
            let pe = metrics.metric.get("peTTM").and_then(|v| v.as_f64());
            let pb = metrics.metric.get("pbQuarterly").and_then(|v| v.as_f64());

            if eps_val.is_some() || pe.is_some() || pb.is_some() {
                reports.push(FinancialReport {
                    code: code.to_string(),
                    report_date: Utc::now().date_naive(),
                    report_type: ReportType::Annual,
                    revenue: metrics.metric.get("revenuePerShare").and_then(|v| v.as_f64()),
                    net_profit: eps_val,
                    operating_profit: None,
                    total_assets: metrics.metric.get("totalAssets").and_then(|v| v.as_f64()),
                    total_liabilities: metrics.metric.get("currentLiabilities").and_then(|v| v.as_f64()),
                    equity: metrics.metric.get("shareholdersEquity").and_then(|v| v.as_f64()),
                    cash_equivalent: metrics.metric.get("cash").and_then(|v| v.as_f64()),
                    operating_cf: metrics.metric.get("operatingCashFlowTTM").and_then(|v| v.as_f64()),
                    eps: eps_val,
                    bvps: metrics.metric.get("bookValuePerShare").and_then(|v| v.as_f64()),
                    roe: metrics.metric.get("roeTTM").and_then(|v| v.as_f64()),
                    gross_margin: metrics.metric.get("grossMargin").and_then(|v| v.as_f64()),
                    net_margin: metrics.metric.get("netMargin").and_then(|v| v.as_f64()),
                });
            }
        }

        Ok(reports)
    }

    async fn get_valuation_ratios(
        &self,
        code: &str,
        _exchange: Exchange,
    ) -> anyhow::Result<ValuationRatios> {
        let client = self.c()?;

        info!("Finnhub valuation: {}", code);

        // Get metrics
        let metrics = client.stock().metrics(code).await?;

        Ok(ValuationRatios {
            code: code.to_string(),
            date: Utc::now().date_naive(),
            pe: metrics.metric.get("peTTM").and_then(|v| v.as_f64()),
            pb: metrics.metric.get("pbQuarterly").and_then(|v| v.as_f64()),
            ps: metrics.metric.get("psTTM").and_then(|v| v.as_f64()),
            pcf: None,
            market_cap: metrics.metric.get("marketCapitalization").and_then(|v| v.as_f64()),
            dividend_yield: metrics.metric.get("dividendYieldIndicatedAnnual").and_then(|v| v.as_f64()),
        })
    }
}
