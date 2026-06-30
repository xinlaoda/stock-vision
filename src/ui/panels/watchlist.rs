use crate::state::AppState;
use crate::app::Message;
use iced::widget::{container, text, Column};
use iced::{Element, Length};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content: Column<'_, Message> = Column::new().spacing(8).padding(16);

    let content = if state.watchlists.is_empty() {
        content.push(
            text("还没有添加自选股\n请搜索股票并添加到自选")
                .size(14)
                .style(iced::Color::from_rgb(0.5, 0.5, 0.6)),
        )
    } else {
        content
    };

    container(content).width(Length::Fill).height(Length::Fill).into()
}
