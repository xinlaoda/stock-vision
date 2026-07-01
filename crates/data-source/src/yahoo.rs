use async_trait::async_trait;
use chrono::NaiveDate;
use stock_vision_data_model::*;
use tracing::info;

use crate::{AdjustType, DataSource};

/// Yahoo Finance data source for US stocks.
/// 
/// Uses Yahoo Finance's v8 chart API (free, no API key required).
/// Supports daily/weekly/monthly K-line data and stock search.
/// 
/// Note: Yahoo Finance is an unofficial API; data may be delayed ~15 min.
pub struct YahooSource {
    client: reqwest::Client,
}

impl YahooSource {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .default_headers({
                    let mut h = reqwest::header::HeaderMap::new();
                    h.insert("Accept", reqwest::header::HeaderValue::from_static("application/json"));
                    h
                })
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Map exchange to Yahoo Finance exchange suffix
    fn yahoo_suffix(exchange: &Exchange) -> &str {
        match exchange {
            Exchange::NYSE => "",     // e.g. "AAPL" without suffix
            Exchange::NASDAQ => "",   // e.g. "MSFT" without suffix
            _ => "",                 // A-share stocks not supported by Yahoo
        }
    }

    /// Build Yahoo Finance chart URL
    fn chart_url(code: &str, exchange: &Exchange, range: &str, interval: &str) -> String {
        let ticker = format!("{}{}", code, Self::yahoo_suffix(exchange));
        format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval={}&includePrePost=false",
            ticker, range, interval
        )
    }

    /// Map KlinePeriod to Yahoo range string
    fn yahoo_range(period: KlinePeriod) -> &'static str {
        match period {
            KlinePeriod::Daily => "2y",
            KlinePeriod::Weekly => "5y",
            KlinePeriod::Monthly => "10y",
            KlinePeriod::Yearly => "max",
        }
    }

    /// Map KlinePeriod to Yahoo interval string
    fn yahoo_interval(period: KlinePeriod) -> &'static str {
        match period {
            KlinePeriod::Daily => "1d",
            KlinePeriod::Weekly => "1wk",
            KlinePeriod::Monthly => "1mo",
            KlinePeriod::Yearly => "3mo",
        }
    }
}

/// Simple KlinePeriod enum for Yahoo (not the same as app's KlinePeriod)
enum KlinePeriod {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[async_trait]
impl DataSource for YahooSource {
    async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>> {
        info!("Yahoo search: {}", keyword);
        
        let url = format!(
            "https://query1.finance.yahoo.com/v1/finance/search?q={}&quotesCount=10&newsCount=0",
            keyword
        );

        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;
        let json: serde_json::Value = serde_json::from_str(&text)?;

        let empty_vec: Vec<serde_json::Value> = Vec::new();
        let quotes = json["quotes"].as_array().unwrap_or(&empty_vec);

        let stocks: Vec<Stock> = quotes.iter()
            .filter_map(|item| {
                let symbol = item["symbol"].as_str()?;
                let name = item["shortname"].as_str()
                    .or_else(|| item["longname"].as_str())
                    .unwrap_or(symbol);
                let exchange_str = item["exchange"].as_str().unwrap_or("");

                let exchange = match exchange_str {
                    "NYQ" | "NYE" => Exchange::NYSE,
                    "NAS" | "NMS" | "NCM" | "NGM" => Exchange::NASDAQ,
                    "SHS" | "SHH" => Exchange::SH,
                    "SZN" | "SHE" => Exchange::SZ,
                    _ => return None,
                };

                // Skip non-US/non-China stocks for simplicity
                if !exchange.is_us() && !matches!(exchange, Exchange::SH | Exchange::SZ) {
                    return None;
                }

                Some(Stock {
                    code: symbol.to_string(),
                    name: name.to_string(),
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
        // Only handle US stocks
        if !exchange.is_us() {
            return Ok(Vec::new());
        }

        let range = Self::yahoo_range(KlinePeriod::Daily);
        let interval = Self::yahoo_interval(KlinePeriod::Daily);
        let url = Self::chart_url(code, &exchange, range, interval);

        info!("Fetching Yahoo daily K-line: {} ({})", code, url);

        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;
        let json: serde_json::Value = serde_json::from_str(&text)?;

        Self::parse_chart_response(&json, code, &exchange)
    }

    async fn get_financial_reports(
        &self,
        _code: &str,
        _exchange: Exchange,
        _report_type: Option<ReportType>,
    ) -> anyhow::Result<Vec<FinancialReport>> {
        Ok(Vec::new())
    }

    async fn get_valuation_ratios(
        &self,
        _code: &str,
        _exchange: Exchange,
    ) -> anyhow::Result<ValuationRatios> {
        Ok(ValuationRatios {
            code: String::new(),
            date: chrono::Utc::now().date_naive(),
            pe: None,
            pb: None,
            ps: None,
            pcf: None,
            market_cap: None,
            dividend_yield: None,
        })
    }
}

impl YahooSource {
    /// Parse Yahoo Finance v8 chart API response into DailyBar vector
    fn parse_chart_response(json: &serde_json::Value, code: &str, exchange: &Exchange) -> anyhow::Result<Vec<DailyBar>> {
        let result = json["chart"]["result"]
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| anyhow::anyhow!("No chart result"))?;

        let timestamps = result["timestamp"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No timestamp array"))?;

        let quote = &result["indicators"]["quote"]
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| anyhow::anyhow!("No quote data"))?;

        let opens = quote["open"].as_array().ok_or_else(|| anyhow::anyhow!("No open"))?;
        let highs = quote["high"].as_array().ok_or_else(|| anyhow::anyhow!("No high"))?;
        let lows = quote["low"].as_array().ok_or_else(|| anyhow::anyhow!("No low"))?;
        let closes = quote["close"].as_array().ok_or_else(|| anyhow::anyhow!("No close"))?;
        let volumes = quote["volume"].as_array().ok_or_else(|| anyhow::anyhow!("No volume"))?;

        let mut bars: Vec<DailyBar> = Vec::new();

        for (i, ts) in timestamps.iter().enumerate() {
            let timestamp = ts.as_i64().ok_or_else(|| anyhow::anyhow!("Bad timestamp"))?;
            
            // Skip null values (trading holidays, etc.)
            let open = opens.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let high = highs.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let low = lows.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let close = closes.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let volume = volumes.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0);

            // Convert Unix timestamp to NaiveDate
            let ndays = timestamp / 86400;
            let date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(ndays);

            bars.push(DailyBar {
                code: code.to_string(),
                date,
                open,
                high,
                low,
                close,
                volume,
                amount: 0.0,
                change_pct: None,
            });
        }

        Ok(bars)
    }
}
