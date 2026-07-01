use crate::state::AppState;
use crate::app::Message;
use crate::ui::style;
use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Color, Element, Fill};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mut content = Column::new().spacing(12).padding(24);

    // ── Header ──
    content = content.push(
        text("设置").size(24.0).color(style::palette::TEXT_PRIMARY),
    );
    content = content.push(text("").size(4.0));

    // ── 数据源信息 ──
    content = content.push(
        text("数据源配置").size(18.0).color(style::palette::TEXT_PRIMARY),
    );
    content = content.push(
        text("当前数据源为 Tencent + EastMoney 双源 Fallback 模式")
            .size(14.0).color(style::palette::TEXT_SECONDARY),
    );
    content = content.push(
        text("• 日K线: 腾讯财经 (web.ifzq.gtimg.cn)")
            .size(13.0).color(style::palette::TEXT_SECONDARY),
    );
    content = content.push(
        text("• 搜索/基本面: 东方财富 (eastmoney.com)")
            .size(13.0).color(style::palette::TEXT_SECONDARY),
    );
    content = content.push(
        text("• 分时数据: 腾讯 + 东方财富 Fallback")
            .size(13.0).color(style::palette::TEXT_SECONDARY),
    );
    content = content.push(text("").size(8.0));

    // ── 外观设置 ──
    content = content.push(
        text("外观设置").size(18.0).color(style::palette::TEXT_PRIMARY),
    );
    content = content.push(
        text("当前主题: 深色模式 (Dark)")
            .size(14.0).color(style::palette::TEXT_SECONDARY),
    );
    content = content.push(
        text("亮色主题将在后续版本支持")
            .size(13.0).color(style::palette::TEXT_SECONDARY),
    );
    content = content.push(text("").size(8.0));

    // ── 缓存管理 ──
    let cache_info = format_cache_info(state);
    content = content.push(
        text("缓存管理").size(18.0).color(style::palette::TEXT_PRIMARY),
    );
    content = content.push(
        text(cache_info).size(13.0).color(style::palette::TEXT_SECONDARY),
    );

    // ── 关于 ──
    content = content.push(text("").size(16.0));
    content = content.push(
        text("关于").size(18.0).color(style::palette::TEXT_PRIMARY),
    );
    content = content.push(
        text("Stock Vision v0.1.0 — A股行情分析与投资工具")
            .size(13.0).color(style::palette::TEXT_SECONDARY),
    );
    content = content.push(
        text("技术栈: Rust + Iced 0.14 + SQLite")
            .size(13.0).color(style::palette::TEXT_SECONDARY),
    );
    content = content.push(
        text("支持平台: Windows / macOS / Linux")
            .size(13.0).color(style::palette::TEXT_SECONDARY),
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
