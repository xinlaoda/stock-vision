use crate::state::AppState;
use crate::app::Message;
use crate::ui::style;
use iced::widget::{container, text, Column};
use iced::{Element, Length};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content: Column<'_, Message> = Column::new().spacing(8).padding(16);

    let content = if state.watchlists.is_empty() {
        content
            .push(text("自选股").size(22))
            .push(text("").size(4))
            .push(
                text("还没有添加自选股\n搜索股票并选择后，计划添加「加入自选」功能")
                    .size(14)
                    .style(style::palette::TEXT_SECONDARY),
            )
    } else {
        content
    };

    container(content).width(Length::Fill).height(Length::Fill).into()
}
