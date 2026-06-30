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
                .default_headers({
                    let mut h = reqwest::header::HeaderMap::new();
                    h.insert(
                        "Referer",
                        reqwest::header::HeaderValue::from_static("https://data.eastmoney.com/"),
                    );
                    h
                })
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    fn to_secucode(code: &str, exchange: &Exchange) -> String {
        let mkt = match exchange {
            Exchange::SH => ".SH",
            Exchange::SZ => ".SZ",
            Exchange::BJ => ".BJ",
        };
        format!("{}{}", code, mkt)
    }

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

// ── Search API ──

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
}

// ── Financial API response ──

#[derive(Deserialize)]
struct FinApiResponse {
    result: Option<FinResult>,
    success: bool,
    message: Option<String>,
}

#[derive(Deserialize)]
struct FinResult {
    data: Vec<serde_json::Value>,
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
        Ok(Vec::new())
    }

    async fn get_financial_reports(
        &self,
        code: &str,
        exchange: Exchange,
        _report_type: Option<ReportType>,
    ) -> anyhow::Result<Vec<FinancialReport>> {
        let secucode = Self::to_secucode(code, &exchange);
        info!("Fetching financial reports for {}", secucode);

        let cols = [
            "REPORT_DATE", "REPORT_TYPE",
            "EPSJB",      // 基本每股收益
            "EPSKCJB",    // 扣非每股收益
            "BPS",        // 每股净资产
            "ROEJQ",      // ROE(加权)
            "TOTALOPERATEREVE",   // 营业总收入
            "PARENTNETPROFIT",     // 归母净利润
            "KCFJCXSYJLR",         // 扣非净利润
            "XSJLL",      // 销售净利率
            "XSMLL",      // 销售毛利率
            "JZC",        // 净资产
            "ZCFZL",      // 资产负债率
            "MGJYXJJE",   // 每股经营现金流
        ];

        let url = format!(
            "https://datacenter.eastmoney.com/securities/api/data/v1/get\
             ?reportName=RPT_F10_FINANCE_MAINFINADATA\
             &columns={}\
             &filter=(SECUCODE=%22{}%22)\
             &pageNumber=1&pageSize=8\
             &sortTypes=-1&sortColumns=REPORT_DATE",
            cols.join(","),
            secucode
        );

        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;
        let api_resp: FinApiResponse = serde_json::from_str(&text)?;

        let reports = api_resp
            .result
            .map(|r| {
                r.data
                    .iter()
                    .filter_map(|item| {
                        let date_str = item.get("REPORT_DATE")?.as_str()?;
                        let report_date = NaiveDate::parse_from_str(
                            date_str.split(' ').next().unwrap_or(date_str),
                            "%Y-%m-%d",
                        )
                        .ok()?;

                        let report_type = match item.get("REPORT_TYPE")?.as_i64() {
                            Some(1) => ReportType::Q1,
                            Some(2) => ReportType::Mid,
                            Some(3) => ReportType::Q3,
                            Some(4) => ReportType::Annual,
                            _ => ReportType::Annual,
                        };

                        let f = |name: &str| item.get(name).and_then(|v| v.as_f64());

                        Some(FinancialReport {
                            code: code.to_string(),
                            report_date,
                            report_type,
                            revenue: f("TOTALOPERATEREVE"),
                            net_profit: f("PARENTNETPROFIT"),
                            operating_profit: f("KCFJCXSYJLR"),
                            total_assets: None, // 在另一张表
                            total_liabilities: None,
                            equity: f("JZC"),
                            cash_equivalent: None,
                            operating_cf: f("MGJYXJJE"),
                            eps: f("EPSJB"),
                            bvps: f("BPS"),
                            roe: f("ROEJQ"),
                            gross_margin: f("XSMLL"),
                            net_margin: f("XSJLL"),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(reports)
    }

    async fn get_valuation_ratios(
        &self,
        code: &str,
        exchange: Exchange,
    ) -> anyhow::Result<ValuationRatios> {
        // Get latest financial data to compute ratios
        let reports = self.get_financial_reports(code, exchange, None).await?;

        // Market cap from real-time quote - use Tencent for this
        Ok(ValuationRatios {
            code: code.to_string(),
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
