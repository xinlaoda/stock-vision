use crate::state::AppState;
use crate::app::Message;
use crate::ui::style;
use iced::widget::{column, container, row, text, Column};
use iced::{Color, Element, Fill};

/// Market index data structure
struct MarketIndex {
    name: &'static str,
    code: &'static str,
    price: &'static str,
    change: &'static str,
    change_pct: &'static str,
    is_up: bool,
}

/// Static market indices for home page display
/// In a full implementation, these would be fetched from API
const MARKET_INDICES: &[MarketIndex] = &[
    MarketIndex { name: "上证指数", code: "000001.SH", price: "--", change: "--", change_pct: "--", is_up: true },
    MarketIndex { name: "深证成指", code: "399001.SZ", price: "--", change: "--", change_pct: "--", is_up: true },
    MarketIndex { name: "创业板指", code: "399006.SZ", price: "--", change: "--", change_pct: "--", is_up: true },
    MarketIndex { name: "科创50", code: "000688.SH", price: "--", change: "--", change_pct: "--", is_up: true },
];

pub fn view(state: &AppState) -> Element<'_, Message> {
    let now_str = state.current_time.format("%Y-%m-%d %H:%M:%S").to_string();

    let mut content = Column::new().spacing(12).padding(24);

    // ── Welcome header ──
    content = content.push(
        text("Stock Vision").size(32.0).color(Color::from_rgb(1.0, 0.65, 0.0))
    );
    content = content.push(
        text("A股行情分析与投资工具").size(16.0).color(style::palette::TEXT_SECONDARY)
    );
    content = content.push(text(now_str).size(14.0).color(style::palette::TEXT_ACCENT));
    content = content.push(text("").size(8.0));

    // ── Market Overview ──
    content = content.push(
        text("市场概况").size(20.0).color(style::palette::TEXT_PRIMARY)
    );

    // Table header
    content = content.push(
        row![
            text("指数").width(120).size(14.0).color(style::palette::TEXT_SECONDARY),
            text("最新价").width(100).size(14.0).color(style::palette::TEXT_SECONDARY),
            text("涨跌额").width(100).size(14.0).color(style::palette::TEXT_SECONDARY),
            text("涨跌幅").width(80).size(14.0).color(style::palette::TEXT_SECONDARY),
        ].spacing(4)
    );

    // Table rows
    for idx in MARKET_INDICES {
        let ch_color = if idx.is_up { style::palette::RISE } else { style::palette::FALL };
        content = content.push(
            row![
                text(idx.name).width(120).size(14.0).color(style::palette::TEXT_PRIMARY),
                text(idx.price).width(100).size(14.0).color(style::palette::TEXT_PRIMARY),
                text(idx.change).width(100).size(14.0).color(ch_color),
                text(idx.change_pct).width(80).size(14.0).color(ch_color),
            ].spacing(4)
        );
    }

    content = content.push(text("").size(16.0));

    // ── Quick start guide ──
    content = content.push(
        text("快速开始").size(20.0).color(style::palette::TEXT_PRIMARY)
    );

    let tips = vec![
        "🔍 在左侧搜索框输入股票代码或名称，搜索并选择股票",
        "📈 查看 K 线走势、成交量、MACD 等技术指标",
        "📋 分析公司基本面财务数据",
        "📊 管理自选股列表，快速切换关注的股票",
        "🖱️ 鼠标滚轮缩放K线，点击添加画线参考",
    ];
    for tip in tips {
        content = content.push(text(tip).size(14.0).color(style::palette::TEXT_SECONDARY));
    }

    container(content).width(Fill).height(Fill).into()
}
