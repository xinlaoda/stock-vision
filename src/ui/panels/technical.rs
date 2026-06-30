use crate::state::AppState;
use crate::app::Message;
use iced::widget::{container, text, Column};
use iced::{Element, Length};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content: Column<'_, Message> = Column::new().spacing(8).padding(16);

    let content = match &state.selected_stock {
        Some(_) => content
            .push(text("技术分析").size(18))
            .push(text("技术指标和量价分析将在 Phase 2 实现").size(14).style(iced::Color::from_rgb(0.5, 0.5, 0.6))),
        None => content.push(
            text("请选择一只股票查看技术分析").size(14).style(iced::Color::from_rgb(0.5, 0.5, 0.6)),
        ),
    };

    container(content).width(Length::Fill).height(Length::Fill).into()
}
