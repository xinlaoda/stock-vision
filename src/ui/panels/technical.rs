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
            background: Some(style::palette::ACCENT.into()),
            text_color: Color::WHITE,
            ..Default::default()
        }
    } else {
        iced::widget::button::Style {
            background: Some(style::palette::BG_LIGHT.into()),
            text_color: style::palette::TEXT_SECONDARY,
            ..Default::default()
        }
    }
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    match &state.selected_stock {
        None => {
            let content = column![
                text("请搜索并选择一只股票").size(18.0).color(style::palette::TEXT_PRIMARY),
                text("搜索后点击「技术分析」查看技术指标")
                    .size(14.0).color(style::palette::TEXT_SECONDARY),
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
                text("指标选择 (点击切换开/关)").size(16.0).color(style::palette::TEXT_PRIMARY),
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
                text("当前激活的指标").size(16.0).color(style::palette::TEXT_PRIMARY),
            );

            if state.active_indicators.is_empty() {
                content = content.push(
                    text("没有激活的指标").size(14.0).color(style::palette::TEXT_SECONDARY),
                );
            } else {
                for indicator in &state.active_indicators {
                    let status_text = if indicator.is_chart_overlay() {
                        format!("• {} — 叠加在K线图上", indicator.label())
                    } else {
                        format!("• {} — 显示在副图区域", indicator.label())
                    };
                    content = content.push(
                        text(status_text).size(13.0).color(style::palette::TEXT_ACCENT),
                    );
                }
            }

            // ── Current Values (computed on the fly) ──
            if !state.daily_bars.is_empty() {
                content = content.push(text("").size(8.0));
                content = content.push(render_divider());
                content = content.push(
                    text("当前技术指标值").size(16.0).color(style::palette::TEXT_PRIMARY),
                );

                for indicator in &state.active_indicators {
                    if let Some(computed) = compute_indicator(*indicator, &state.daily_bars) {
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
                            text(info).size(13.0).color(style::palette::TEXT_PRIMARY),
                        );
                    }
                }
            }

            // ── Description ──
            content = content.push(text("").size(12.0));
            content = content.push(render_divider());
            content = content.push(
                text("说明").size(16.0).color(style::palette::TEXT_PRIMARY),
            );
            let descriptions = [
                "• MACD: 趋势跟踪指标，显示DIF(白)、DEA(黄)线和MACD柱状图",
                "• KDJ: 随机指标，K(黄)线为快速线，D(蓝)线为慢速线，J(粉)为方向线",
                "• RSI: 相对强弱指标，值>70超买，<30超卖",
                "• BOLL: 布林带，中轨为SMA(20)，上下轨为±2倍标准差",
            ];
            for desc in &descriptions {
                content = content.push(
                    text(*desc).size(13.0).color(style::palette::TEXT_SECONDARY),
                );
            }

            // ── Chart link ──
            content = content.push(text("").size(12.0));
            content = content.push(
                text("👉 切换到「行情走势」页面查看指标在K线图上的显示效果")
                    .size(14.0).color(style::palette::TEXT_ACCENT),
            );

            container(scrollable(content)).width(Fill).height(Fill).into()
        }
    }
}

fn render_divider() -> Element<'static, Message> {
    container(text("")).width(Fill).height(1).style(|_: &Theme| iced::widget::container::Style {
        background: Some(style::palette::BG_LIGHT.into()),
        ..Default::default()
    }).into()
}

fn format_val(v: Option<f64>) -> String {
    match v {
        Some(val) if val.is_nan() => "-".to_string(),
        Some(val) => format!("{:.2}", val),
        None => "-".to_string(),
    }
}
