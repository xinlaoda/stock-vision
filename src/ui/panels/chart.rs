use crate::state::AppState;
use crate::app::Message;
use crate::ui::charts::CandlestickCanvas;
use iced::widget::{column, container, row, text, Column};
use iced::{Color, Element, Length};

pub fn view(state: &AppState) -> Element<'_, Message> {
    match &state.selected_stock {
        None => {
            let content = column![
                text("请搜索并选择一只股票").size(18),
                text("在左侧搜索框输入股票名称或代码，回车搜索，点击结果查看K线")
                    .size(14)
                    .style(Color::from_rgb(0.5, 0.5, 0.6)),
            ]
            .spacing(8)
            .padding(16);
            return container(content).width(Length::Fill).height(Length::Fill).into();
        }
        Some(code) => {
            let title = format!(
                "{} - {}",
                state.stock_name.as_deref().unwrap_or(code),
                code
            );

            // Price summary
            let price_summary: Element<'_, Message> = if !state.daily_bars.is_empty() {
                let latest = &state.daily_bars[state.daily_bars.len() - 1];
                let change_pct = ((latest.close - latest.open) / latest.open * 100.0);
                let color = if change_pct >= 0.0 {
                    Color::from_rgb(0.9, 0.24, 0.24)
                } else {
                    Color::from_rgb(0.15, 0.65, 0.24)
                };

                row![
                    column![
                        text("最新价").size(12).style(Color::from_rgb(0.5, 0.5, 0.6)),
                        text(format!("{:.2}", latest.close)).size(28).style(color),
                    ],
                    column![
                        text("涨幅").size(12).style(Color::from_rgb(0.5, 0.5, 0.6)),
                        text(format!("{:.2}%", change_pct)).size(18).style(color),
                    ],
                    column![
                        text("开盘").size(12).style(Color::from_rgb(0.5, 0.5, 0.6)),
                        text(format!("{:.2}", latest.open)).size(18),
                    ],
                    column![
                        text("最高").size(12).style(Color::from_rgb(0.5, 0.5, 0.6)),
                        text(format!("{:.2}", latest.high)).size(18),
                    ],
                    column![
                        text("最低").size(12).style(Color::from_rgb(0.5, 0.5, 0.6)),
                        text(format!("{:.2}", latest.low)).size(18),
                    ],
                    column![
                        text("成交量").size(12).style(Color::from_rgb(0.5, 0.5, 0.6)),
                        text(format!("{:.0}万", latest.volume / 10000.0)).size(18),
                    ],
                ]
                .spacing(24)
                .into()
            } else {
                text("正在加载数据...")
                    .size(14)
                    .style(Color::from_rgb(0.5, 0.5, 0.6))
                    .into()
            };

            // Build chart (consumes the CandlestickCanvas)
            let chart_element: Element<'static, Message> = if !state.daily_bars.is_empty() {
                let candlestick = CandlestickCanvas::new(state.daily_bars.clone());
                candlestick.into_element()
            } else {
                text("").into()
            };

            let content = column![
                text(&title).size(22),
                price_summary,
                text("").size(4),
                chart_element,
            ]
            .spacing(4)
            .padding(16);

            container(content).width(Length::Fill).height(Length::Fill).into()
        }
    }
}
