use crate::state::{AppState, KlinePeriod, TimeRange};
use crate::app::Message;
use crate::ui::style;
use crate::ui::charts::{CandlestickCanvas, IntradayCanvas};
use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Fill};
use iced::widget::button::Status;
use iced::Theme;
use stock_vision_data_model::{IntradayPeriod, IntradayBar};

fn inactive_btn_style() -> impl Fn(&Theme, Status) -> iced::widget::button::Style {
    |_t: &Theme, _s: Status| iced::widget::button::Style {
        background: Some(style::colors().bg_light.into()),
        text_color: style::colors().text_secondary,
        ..Default::default()
    }
}

fn active_period_style() -> impl Fn(&Theme, Status) -> iced::widget::button::Style {
    |_t: &Theme, _s: Status| iced::widget::button::Style {
        background: Some(style::colors().accent.into()),
        text_color: Color::WHITE,
        ..Default::default()
    }
}

fn active_range_style() -> impl Fn(&Theme, Status) -> iced::widget::button::Style {
    |_t: &Theme, _s: Status| iced::widget::button::Style {
        background: Some(style::colors().accent.into()),
        text_color: Color::WHITE,
        ..Default::default()
    }
}

fn make_period_btn(label: &'static str, period: KlinePeriod, is_active: bool) -> iced::widget::Button<'static, Message> {
    let btn = button(text(label).size(13.0))
        .on_press(Message::SetKlinePeriod(period))
        .padding(4);
    if is_active { btn.style(active_period_style()) } else { btn.style(inactive_btn_style()) }
}

fn make_range_btn(label: &'static str, range: TimeRange, is_active: bool) -> iced::widget::Button<'static, Message> {
    let btn = button(text(label).size(12.0))
        .on_press(Message::SetTimeRange(range))
        .padding(4);
    if is_active { btn.style(active_range_style()) } else { btn.style(inactive_btn_style()) }
}

fn compute_ma(bars: &[stock_vision_data_model::DailyBar], period: usize) -> Option<f64> {
    if bars.len() < period { return None; }
    let start = bars.len() - period;
    let sum: f64 = bars[start..].iter().map(|b| b.close).sum();
    Some(sum / period as f64)
}

fn ma_item(label: &str, value: Option<f64>, color: Color) -> Element<'_, Message> {
    row![
        text(label).size(10.0).color(color),
        text(value.map_or("-".into(), |v| format!("{:.2}", v))).size(12.0).color(style::colors().text_primary),
    ].spacing(2).into()
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    match &state.selected_stock {
        None => {
            let content = column![
                text("请搜索并选择一只股票").size(18.0).color(style::colors().text_primary),
                text("在左侧搜索框输入股票名称或代码，点击搜索或回车查看K线")
                    .size(14.0).color(style::colors().text_secondary),
            ].spacing(8).padding(16);
            return container(content).width(Fill).height(Fill).into();
        }
        Some(code) => {
            let title_color = Color::from_rgb(1.0, 0.65, 0.0);
            let title = format!("{}  {}", state.stock_name.as_deref().unwrap_or(code), code);
            let title_elem = text(title).size(22.0).color(title_color);

            let label_color = style::colors().text_secondary;
            let price_summary: Element<'_, Message> = if !state.daily_bars.is_empty() {
                let latest = &state.daily_bars[state.daily_bars.len() - 1];
                let change_pct = (latest.close - latest.open) / latest.open * 100.0;
                let change_color = if change_pct >= 0.0 { style::colors().rise } else { style::colors().fall };
                row![
                    metric("最新价", format!("{:.2}", latest.close), 28.0, label_color, change_color),
                    metric("涨幅", format!("{:.2}%", change_pct), 18.0, label_color, change_color),
                    metric("开盘", format!("{:.2}", latest.open), 18.0, label_color, style::colors().text_primary),
                    metric("最高", format!("{:.2}", latest.high), 18.0, label_color, style::colors().text_primary),
                    metric("最低", format!("{:.2}", latest.low), 18.0, label_color, style::colors().text_primary),
                    metric("成交量", format!("{:.0}万", latest.volume / 10000.0), 18.0, label_color, style::colors().text_primary),
                ].spacing(24).into()
            } else {
                if state.intraday_period.is_some() && !state.intraday_bars.is_empty() {
                    let latest = &state.intraday_bars[state.intraday_bars.len() - 1];
                    let change_pct = (latest.close - latest.open) / latest.open * 100.0;
                    let change_color = if change_pct >= 0.0 { style::colors().rise } else { style::colors().fall };
                    row![
                        metric("最新价", format!("{:.2}", latest.close), 28.0, style::colors().text_secondary, change_color),
                        metric("涨幅", format!("{:.2}%", change_pct), 18.0, style::colors().text_secondary, change_color),
                        metric("开盘", format!("{:.2}", latest.open), 18.0, style::colors().text_secondary, style::colors().text_primary),
                        metric("最高", format!("{:.2}", latest.high), 18.0, style::colors().text_secondary, style::colors().text_primary),
                        metric("最低", format!("{:.2}", latest.low), 18.0, style::colors().text_secondary, style::colors().text_primary),
                        metric("成交量", format!("{:.0}万", latest.volume / 10000.0), 18.0, style::colors().text_secondary, style::colors().text_primary),
                    ].spacing(24).into()
                } else {
                    text("正在加载数据...").size(14.0).color(style::colors().text_secondary).into()
                }
            };

            // ── MA Legend (hidden for intraday) ──
            let ma_legend: Element<'_, Message> = if state.intraday_period.is_none() && !state.daily_bars.is_empty() {
                let bars = &state.daily_bars;
                row![
                    ma_item("MA5", compute_ma(bars, 5), Color::from_rgb(1.0, 0.8, 0.0)),
                    ma_item("MA10", compute_ma(bars, 10), Color::from_rgb(0.3, 0.7, 1.0)),
                    ma_item("MA20", compute_ma(bars, 20), Color::from_rgb(1.0, 0.4, 0.7)),
                    ma_item("MA60", compute_ma(bars, 60), Color::from_rgb(0.2, 0.8, 0.4)),
                ].spacing(8).into()
            } else { text("").into() };

            // ── Period selector ──
            let period_row: Element<'_, Message> = row(vec![
                make_period_btn("日K", KlinePeriod::Daily, state.kline_period == KlinePeriod::Daily).into(),
                make_period_btn("周K", KlinePeriod::Weekly, state.kline_period == KlinePeriod::Weekly).into(),
                make_period_btn("月K", KlinePeriod::Monthly, state.kline_period == KlinePeriod::Monthly).into(),
                make_period_btn("年K", KlinePeriod::Yearly, state.kline_period == KlinePeriod::Yearly).into(),
            ]).spacing(4).into();

            // ── Time range selector ──
            let range_row: Element<'_, Message> = row(vec![
                make_range_btn("1月", TimeRange::OneMonth, state.time_range == TimeRange::OneMonth).into(),
                make_range_btn("3月", TimeRange::ThreeMonths, state.time_range == TimeRange::ThreeMonths).into(),
                make_range_btn("6月", TimeRange::SixMonths, state.time_range == TimeRange::SixMonths).into(),
                make_range_btn("年初", TimeRange::YearToDate, state.time_range == TimeRange::YearToDate).into(),
                make_range_btn("1年", TimeRange::OneYear, state.time_range == TimeRange::OneYear).into(),
                make_range_btn("2年", TimeRange::TwoYears, state.time_range == TimeRange::TwoYears).into(),
                make_range_btn("5年", TimeRange::FiveYears, state.time_range == TimeRange::FiveYears).into(),
                make_range_btn("全部", TimeRange::Max, state.time_range == TimeRange::Max).into(),
            ]).spacing(4).into();

            // ── Hover tooltip ──
            let tooltip_row: Element<'_, Message> = if let Some(idx) = state.hovered_bar_index {
                if idx < state.daily_bars.len() {
                    let bar = &state.daily_bars[idx];
                    let dt = bar.date.format("%Y-%m-%d").to_string();
                    let clr = |pct: f64| if pct >= 0.0 { style::colors().rise } else { style::colors().fall };
                    let day_change = (bar.close - bar.open) / bar.open * 100.0;
                    let vag = bar.volume as f64 / 10000.0;
                    row![
                        text(dt).size(12.0).color(style::colors().text_accent),
                        text(format!("开 {:.2}", bar.open)).size(12.0).color(style::colors().text_primary),
                        text(format!("高 {:.2}", bar.high)).size(12.0).color(style::colors().rise),
                        text(format!("低 {:.2}", bar.low)).size(12.0).color(style::colors().fall),
                        text(format!("收 {:.2}", bar.close)).size(12.0).color(style::colors().text_primary),
                        text(format!("{:.2}%", day_change)).size(12.0).color(clr(day_change)),
                        text(format!("量 {:.0}万", vag)).size(12.0).color(style::colors().text_secondary),
                    ].spacing(16).padding(4).into()
                } else { text("").into() }
            } else { text("").into() };

            // ── Drawing Tools ──
            let tool_btn = |mode: crate::state::DrawingToolMode, label: &'static str| {
                let active = state.drawing_tool_mode == mode;
                let btn = button(text(label).size(10.0))
                    .on_press(Message::SetDrawingToolMode(mode))
                    .padding(3);
                if active { btn.style(active_period_style()) } else { btn.style(inactive_btn_style()) }
            };
            
            // Pending drawing indicator
            let draw_hint: Element<'_, Message> = if state.pending_drawing.is_some() {
                text("点击K线图选择终点").size(12.0).color(Color::from_rgb(0.8, 0.3, 0.8)).into()
            } else {
                text("").into()
            };
            
            let draw_btn: Element<'_, Message> = {
                let mut items: Vec<Element<'_, Message>> = Vec::new();
                items.push(tool_btn(crate::state::DrawingToolMode::HorizontalLine, "水平线").into());
                items.push(tool_btn(crate::state::DrawingToolMode::TrendLine, "趋势线").into());
                items.push(tool_btn(crate::state::DrawingToolMode::Ray, "射线").into());
                items.push(tool_btn(crate::state::DrawingToolMode::ParallelChannel, "平行通道").into());
                items.push(button(text("清除").size(10.0)).on_press(Message::ClearDrawingLines).padding(3).style(inactive_btn_style()).into());
                if !state.drawing_lines.is_empty() {
                    items.push(text(format!("{}条", state.drawing_lines.len())).size(11.0).color(style::colors().text_accent).into());
                }
                items.push(draw_hint);
                row(items).spacing(4).into()
            };

            // ── Chart ──
            let chart_element: Element<'static, Message> = if state.intraday_period.is_some() && !state.intraday_bars.is_empty() {
                // Use dedicated intraday chart (line chart + volume)
                IntradayCanvas::new(state.intraday_bars.clone(), state.hovered_bar_index).into_element()
            } else if !state.daily_bars.is_empty() {
                let filtered = filter_bars(&state.daily_bars, state.time_range);
                let period = state.kline_period;
                let aggregated = aggregate_bars(&filtered, period);
                CandlestickCanvas::new(aggregated, state.time_range, state.zoom_level, state.hovered_bar_index, state.pan_offset, state.drawing_lines.clone(), &state.active_indicators, &state.indicator_params, state.drawing_tool_mode).into_element()
            } else {
                text("").into()
            };

            // ── Export Button ──
            let export_btn: Element<'_, Message> = if !state.daily_bars.is_empty() {
                button(text("导出CSV").size(10.0))
                    .on_press(Message::ExportCSV)
                    .padding(3)
                    .style(inactive_btn_style())
                    .into()
            } else { text("").into() };

            let content = column![
                title_elem,
                price_summary,
                ma_legend,
                text("").size(4.0),
                period_row,
                range_row,
                text("").size(2.0),
                tooltip_row,
                row![draw_btn, export_btn].spacing(8),
                chart_element,
            ].spacing(2).padding(16);

            container(content).width(Fill).height(Fill).into()
        }
    }
}

fn metric(label: &str, value: String, size: f32, label_color: Color, value_color: Color) -> Element<'_, Message> {
    column![
        text(label).size(12.0).color(label_color),
        text(value).size(size).color(value_color),
    ].into()
}

fn filter_bars(bars: &[stock_vision_data_model::DailyBar], range: TimeRange) -> Vec<stock_vision_data_model::DailyBar> {
    if range == TimeRange::Max {
        return bars.to_vec();
    }
    let cutoff = chrono::Utc::now().date_naive() - chrono::Duration::days(range.days());
    bars.iter().filter(|b| b.date >= cutoff).cloned().collect()
}

fn aggregate_bars(bars: &[stock_vision_data_model::DailyBar], period: KlinePeriod) -> Vec<stock_vision_data_model::DailyBar> {
    if period == KlinePeriod::Daily {
        return bars.to_vec();
    }
    use chrono::Datelike;
    let mut result: Vec<stock_vision_data_model::DailyBar> = Vec::new();
    let mut current: Option<stock_vision_data_model::DailyBar> = None;

    for bar in bars {
        let key = match period {
            KlinePeriod::Weekly => format!("{}-W{:02}", bar.date.iso_week().year(), bar.date.iso_week().week()),
            KlinePeriod::Monthly => format!("{}-{:02}", bar.date.year(), bar.date.month()),
            KlinePeriod::Yearly => bar.date.year().to_string(),
            _ => String::new(),
        };

        let should_break = match &current {
            Some(c) => {
                let c_key = match period {
                    KlinePeriod::Weekly => format!("{}-W{:02}", c.date.iso_week().year(), c.date.iso_week().week()),
                    KlinePeriod::Monthly => format!("{}-{:02}", c.date.year(), c.date.month()),
                    KlinePeriod::Yearly => c.date.year().to_string(),
                    _ => String::new(),
                };
                c_key != key
            }
            None => false,
        };

        if should_break {
            if let Some(c) = current.take() {
                result.push(c);
            }
        }

        match &mut current {
            Some(c) => {
                c.high = c.high.max(bar.high);
                c.low = c.low.min(bar.low);
                c.close = bar.close;
                c.volume += bar.volume;
                c.amount += bar.amount;
            }
            None => {
                current = Some(stock_vision_data_model::DailyBar {
                    code: bar.code.clone(),
                    date: bar.date,
                    open: bar.open,
                    high: bar.high,
                    low: bar.low,
                    close: bar.close,
                    volume: bar.volume,
                    amount: bar.amount,
                    change_pct: None,
                });
            }
        }
    }
    if let Some(c) = current {
        result.push(c);
    }
    result
}
