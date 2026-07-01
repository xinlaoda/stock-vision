use crate::state::{AppState, KlinePeriod, TimeRange};
use crate::app::Message;
use crate::ui::style;
use crate::ui::charts::CandlestickCanvas;
use iced::widget::{button, column, container, row, text};
use iced::{Color, Element, Fill};
use iced::widget::button::Status;
use iced::Theme;

fn period_style(is_active: bool) -> impl Fn(&Theme, Status) -> iced::widget::button::Style {
    move |_t: &Theme, _s: Status| {
        if is_active {
            iced::widget::button::Style {
                background: Some(style::palette::ACCENT.into()),
                text_color: style::palette::TEXT_PRIMARY,
                ..Default::default()
            }
        } else {
            iced::widget::button::Style::default()
        }
    }
}

fn range_style(is_active: bool) -> impl Fn(&Theme, Status) -> iced::widget::button::Style {
    move |_t: &Theme, _s: Status| {
        if is_active {
            iced::widget::button::Style {
                background: Some(style::palette::BG_LIGHT.into()),
                text_color: style::palette::TEXT_ACCENT,
                ..Default::default()
            }
        } else {
            iced::widget::button::Style::default()
        }
    }
}

fn make_period_btn(label: &'static str, period: KlinePeriod, is_active: bool) -> iced::widget::Button<'static, Message> {
    let btn = button(text(label).size(13.0))
        .on_press(Message::SetKlinePeriod(period))
        .padding(4);
    if is_active {
        btn.style(period_style(true))
    } else {
        btn.style(period_style(false))
    }
}

fn make_range_btn(label: &'static str, range: TimeRange, is_active: bool) -> iced::widget::Button<'static, Message> {
    let btn = button(text(label).size(12.0))
        .on_press(Message::SetTimeRange(range))
        .padding(4);
    if is_active {
        btn.style(range_style(true))
    } else {
        btn.style(range_style(false))
    }
}

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
            // ── Title (bright orange) ──
            let title_color = Color::from_rgb(1.0, 0.65, 0.0);
            let title = format!("{}  {}", state.stock_name.as_deref().unwrap_or(code), code);
            let title_elem = text(title).size(22.0).color(title_color);

            // ── Price summary ──
            let price_summary: Element<'_, Message> = if !state.daily_bars.is_empty() {
                let latest = &state.daily_bars[state.daily_bars.len() - 1];
                let change_pct = (latest.close - latest.open) / latest.open * 100.0;
                let color = if change_pct >= 0.0 { style::palette::RISE } else { style::palette::FALL };
                row![
                    metric("最新价", format!("{:.2}", latest.close), 28.0, color),
                    metric("涨幅", format!("{:.2}%", change_pct), 18.0, color),
                    metric("开盘", format!("{:.2}", latest.open), 18.0, style::palette::TEXT_PRIMARY),
                    metric("最高", format!("{:.2}", latest.high), 18.0, style::palette::TEXT_PRIMARY),
                    metric("最低", format!("{:.2}", latest.low), 18.0, style::palette::TEXT_PRIMARY),
                    metric("成交量", format!("{:.0}万", latest.volume / 10000.0), 18.0, style::palette::TEXT_PRIMARY),
                ].spacing(24).into()
            } else {
                text("正在加载数据...").size(14.0).color(style::palette::TEXT_SECONDARY).into()
            };

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
                make_range_btn("1年", TimeRange::OneYear, state.time_range == TimeRange::OneYear).into(),
                make_range_btn("2年", TimeRange::TwoYears, state.time_range == TimeRange::TwoYears).into(),
                make_range_btn("5年", TimeRange::FiveYears, state.time_range == TimeRange::FiveYears).into(),
                make_range_btn("全部", TimeRange::Max, state.time_range == TimeRange::Max).into(),
            ]).spacing(4).into();

            // ── Chart ──
            let chart_element: Element<'static, Message> = if !state.daily_bars.is_empty() {
                let filtered = filter_bars(&state.daily_bars, state.time_range);
                let period = state.kline_period;
                let aggregated = aggregate_bars(&filtered, period);
                CandlestickCanvas::new(aggregated, state.time_range, state.zoom_level).into_element()
            } else {
                text("").into()
            };

            let content = column![
                title_elem,
                price_summary,
                text("").size(4.0),
                period_row,
                range_row,
                text("").size(2.0),
                chart_element,
            ].spacing(2).padding(16);

            container(content).width(Fill).height(Fill).into()
        }
    }
}

fn metric(label: &str, value: String, size: f32, color: Color) -> Element<'_, Message> {
    column![
        text(label).size(12.0).color(style::palette::TEXT_SECONDARY),
        text(value).size(size).color(color),
    ].into()
}

/// Filter bars based on time range
fn filter_bars(bars: &[stock_vision_data_model::DailyBar], range: TimeRange) -> Vec<stock_vision_data_model::DailyBar> {
    if range == TimeRange::Max {
        return bars.to_vec();
    }
    let cutoff = chrono::Utc::now().date_naive() - chrono::Duration::days(range.days());
    bars.iter().filter(|b| b.date >= cutoff).cloned().collect()
}

/// Aggregate daily bars into weekly/monthly/yearly
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
