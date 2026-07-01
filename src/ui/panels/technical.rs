use crate::state::AppState;
use crate::app::Message;
use crate::ui::style;
use iced::widget::{column, container, text, Column};
use iced::{Element, Fill};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content: Column<'_, Message> = Column::new().spacing(8).padding(16);
    let content = match &state.selected_stock {
        Some(code) => {
            let name = state.stock_name.as_deref().unwrap_or(code);
            content
                .push(text(format!("{} {}", name, code)).size(22.0).color(style::palette::TEXT_PRIMARY))
                .push(text("").size(4.0))
                .push(text("技术分析即将推出").size(16.0).color(style::palette::TEXT_PRIMARY))
                .push(text("已实现的技术指标代码（待绑定到UI）：SMA / EMA / MACD / RSI / Bollinger Bands")
                    .size(14.0).color(style::palette::TEXT_SECONDARY))
        }
        None => content
            .push(text("技术分析").size(22.0).color(style::palette::TEXT_PRIMARY))
            .push(text("请先搜索并选择一只股票").size(14.0).color(style::palette::TEXT_SECONDARY)),
    };
    container(content).width(Fill).height(Fill).into()
}
