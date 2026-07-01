use crate::state::AppState;
use crate::app::{Message, EMOJI_FONT};
use crate::ui::style;
use iced::widget::{column, container, row, scrollable, text, Column};
use iced::{Color, Element, Fill};
use stock_vision_analysis_core::FinancialAnalyzer;

pub fn view(state: &AppState) -> Element<'_, Message> {
    match &state.selected_stock {
        None => {
            let content = column![
                text("请搜索并选择一只股票").size(18.0).color(style::palette::TEXT_PRIMARY),
                text("搜索后点击「基本面分析」查看财务数据")
                    .size(14.0).color(style::palette::TEXT_SECONDARY),
            ].spacing(8).padding(16);
            return container(content).width(Fill).height(Fill).into();
        }
        Some(code) => {
            let name = state.stock_name.as_deref().unwrap_or(code).to_string();
            let mut content = Column::new().spacing(6).padding(16);
            content = content.push(text(format!("{}  {}", name, code)).size(22.0).color(style::palette::TEXT_PRIMARY));

            if state.financial_reports.is_empty() {
                content = content.push(
                    text("正在加载财务数据...")
                        .size(14.0).color(style::palette::TEXT_SECONDARY),
                );
            } else {
                if let Some(health) = &state.financial_health {
                    let score_color = if health.score >= 65 { Color::from_rgb(0.2, 0.8, 0.3) }
                        else if health.score >= 45 { Color::from_rgb(0.9, 0.7, 0.1) }
                        else { Color::from_rgb(0.9, 0.2, 0.2) };
                    content = content.push(text("").size(4.0));
                    content = content.push(
                        row![
                            text("财务健康评分").size(16.0).color(style::palette::TEXT_PRIMARY),
                            text(format!("{} / 100", health.score)).size(24.0).color(score_color),
                        ].spacing(12)
                    );
                    content = content.push(text(&health.summary).size(14.0).color(style::palette::TEXT_SECONDARY));
                    for detail in &health.details {
                        content = content.push(
                            row![
                                text("✅").font(EMOJI_FONT).size(12.0),
                                text(detail.clone()).size(12.0).color(Color::from_rgb(0.6, 0.8, 0.6)),
                            ].spacing(4)
                        );
                    }
                }
                content = content.push(text("").size(4.0));
                content = content.push(text("财务数据（最近4期）").size(16.0).color(style::palette::TEXT_PRIMARY));
                content = content.push(
                    row![
                        text("报告期").width(110).color(style::palette::TEXT_SECONDARY),
                        text("EPS").width(70).color(style::palette::TEXT_SECONDARY),
                        text("ROE").width(70).color(style::palette::TEXT_SECONDARY),
                        text("营收(亿)").width(90).color(style::palette::TEXT_SECONDARY),
                        text("净利(亿)").width(90).color(style::palette::TEXT_SECONDARY),
                        text("毛利率").width(80).color(style::palette::TEXT_SECONDARY),
                    ].spacing(4)
                );
                for report in state.financial_reports.iter().take(4) {
                    content = content.push(
                        row![
                            text(report.report_date.format("%Y-%m").to_string()).width(110).color(style::palette::TEXT_PRIMARY),
                            text(report.eps.map(|v| format!("{:.2}", v)).unwrap_or("-".into())).width(70).color(style::palette::TEXT_PRIMARY),
                            text(report.roe.map(|v| format!("{:.1}%", v)).unwrap_or("-".into())).width(70).color(style::palette::TEXT_PRIMARY),
                            text(report.revenue.map(|v| format!("{:.1}", v / 1e8)).unwrap_or("-".into())).width(90).color(style::palette::TEXT_PRIMARY),
                            text(report.net_profit.map(|v| format!("{:.1}", v / 1e8)).unwrap_or("-".into())).width(90).color(style::palette::TEXT_PRIMARY),
                            text(report.gross_margin.map(|v| format!("{:.1}%", v)).unwrap_or("-".into())).width(80).color(style::palette::TEXT_PRIMARY),
                        ].spacing(4)
                    );
                }
                if let Some(latest) = state.financial_reports.first() {
                    content = content.push(text("").size(4.0));
                    content = content.push(text("详细指标").size(16.0).color(style::palette::TEXT_PRIMARY));
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
                                text(label.to_string()).width(150).color(style::palette::TEXT_SECONDARY),
                                text(value.as_deref().unwrap_or("-").to_string()).width(120).color(style::palette::TEXT_PRIMARY),
                            ].spacing(8)
                        );
                    }
                }
            }
            container(scrollable(content)).width(Fill).height(Fill).into()
        }
    }
}
