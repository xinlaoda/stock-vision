use crate::state::AppState;
use crate::app::Message;
use iced::widget::{container, text, Column};
use iced::{Element, Length};

pub fn view(_state: &AppState) -> Element<'_, Message> {
    let content: Column<'_, Message> = Column::new()
        .push(text("设置").size(18))
        .push(text("数据源配置、外观主题等将在后续版本实现").size(14).style(iced::Color::from_rgb(0.5, 0.5, 0.6)))
        .spacing(8)
        .padding(16);

    container(content).width(Length::Fill).height(Length::Fill).into()
}
