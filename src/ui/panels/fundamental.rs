use crate::state::AppState;
use crate::app::Message;
use iced::widget::{column, container, row, scrollable, text, Column};
use iced::{Color, Element, Length};

pub fn view(state: &AppState) -> Element<'_, Message> {
    match &state.selected_stock {
        None => {
            let content = column![
                text("请搜索并选择一只股票").size(18),
                text("在左侧搜索后，点击「基本面分析」查看财务数据").size(14)
                    .style(Color::from_rgb(0.5, 0.5, 0.6)),
            ]
            .spacing(8).padding(16);
            return container(content).width(Length::Fill).height(Length::Fill).into();
        }
        Some(code) => {
            let name = state.stock_name.as_deref().unwrap_or(code);
            let mut content = Column::new().spacing(8).padding(16);

            content = content.push(text(format!("{} ({}) 基本面分析", name, code)).size(22));

            if state.financial_reports.is_empty() {
                content = content.push(
                    text("点击右侧菜单「基本面分析」加载财务数据")
                        .size(14).style(Color::from_rgb(0.5, 0.5, 0.6)),
                );
            } else {
                // ── Financial Health Score ──
                if let Some(health) = &state.financial_health {
                    let score_color = if health.score >= 65 {
                        Color::from_rgb(0.2, 0.8, 0.3)
                    } else if health.score >= 45 {
                        Color::from_rgb(0.9, 0.7, 0.1)
                    } else {
                        Color::from_rgb(0.9, 0.2, 0.2)
                    };

                    content = content.push(text("").size(4));
                    content = content.push(
                        row![
                            text("财务健康评分").size(16),
                            text(format!("{} / 100", health.score)).size(24).style(score_color),
                        ].spacing(12)
                    );
                    content = content.push(
                        text(&health.summary).size(14).style(Color::from_rgb(0.7, 0.7, 0.8)),
                    );
                    for detail in &health.details {
                        content = content.push(
                            text(format!("  ✅ {}", detail)).size(12).style(Color::from_rgb(0.6, 0.8, 0.6)),
                        );
                    }
                }

                // ── Financial Data Table ──
                content = content.push(text("").size(8));
                content = content.push(text("财务数据 (最近4期)").size(16));
                content = content.push(
                    row![
                        text("报告期").width(110),
                        text("EPS").width(80),
                        text("ROE").width(80),
                        text("营收(亿)").width(100),
                        text("净利(亿)").width(100),
                        text("毛利率").width(80),
                    ].spacing(4)
                );

                for report in state.financial_reports.iter().take(4) {
                    let row_content = row![
                        text(report.report_date.format("%Y-%m").to_string()).width(110),
                        text(report.eps.map(|v| format!("{:.2}", v)).unwrap_or("-".into())).width(80),
                        text(report.roe.map(|v| format!("{:.1}%", v)).unwrap_or("-".into())).width(80),
                        text(report.revenue.map(|v| format!("{:.1}", v / 1e8)).unwrap_or("-".into())).width(100),
                        text(report.net_profit.map(|v| format!("{:.1}", v / 1e8)).unwrap_or("-".into())).width(100),
                        text(report.gross_margin.map(|v| format!("{:.1}%", v)).unwrap_or("-".into())).width(80),
                    ].spacing(4);
                    content = content.push(row_content);
                }

                // ── Detailed Ratios ──
                if let Some(latest) = state.financial_reports.first() {
                    content = content.push(text("").size(8));
                    content = content.push(text("详细指标").size(16));

                    let ratios = FinancialAnalyzer::calculate_ratios(latest);
                    let details = vec![
                        ("净资产(亿)", latest.equity.map(|v| format!("{:.1}", v / 1e8))),
                        ("每股净资产", latest.bvps.map(|v| format!("{:.2}", v))),
                        ("经营现金流/股", latest.operating_cf.map(|v| format!("{:.2}", v))),
                        ("净利率", ratios.profit_margin.map(|v| format!("{:.1}%", v))),
                        ("负债率", ratios.debt_ratio.map(|v| format!("{:.1}%", v))),
                    ];

                    for (label, value) in &details {
                        content = content.push(
                            row![
                                text(*label).width(150).style(Color::from_rgb(0.6, 0.6, 0.7)),
                                text(value.as_deref().unwrap_or("-")).width(120),
                            ].spacing(8)
                        );
                    }
                }
            }

            return container(scrollable(content)).width(Length::Fill).height(Length::Fill).into();
        }
    }
}

// Re-use FinancialAnalyzer from analysis-core
use stock_vision_analysis_core::FinancialAnalyzer;
