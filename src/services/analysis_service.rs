/// Analysis service - fundamental and technical analysis

use stock_vision_analysis_core::FinancialAnalyzer;
use stock_vision_data_model::*;

pub struct AnalysisService {
    fundamental: FinancialAnalyzer,
}

impl AnalysisService {
    pub fn new() -> Self {
        Self {
            fundamental: FinancialAnalyzer,
        }
    }

    pub fn assess_financial_health(
        &self,
        report: &FinancialReport,
        ratios: &ValuationRatios,
    ) -> FinancialHealth {
        self.fundamental.score(report, ratios)
    }
}
