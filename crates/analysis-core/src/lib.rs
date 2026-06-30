/// Fundamental analysis and financial health scoring engine.

use stock_vision_data_model::*;

/// Financial health scoring based on fundamentals
pub struct FinancialAnalyzer;

impl FinancialAnalyzer {
    /// Score a company's financial health (0-100)
    pub fn score(&self, report: &FinancialReport, ratios: &ValuationRatios) -> FinancialHealth {
        let mut score = 50u8;
        let mut details = Vec::new();

        // Profitability (max 25 points)
        if let Some(roe) = report.roe {
            if roe > 20.0 {
                score += 15;
                details.push("ROE > 20%, excellent profitability".to_string());
            } else if roe > 15.0 {
                score += 10;
                details.push("ROE > 15%, good profitability".to_string());
            } else if roe > 10.0 {
                score += 5;
                details.push("ROE > 10%, adequate profitability".to_string());
            } else if roe < 5.0 {
                score = score.saturating_sub(10);
                details.push("ROE < 5%, low profitability".to_string());
            }
        }

        if let Some(margin) = report.net_margin {
            if margin > 20.0 {
                score += 10;
                details.push("Net margin > 20%, strong pricing power".to_string());
            } else if margin > 10.0 {
                score += 5;
            }
        }

        // Growth (max 15 points)
        if let Some(pe) = ratios.pe {
            if pe < 15.0 {
                score += 5;
                details.push("PE < 15, potentially undervalued".to_string());
            } else if pe > 50.0 {
                score = score.saturating_sub(5);
                details.push("PE > 50, potentially overvalued".to_string());
            }
        }

        if let Some(pb) = ratios.pb {
            if pb < 2.0 {
                score += 5;
            } else if pb > 10.0 {
                score = score.saturating_sub(5);
            }
        }

        // Debt (max 10 points)
        if let (Some(assets), Some(liabilities)) = (report.total_assets, report.total_liabilities) {
            if assets > 0.0 {
                let debt_ratio = liabilities / assets;
                if debt_ratio < 0.3 {
                    score += 10;
                    details.push("Low debt ratio, financially conservative".to_string());
                } else if debt_ratio < 0.5 {
                    score += 5;
                } else if debt_ratio > 0.7 {
                    score = score.saturating_sub(10);
                    details.push("High debt ratio, financial risk".to_string());
                }
            }
        }

        // Cash Flow (max 10 points)
        if let (Some(opcf), Some(np)) = (report.operating_cf, report.net_profit) {
            if np > 0.0 && opcf / np > 0.8 {
                score += 10;
                details.push("Strong operating cash flow".to_string());
            }
        }

        let summary = if score >= 80 {
            "Excellent financial health".to_string()
        } else if score >= 65 {
            "Good financial health".to_string()
        } else if score >= 50 {
            "Average financial health".to_string()
        } else if score >= 35 {
            "Below average financial health".to_string()
        } else {
            "Poor financial health, invest with caution".to_string()
        };

        FinancialHealth {
            code: report.code.clone(),
            score: score.min(100),
            summary,
            details,
        }
    }

    /// Calculate key financial ratios
    pub fn calculate_ratios(report: &FinancialReport) -> Ratios {
        Ratios {
            roe: report.net_profit.zip(report.equity).map(|(np, eq)| {
                if eq > 0.0 { np / eq * 100.0 } else { 0.0 }
            }),
            debt_ratio: report
                .total_liabilities
                .zip(report.total_assets)
                .map(|(l, a)| if a > 0.0 { l / a * 100.0 } else { 0.0 }),
            eps: report.eps,
            bvps: report.bvps,
            profit_margin: report
                .net_profit
                .zip(report.revenue)
                .map(|(np, rev)| if rev > 0.0 { np / rev * 100.0 } else { 0.0 }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ratios {
    pub roe: Option<f64>,
    pub debt_ratio: Option<f64>,
    pub eps: Option<f64>,
    pub bvps: Option<f64>,
    pub profit_margin: Option<f64>,
}
