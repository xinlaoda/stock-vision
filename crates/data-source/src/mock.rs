use async_trait::async_trait;
use chrono::{NaiveDate, Datelike};
use stock_vision_data_model::*;

use crate::{AdjustType, DataSource};

pub struct MockSource;

#[async_trait]
impl DataSource for MockSource {
    async fn search_stocks(&self, keyword: &str) -> anyhow::Result<Vec<Stock>> {
        Ok(vec![
            Stock {
                code: "000001".to_string(),
                name: format!("Mock{}", keyword),
                exchange: Exchange::SZ,
                sector: Some("银行".to_string()),
                industry: Some("银行".to_string()),
                list_date: None,
                total_shares: Some(100_000_000_000.0),
                float_shares: Some(80_000_000_000.0),
            },
        ])
    }

    async fn get_daily_bars(
        &self,
        code: &str,
        _exchange: Exchange,
        _start_date: Option<NaiveDate>,
        _end_date: Option<NaiveDate>,
        _adjust: Option<AdjustType>,
    ) -> anyhow::Result<Vec<DailyBar>> {
        let mut bars = Vec::new();
        let start = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        let mut price = 10.0;
        for i in 0..250 {
            let date = start + chrono::Duration::days(i);
            if date.weekday() == chrono::Weekday::Sat || date.weekday() == chrono::Weekday::Sun {
                continue;
            }
            let change = (i as f64 * 0.5).sin() * 0.5;
            let open = price;
            let close = price + change;
            let high = open.max(close) + 0.2;
            let low = open.min(close) - 0.2;
            bars.push(DailyBar {
                code: code.to_string(),
                date,
                open,
                high,
                low,
                close,
                volume: 10_000_000.0 + (i as f64 * 1000.0),
                amount: (open * 10_000_000.0),
                change_pct: Some(change / open * 100.0),
            });
            price = close;
        }
        Ok(bars)
    }

    async fn get_financial_reports(
        &self,
        code: &str,
        _exchange: Exchange,
        _report_type: Option<ReportType>,
    ) -> anyhow::Result<Vec<FinancialReport>> {
        Ok(vec![FinancialReport {
            code: code.to_string(),
            report_date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            report_type: ReportType::Annual,
            revenue: Some(100_000_000_000.0),
            net_profit: Some(15_000_000_000.0),
            operating_profit: Some(18_000_000_000.0),
            total_assets: Some(500_000_000_000.0),
            total_liabilities: Some(300_000_000_000.0),
            equity: Some(200_000_000_000.0),
            cash_equivalent: Some(50_000_000_000.0),
            operating_cf: Some(20_000_000_000.0),
            eps: Some(1.5),
            bvps: Some(12.0),
            roe: Some(15.0),
            gross_margin: Some(40.0),
            net_margin: Some(15.0),
        }])
    }

    async fn get_valuation_ratios(
        &self,
        code: &str,
        _exchange: Exchange,
    ) -> anyhow::Result<ValuationRatios> {
        Ok(ValuationRatios {
            code: code.to_string(),
            date: chrono::Utc::now().date_naive(),
            pe: Some(15.0),
            pb: Some(2.0),
            ps: Some(3.0),
            pcf: Some(10.0),
            market_cap: Some(200_000_000_000.0),
            dividend_yield: Some(3.5),
        })
    }
}
