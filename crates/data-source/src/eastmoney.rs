use async_trait::async_trait;
use chrono::NaiveDate;
use serde::Deserialize;
use stock_vision_data_model::*;
use tracing::info;

use crate::{AdjustType, DataSource};

pub struct EastMoneySource {
    client: reqwest::Client,
}

impl EastMoneySource {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(rename = "QuotationCodeTable")]
    quotation_table: QuotationCodeTable,
}

#[derive(Deserialize)]
struct QuotationCodeTable {
    #[serde(rename = "Data")]
    data: Vec<SearchItem>,
}

#[derive(Deserialize)]
struct SearchItem {
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "JYS")]
    jys: String,
    #[serde(rename = "SecurityTypeName")]
    sec_type: Option<String>,
}

#[async_trait]
impl DataSource for EastMoneySource {
    async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>> {
        info!("Searching stocks: {}", keyword);
        let url = format!(
            "https://searchadapter.eastmoney.com/api/suggest/get?input={}&count=20&type=14",
            Self::url_encode(keyword)
        );
        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;
        let search_resp: SearchResponse = serde_json::from_str(&text)?;

        let stocks: Vec<Stock> = search_resp
            .quotation_table
            .data
            .into_iter()
            // Filter: only A-stock exchanges (JYS=2 for SH, JYS=6 for SZ)
            .filter(|item| item.jys == "2" || item.jys == "6")
            .filter_map(|item| {
                let exchange = match item.jys.as_str() {
                    "2" => Exchange::SH,
                    "6" => Exchange::SZ,
                    _ => return None,
                };
                Some(Stock {
                    code: item.code,
                    name: item.name,
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
        _code: &str,
        _exchange: Exchange,
        _start_date: Option<NaiveDate>,
        _end_date: Option<NaiveDate>,
        _adjust: Option<AdjustType>,
    ) -> anyhow::Result<Vec<DailyBar>> {
        // Use TencentSource for K-line data
        Ok(Vec::new())
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

impl EastMoneySource {
    fn url_encode(s: &str) -> String {
        let mut result = String::new();
        for byte in s.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                b' ' => result.push_str("%20"),
                _ => result.push_str(&format!("%{:02X}", byte)),
            }
        }
        result
    }
}
