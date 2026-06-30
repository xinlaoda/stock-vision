use crate::state::AppState;
use crate::app::Message;
use iced::widget::{column, container, row, scrollable, text, Column, Text};
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
            container(content).width(Length::Fill).height(Length::Fill).into()
        }
        Some(code) => {
            let title = format!(
                "{} - {}",
                state.stock_name.as_deref().unwrap_or(code),
                code
            );

            let mut content = Column::new().spacing(4).padding(16);
            content = content.push(text(&title).size(22));

            if state.daily_bars.is_empty() {
                content = content.push(
                    text("正在加载数据...")
                        .size(14)
                        .style(Color::from_rgb(0.5, 0.5, 0.6)),
                );
            } else {
                let latest = &state.daily_bars[state.daily_bars.len() - 1];
                let change_pct = ((latest.close - latest.open) / latest.open * 100.0);
                let color = if change_pct >= 0.0 {
                    Color::from_rgb(0.9, 0.24, 0.24)
                } else {
                    Color::from_rgb(0.15, 0.65, 0.24)
                };

                // Price summary bar
                let info = row![
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
                .spacing(24);

                content = content.push(info);
                content = content.push(text("")); // spacer

                // Recent bars table header
                let header = row![
                    text("日期").width(120),
                    text("开盘").width(80),
                    text("最高").width(80),
                    text("最低").width(80),
                    text("收盘").width(80),
                    text("成交量").width(100),
                ]
                .spacing(4);
                content = content.push(header);

                // Recent 20 bars
                let start = if state.daily_bars.len() > 20 {
                    state.daily_bars.len() - 20
                } else {
                    0
                };
                for bar in state.daily_bars[start..].iter().rev() {
                    let row_color = if bar.close >= bar.open {
                        Color::from_rgb(0.9, 0.24, 0.24)
                    } else {
                        Color::from_rgb(0.15, 0.65, 0.24)
                    };
                    let bar_row = row![
                        text(bar.date.format("%Y-%m-%d").to_string()).width(120),
                        text(format!("{:.2}", bar.open)).width(80).style(row_color),
                        text(format!("{:.2}", bar.high)).width(80),
                        text(format!("{:.2}", bar.low)).width(80),
                        text(format!("{:.2}", bar.close)).width(80).style(row_color),
                        text(format!("{:.0}", bar.volume / 10000.0)).width(100),
                    ]
                    .spacing(4);
                    content = content.push(bar_row);
                }

                content = content.push(
                    text(format!("共 {} 条日K线数据", state.daily_bars.len()))
                        .size(12)
                        .style(Color::from_rgb(0.5, 0.5, 0.6)),
                );
            }

            container(scrollable(content))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}
