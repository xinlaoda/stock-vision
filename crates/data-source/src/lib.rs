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
}
