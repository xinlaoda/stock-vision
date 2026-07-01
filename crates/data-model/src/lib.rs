/// Core data models for Stock Vision
/// Defines all domain types used across the application.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════
// Stock & Market Data
// ═══════════════════════════════════

/// Represents a stock listed on A-shares market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stock {
    pub code: String,          // e.g. "000001"
    pub name: String,          // e.g. "平安银行"
    pub exchange: Exchange,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub list_date: Option<NaiveDate>,
    pub total_shares: Option<f64>,
    pub float_shares: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Exchange {
    SZ, // Shenzhen
    SH, // Shanghai
    BJ, // Beijing
}

impl Exchange {
    pub fn prefix(&self) -> &str {
        match self {
            Exchange::SZ => "SZ",
            Exchange::SH => "SH",
            Exchange::BJ => "BJ",
        }
    }
}

/// Daily K-line data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyBar {
    pub code: String,
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,        // shares traded
    pub amount: f64,        // turnover
    pub change_pct: Option<f64>,
}

impl DailyBar {
    pub fn is_up(&self) -> bool {
        self.close >= self.open
    }
}

// ═══════════════════════════════════
// Financial Reports (Fundamental)
// ═══════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialReport {
    pub code: String,
    pub report_date: NaiveDate,     // e.g. 2024-12-31 (annual)
    pub report_type: ReportType,
    // Income Statement
    pub revenue: Option<f64>,       // 营业收入
    pub net_profit: Option<f64>,    // 净利润
    pub operating_profit: Option<f64>,
    // Balance Sheet
    pub total_assets: Option<f64>,
    pub total_liabilities: Option<f64>,
    pub equity: Option<f64>,        // 股东权益
    pub cash_equivalent: Option<f64>,
    // Cash Flow
    pub operating_cf: Option<f64>,  // 经营活动现金流净额
    // Per Share
    pub eps: Option<f64>,           // 每股收益
    pub bvps: Option<f64>,          // 每股净资产
    // Additional
    pub roe: Option<f64>,           // ROE %
    pub gross_margin: Option<f64>,
    pub net_margin: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ReportType {
    Q1,     // 一季报
    Mid,    // 中报
    Q3,     // 三季报
    Annual, // 年报
}

// ═══════════════════════════════════
// Financial Ratios
// ═══════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationRatios {
    pub code: String,
    pub date: NaiveDate,
    pub pe: Option<f64>,            // 市盈率
    pub pb: Option<f64>,            // 市净率
    pub ps: Option<f64>,            // 市销率
    pub pcf: Option<f64>,           // 市现率
    pub market_cap: Option<f64>,    // 总市值
    pub dividend_yield: Option<f64>,
}

// ═══════════════════════════════════
// Technical Indicators
// ═══════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorResult {
    pub name: String,
    pub values: Vec<IndicatorValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorValue {
    pub date: NaiveDate,
    pub value: f64,
}

// ═══════════════════════════════════
// Analysis Results
// ═══════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialHealth {
    pub code: String,
    pub score: u8,          // 0-100
    pub summary: String,
    pub details: Vec<String>,
}

/// User's watchlist / portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchlist {
    pub id: String,
    pub name: String,
    pub stocks: Vec<String>,
}

/// Intraday/minute-level K-line data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntradayBar {
    pub code: String,
    pub datetime: String,       // e.g. "2026-07-01 09:31"  or "2026-07-01 14:00"
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,            // shares
    pub amount: f64,            // turnover
}

/// Intraday period granularity
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum IntradayPeriod {
    Min1,
    Min5,
    Min15,
    Min30,
    Min60,
}

impl IntradayPeriod {
    pub fn label(&self) -> &str {
        match self {
            IntradayPeriod::Min1 => "1分钟",
            IntradayPeriod::Min5 => "5分钟",
            IntradayPeriod::Min15 => "15分钟",
            IntradayPeriod::Min30 => "30分钟",
            IntradayPeriod::Min60 => "60分钟",
        }
    }

    /// Tencent ktype parameter
    pub fn tencent_param(&self) -> &str {
        match self {
            IntradayPeriod::Min1 => "1min",
            IntradayPeriod::Min5 => "5min",
            IntradayPeriod::Min15 => "15min",
            IntradayPeriod::Min30 => "30min",
            IntradayPeriod::Min60 => "60min",
        }
    }
}

/// A data source error that allows fallback chaining
#[derive(Debug, thiserror::Error)]
pub enum DataSourceError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Not supported by this source")]
    NotSupported,
}

