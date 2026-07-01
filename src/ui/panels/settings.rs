use crate::state::AppState;
use crate::app::Message;
use crate::ui::style;
use iced::widget::{column, container, text, Column};
use iced::{Element, Fill};

pub fn view(_state: &AppState) -> Element<'_, Message> {
    let content: Column<'_, Message> = Column::new()
        .push(text("设置").size(22.0).color(style::palette::TEXT_PRIMARY))
        .push(text("").size(4.0))
        .push(text("数据源配置、外观主题等将在后续版本实现")
            .size(14.0).color(style::palette::TEXT_SECONDARY))
        .spacing(8).padding(16);
    container(content).width(Fill).height(Fill).into()
}
