use crate::state::AppState;
use crate::app::Message;
use crate::ui::style;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Color, Element, Fill};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mut content = Column::new().spacing(12).padding(24);

    // ── Header ──
    content = content.push(
        text("设置").size(24.0).color(style::colors().text_primary),
    );
    content = content.push(text("").size(4.0));

    // ── 数据源信息 ──
    content = content.push(
        text("数据源配置").size(18.0).color(style::colors().text_primary),
    );
    content = content.push(
        text("当前支持 A股 + 美股，多数据源自动 Fallback")
            .size(14.0).color(style::colors().text_secondary),
    );
    content = content.push(
        text("• A股日K线: 腾讯财经 (web.ifzq.gtimg.cn)")
            .size(13.0).color(style::colors().text_secondary),
    );
    content = content.push(
        text("• A股搜索/基本面: 东方财富 (eastmoney.com)")
            .size(13.0).color(style::colors().text_secondary),
    );
    content = content.push(
        text("• A股分时数据: 腾讯 + 东方财富 Fallback")
            .size(13.0).color(style::colors().text_secondary),
    );
    content = content.push(text("").size(4.0));

    // ── 美股数据源 ──
    content = content.push(
        text("美股数据源").size(18.0).color(style::colors().text_primary),
    );
    content = content.push(
        text("• 美股K线/搜索: Yahoo Finance (免费/无需注册)")
            .size(13.0).color(style::colors().text_secondary),
    );

    // Finnhub API Key - editable input
    content = content.push(text("").size(2.0));
    content = content.push(
        text("Finnhub API Key（提供美股基本面/估值数据）").size(14.0).color(style::colors().text_primary),
    );
    
    let finnhub_key = state.finnhub_api_key.clone();
    let key_display = if finnhub_key.is_empty() { 
        String::from("输入 Finnhub API Key...") 
    } else {
        // Show masked key
        let visible = if finnhub_key.len() > 8 { &finnhub_key[..4] } else { &finnhub_key[..finnhub_key.len().min(4)] };
        format!("{}****", visible)
    };
    
    content = content.push(
        row![
            iced::widget::text_input(
                "输入 Finnhub API Key（留空禁用）", 
                &key_display
            )
                .on_input(Message::FinnhubKeyChanged)
                .on_submit(Message::FinnhubKeySubmitted)
                .padding(6).size(13.0)
                .width(350.0),
        ].spacing(4)
    );
    
    let finnhub_status = if state.finnhub_available {
        "✅ Finnhub 已启用"
    } else if !state.finnhub_api_key.is_empty() {
        "⏳ Finnhub 已配置，重启后生效"
    } else {
        "❌ Finnhub 未配置（使用 Yahoo Finance 作为美股数据源）"
    };
    content = content.push(
        text(finnhub_status)
            .size(12.0).color(style::colors().text_secondary),
    );
    content = content.push(text("").size(8.0));

    // ── 外观设置 ──
    content = content.push(
        text("外观设置").size(18.0).color(style::colors().text_primary),
    );
    content = content.push(
        text(format!("当前主题: {}模式", state.theme_mode.label()))
            .size(14.0).color(style::colors().text_secondary),
    );
    let toggle_btn = button(
        text(if state.theme_mode == crate::ui::style::ThemeMode::Dark { "切换到亮色主题" } else { "切换到深色主题" })
            .size(14.0).color(style::colors().text_primary)
    )
        .on_press(Message::ToggleTheme)
        .padding(8)
        .style(|_: &iced::Theme, _: iced::widget::button::Status| iced::widget::button::Style {
            background: Some(style::colors().accent.into()),
            text_color: Color::WHITE,
            ..Default::default()
        });
    content = content.push(toggle_btn);
    content = content.push(text("").size(8.0));

    // ── 缓存管理 ──
    let cache_info = format_cache_info(state);
    content = content.push(
        text("缓存管理").size(18.0).color(style::colors().text_primary),
    );
    content = content.push(
        text(cache_info).size(13.0).color(style::colors().text_secondary),
    );

    // ── 关于 ──
    content = content.push(text("").size(16.0));
    content = content.push(
        text("关于").size(18.0).color(style::colors().text_primary),
    );
    content = content.push(
        text("Stock Vision v0.1.0 — A股+美股行情分析与投资工具")
            .size(13.0).color(style::colors().text_secondary),
    );
    content = content.push(
        text("技术栈: Rust + Iced 0.14 + SQLite")
            .size(13.0).color(style::colors().text_secondary),
    );
    content = content.push(
        text("支持平台: Windows / macOS / Linux")
            .size(13.0).color(style::colors().text_secondary),
    );

    container(scrollable(content)).width(Fill).height(Fill).into()
}

fn format_cache_info(state: &AppState) -> String {
    let mut info = String::from("本地缓存状态:\n");
    info.push_str(&format!("• 日K线数据: {} 支股票\n", count_cached_daily_bars(state)));
    info.push_str(&format!("• 自选股: {} 支\n", state.watchlist.len()));
    info.push_str(&format!("• 浏览历史: {} 条记录\n", state.browse_history.len()));

    if let Some(code) = &state.selected_stock {
        info.push_str(&format!(
            "• 当前 {}: {} 根K线, {} 份财报\n",
            code,
            state.daily_bars.len(),
            state.financial_reports.len(),
        ));
    }

    info
}

fn count_cached_daily_bars(state: &AppState) -> usize {
    // Approximate: we can tell if current stock has cached data
    if state.daily_bars.len() > 0 { 1 } else { 0 }
}
