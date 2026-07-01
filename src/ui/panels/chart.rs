use crate::state::AppState;
use crate::app::Message;
use crate::ui::style;
use crate::ui::charts::CandlestickCanvas;
use iced::widget::{column, container, row, text, Column};
use iced::{Color, Element, Fill};

pub fn view(state: &AppState) -> Element<'_, Message> {
    match &state.selected_stock {
        None => {
            let content = column![
                text("请搜索并选择一只股票").size(18.0).color(style::palette::TEXT_PRIMARY),
                text("在左侧搜索框输入股票名称或代码，点击搜索或回车查看K线")
                    .size(14.0).color(style::palette::TEXT_SECONDARY),
            ].spacing(8).padding(16);
            return container(content).width(Fill).height(Fill).into();
        }
        Some(code) => {
            let title = format!("{}  {}", state.stock_name.as_deref().unwrap_or(code), code);

            let price_summary: Element<'_, Message> = if !state.daily_bars.is_empty() {
                let latest = &state.daily_bars[state.daily_bars.len() - 1];
                let change_pct = ((latest.close - latest.open) / latest.open * 100.0);
                let color = if change_pct >= 0.0 { style::palette::RISE } else { style::palette::FALL };

                row![
                    col_metric("最新价", format!("{:.2}", latest.close), 28.0, color),
                    col_metric("涨幅", format!("{:.2}%", change_pct), 18.0, color),
                    col_metric("开盘", format!("{:.2}", latest.open), 18.0, style::palette::TEXT_PRIMARY),
                    col_metric("最高", format!("{:.2}", latest.high), 18.0, style::palette::TEXT_PRIMARY),
                    col_metric("最低", format!("{:.2}", latest.low), 18.0, style::palette::TEXT_PRIMARY),
                    col_metric("成交量", format!("{:.0}万", latest.volume / 10000.0), 18.0, style::palette::TEXT_PRIMARY),
                ].spacing(24).into()
            } else {
                text("正在加载数据...").size(14.0).color(style::palette::TEXT_SECONDARY).into()
            };

            let chart_element: Element<'static, Message> = if !state.daily_bars.is_empty() {
                CandlestickCanvas::new(state.daily_bars.clone()).into_element()
            } else {
                text("").into()
            };

            let content = column![
                text(title.clone()).size(22.0).color(style::palette::TEXT_PRIMARY),
                price_summary,
                text("").size(4.0),
                chart_element,
            ].spacing(4).padding(16);

            container(content).width(Fill).height(Fill).into()
        }
    }
}

fn col_metric(label: &str, value: String, size: f32, color: Color) -> Element<'_, Message> {
    column![
        text(label).size(12.0).color(style::palette::TEXT_SECONDARY),
        text(value).size(size).color(color),
    ].into()
}
