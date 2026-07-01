use async_trait::async_trait;
use chrono::NaiveDate;
use stock_vision_data_model::*;
use tracing::info;

use crate::{AdjustType, DataSource};

pub struct TencentSource {
    client: reqwest::Client,
}

impl TencentSource {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    fn tencent_code(code: &str, exchange: &Exchange) -> String {
        format!(
            "{}{}",
            match exchange {
                Exchange::SH => "sh",
                Exchange::SZ => "sz",
                Exchange::BJ => "bj",
            },
            code
        )
    }

    fn kline_url(code: &str, exchange: &Exchange, ktype: &str, start: &str, count: u32, adjust: &str) -> String {
        let tcode = Self::tencent_code(code, exchange);
        format!(
            "https://web.ifzq.gtimg.cn/appstock/app/fqkline/get?param={},{},{},,{},{}",
            tcode, ktype, start, count, adjust
        )
    }
}

#[async_trait]
impl DataSource for TencentSource {
    async fn search_stocks(&self, _keyword: &str) -> anyhow::Result<Vec<Stock>> {
        Ok(Vec::new())
    }

    async fn get_daily_bars(
        &self,
        code: &str,
        exchange: Exchange,
        start_date: Option<NaiveDate>,
        _end_date: Option<NaiveDate>,
        adjust: Option<AdjustType>,
    ) -> anyhow::Result<Vec<DailyBar>> {
        let start = start_date
            .unwrap_or(NaiveDate::from_ymd_opt(2005, 1, 1).unwrap())
            .format("%Y-%m-%d")
            .to_string();

        let adj_param = match adjust.unwrap_or(AdjustType::Forward) {
            AdjustType::Forward => "qfq",
            _ => "",
        };

        let url = Self::kline_url(code, &exchange, "day", &start, 2000, adj_param);
        let tcode = Self::tencent_code(code, &exchange);

        info!("Fetching Tencent K-line: {} (from {})", tcode, start);

        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;

        let json: serde_json::Value = serde_json::from_str(&text)?;
        let stock_key = tcode;
        let kline_data = json["data"][&stock_key]
            .as_object()
            .and_then(|obj| {
                if adj_param == "qfq" {
                    obj.get("qfqday").or_else(|| obj.get("day"))
                } else {
                    obj.get("day")
                }
            })
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let arr = item.as_array()?;
                        let date_str = arr.get(0)?.as_str()?;
                        let close = arr.get(1)?.as_str()?.parse::<f64>().ok()?;
                        let open = arr.get(2)?.as_str()?.parse::<f64>().ok()?;
                        let high = arr.get(3)?.as_str()?.parse::<f64>().ok()?;
                        let low = arr.get(4)?.as_str()?.parse::<f64>().ok()?;
                        let volume = arr.get(5)?.as_str()?.parse::<f64>().ok()?;

                        Some(DailyBar {
                            code: code.to_string(),
                            date: NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?,
                            open,
                            high,
                            low,
                            close,
                            volume: volume * 100.0, // convert to shares
                            amount: 0.0,
                            change_pct: None,
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(kline_data)
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
