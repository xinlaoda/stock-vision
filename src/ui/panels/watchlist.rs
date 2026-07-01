use crate::state::AppState;
use crate::app::Message;
use crate::ui::style;
use iced::widget::{button, column, container, row, scrollable, text, Column, Text};
use iced::{Element, Length};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content: Column<'_, Message> = Column::new().spacing(4).padding(16);

    let content = content.push(text("自选股").size(22).style(style::palette::TEXT_PRIMARY));
    let content = content.push(text("").size(4));

    if state.watchlist.is_empty() {
        let content = content.push(
            text("还没有自选股\n搜索股票后点击「+ 加入自选」添加")
                .size(14)
                .style(style::palette::TEXT_SECONDARY),
        );
        container(content).width(Length::Fill).height(Length::Fill).into()
    } else {
        let mut content = content;
        for stock in &state.watchlist {
            let label = format!("{}.{}  {}", stock.exchange.prefix(), stock.code, stock.name);
            content = content.push(
                row![
                    button(text(&label).size(14).style(style::palette::TEXT_PRIMARY))
                        .on_press(Message::SearchResultSelected(stock.clone()))
                        .style(iced::theme::Button::Custom(Box::new(style::NavButton)))
                        .width(Length::Fill)
                        .padding(8),
                    button(text("✕").size(13).style(style::palette::RISE))
                        .on_press(Message::RemoveFromWatchlist(stock.code.clone()))
                        .style(iced::theme::Button::Custom(Box::new(style::SearchResultButton)))
                        .padding(8),
                ]
                .spacing(4)
                .align_items(iced::Alignment::Center),
            );
        }
        container(scrollable(content)).width(Length::Fill).height(Length::Fill).into()
    }
}
