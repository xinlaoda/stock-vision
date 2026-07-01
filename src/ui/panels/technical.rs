/// Technical Analysis Panel — select and configure indicators.

use crate::app::Message;
use crate::services::indicator_service::{compute_indicator, IndicatorType};
use crate::state::AppState;
use crate::ui::style;
use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Color, Element, Fill};
use iced::widget::button::Status;
use iced::Theme;

fn toggle_btn_style(active: bool) -> iced::widget::button::Style {
    if active {
        iced::widget::button::Style {
            background: Some(style::colors().accent.into()),
            text_color: Color::WHITE,
            ..Default::default()
        }
    } else {
        iced::widget::button::Style {
            background: Some(style::colors().bg_light.into()),
            text_color: style::colors().text_secondary,
            ..Default::default()
        }
    }
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    match &state.selected_stock {
        None => {
            let content = column![
                text("请搜索并选择一只股票").size(18.0).color(style::colors().text_primary),
                text("搜索后点击「技术分析」查看技术指标")
                    .size(14.0).color(style::colors().text_secondary),
            ].spacing(8).padding(16);
            return container(content).width(Fill).height(Fill).into();
        }
        Some(code) => {
            let name = state.stock_name.as_deref().unwrap_or(code);
            let mut content = Column::new().spacing(12).padding(24);

            // Title
            content = content.push(
                text(format!("{}  {} — 技术分析", name, code))
                    .size(22.0).color(Color::from_rgb(1.0, 0.65, 0.0)),
            );

            // ── Indicator Buttons ──
            const ALL_INDICATORS: [IndicatorType; 4] = [
                IndicatorType::MACD,
                IndicatorType::KDJ,
                IndicatorType::RSI,
                IndicatorType::BollingerBands,
            ];

            content = content.push(text("").size(4.0));
            content = content.push(
                text("指标选择 (点击切换开/关)").size(16.0).color(style::colors().text_primary),
            );

            let mut btn_row = row![].spacing(8);
            for indicator in ALL_INDICATORS.iter() {
                let is_active = state.active_indicators.contains(indicator);
                let btn = button(text(indicator.label()).size(14.0))
                    .on_press(Message::ToggleIndicator(*indicator))
                    .style(move |_: &Theme, _: Status| toggle_btn_style(is_active))
                    .padding(10);
                btn_row = btn_row.push(btn);
            }
            content = content.push(btn_row);

            // ── Division ──
            content = content.push(text("").size(8.0));
            content = content.push(render_divider());

            // ── Active Indicator Status ──
            content = content.push(
                text("当前激活的指标").size(16.0).color(style::colors().text_primary),
            );

            if state.active_indicators.is_empty() {
                content = content.push(
                    text("没有激活的指标").size(14.0).color(style::colors().text_secondary),
                );
            } else {
                for indicator in &state.active_indicators {
                    let status_text = if indicator.is_chart_overlay() {
                        format!("• {} — 叠加在K线图上", indicator.label())
                    } else {
                        format!("• {} — 显示在副图区域", indicator.label())
                    };
                    content = content.push(
                        text(status_text).size(13.0).color(style::colors().text_accent),
                    );
                }
            }

            // ── Current Values (computed on the fly) ──
            if !state.daily_bars.is_empty() {
                content = content.push(text("").size(8.0));
                content = content.push(render_divider());
                content = content.push(
                    text("当前技术指标值").size(16.0).color(style::colors().text_primary),
                );

                for indicator in &state.active_indicators {
                    if let Some(computed) = compute_indicator(*indicator, &state.daily_bars, &state.indicator_params) {
                        let last_idx = computed.line1.len().saturating_sub(1);
                        let val1 = computed.line1.get(last_idx).copied().flatten();
                        let val2 = computed.line2.get(last_idx).copied().flatten();
                        let val3 = computed.line3.get(last_idx).copied().flatten();

                        let mut info = format!("{}: {}={}", indicator.label(), computed.line1_label, format_val(val1));
                        if !computed.line2_label.is_empty() {
                            info.push_str(&format!("  {}={}", computed.line2_label, format_val(val2)));
                        }
                        if !computed.line3_label.is_empty() {
                            info.push_str(&format!("  {}={}", computed.line3_label, format_val(val3)));
                        }
                        content = content.push(
                            text(info).size(13.0).color(style::colors().text_primary),
                        );
                    }
                }
            }

            // ── Parameter Configuration ──
            content = content.push(text("").size(8.0));
            content = content.push(render_divider());
            content = content.push(
                text("参数配置 (点击数字调整)").size(16.0).color(style::colors().text_primary),
            );

            let p = &state.indicator_params;

            // MA Parameters
            content = content.push(text("移动均线(MA)").size(14.0).color(style::colors().text_accent));
            content = content.push(param_row("MA1周期", p.ma_periods[0], 2, 120, |v| Message::SetMAPeriod(0, v)));
            content = content.push(param_row("MA2周期", p.ma_periods[1], 2, 120, |v| Message::SetMAPeriod(1, v)));
            content = content.push(param_row("MA3周期", p.ma_periods[2], 2, 120, |v| Message::SetMAPeriod(2, v)));
            content = content.push(param_row("MA4周期", p.ma_periods[3], 2, 120, |v| Message::SetMAPeriod(3, v)));
            content = content.push(param_row("量MA周期", p.vol_ma_period, 2, 60, |v| Message::SetVolMAPeriod(v)));

            // MACD Parameters
            content = content.push(text("").size(4.0));
            content = content.push(text("MACD").size(14.0).color(style::colors().text_accent));
            content = content.push(param_row("快线(EMA)", p.macd_fast, 2, 60, |v| Message::SetMACDFast(v)));
            content = content.push(param_row("慢线(EMA)", p.macd_slow, 5, 120, |v| Message::SetMACDSlow(v)));
            content = content.push(param_row("信号(MA)", p.macd_signal, 2, 60, |v| Message::SetMACDSignal(v)));

            // BOLL Parameters
            content = content.push(text("").size(4.0));
            content = content.push(text("布林带(BOLL)").size(14.0).color(style::colors().text_accent));
            content = content.push(param_row("周期", p.boll_period, 2, 120, |v| Message::SetBOLLPeriod(v)));
            content = content.push(param_row_f64("标准差", p.boll_std, 0.5, 5.0, 0.5, |v| Message::SetBOLLStd(v)));

            // KDJ Parameters
            content = content.push(text("").size(4.0));
            content = content.push(text("KDJ").size(14.0).color(style::colors().text_accent));
            content = content.push(param_row("RSV周期(N)", p.kdj_n, 2, 60, |v| Message::SetKDJ_N(v)));
            content = content.push(param_row("K平滑(M1)", p.kdj_m1, 2, 30, |v| Message::SetKDJ_M1(v)));
            content = content.push(param_row("D平滑(M2)", p.kdj_m2, 2, 30, |v| Message::SetKDJ_M2(v)));

            // RSI Parameters
            content = content.push(text("").size(4.0));
            content = content.push(text("RSI").size(14.0).color(style::colors().text_accent));
            content = content.push(param_row("周期", p.rsi_period, 2, 60, |v| Message::SetRSIPeriod(v)));


            // ── Description ──
            content = content.push(text("").size(12.0));
            content = content.push(render_divider());
            content = content.push(
                text("说明").size(16.0).color(style::colors().text_primary),
            );
            let descriptions = [
                "• MACD: 趋势跟踪指标，显示DIF(白)、DEA(黄)线和MACD柱状图",
                "• KDJ: 随机指标，K(黄)线为快速线，D(蓝)线为慢速线，J(粉)为方向线",
                "• RSI: 相对强弱指标，值>70超买，<30超卖",
                "• BOLL: 布林带，中轨为SMA(20)，上下轨为±2倍标准差",
            ];
            for desc in &descriptions {
                content = content.push(
                    text(*desc).size(13.0).color(style::colors().text_secondary),
                );
            }

            // ── Chart link ──
            content = content.push(text("").size(12.0));
            content = content.push(
                text("👉 切换到「行情走势」页面查看指标在K线图上的显示效果")
                    .size(14.0).color(style::colors().text_accent),
            );

            container(scrollable(content)).width(Fill).height(Fill).into()
        }
    }
}

fn render_divider() -> Element<'static, Message> {
    container(text("")).width(Fill).height(1).style(|_: &Theme| iced::widget::container::Style {
        background: Some(style::colors().bg_light.into()),
        ..Default::default()
    }).into()
}

/// Parameter adjustment button: decrease / value / increase
fn param_row(label: &'static str, value: usize, min: usize, max: usize, msg: impl Fn(usize) -> Message + 'static) -> Element<'static, Message> {
    use iced::widget::{row, text, button, container};
    
    let dec = button(text("-").size(14.0))
        .on_press(msg(value.saturating_sub(1).max(min)))
        .padding(4).width(28);
    let inc = button(text("+").size(14.0))
        .on_press(msg((value + 1).min(max)))
        .padding(4).width(28);
    let val = container(
        text(value.to_string()).size(14.0).color(style::colors().text_primary)
    ).width(40).center_x(Fill);
    
    row![
        text(label).size(13.0).color(style::colors().text_secondary).width(100),
        dec, val, inc,
    ].spacing(4).align_y(iced::alignment::Vertical::Center).into()
}

/// Parameter adjustment button for f64 values
fn param_row_f64(label: &'static str, value: f64, min: f64, max: f64, step: f64, msg: impl Fn(f64) -> Message + 'static) -> Element<'static, Message> {
    use iced::widget::{row, text, button, container};
    
    let dec = button(text("-").size(14.0))
        .on_press(msg((value - step).max(min)))
        .padding(4).width(28);
    let inc = button(text("+").size(14.0))
        .on_press(msg((value + step).min(max)))
        .padding(4).width(28);
    let val = container(
        text(format!("{:.1}", value)).size(14.0).color(style::colors().text_primary)
    ).width(40).center_x(Fill);
    
    row![
        text(label).size(13.0).color(style::colors().text_secondary).width(100),
        dec, val, inc,
    ].spacing(4).align_y(iced::alignment::Vertical::Center).into()
}

fn format_val(v: Option<f64>) -> String {
    match v {
        Some(val) if val.is_nan() => "-".to_string(),
        Some(val) => format!("{:.2}", val),
        None => "-".to_string(),
    }
}
