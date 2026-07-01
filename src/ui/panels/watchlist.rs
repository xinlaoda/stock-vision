use crate::state::AppState;
use crate::app::Message;
use crate::ui::style;
use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Element, Fill};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content: Column<'_, Message> = Column::new().spacing(4).padding(16);
    let content = content.push(text("自选股").size(22.0).color(style::palette::TEXT_PRIMARY));
    let content = content.push(text("").size(4.0));

    if state.watchlist.is_empty() {
        let content = content.push(
            text("还没有自选股\n搜索股票后点击「+ 加入自选」添加")
                .size(14.0).color(style::palette::TEXT_SECONDARY),
        );
        container(content).width(Fill).height(Fill).into()
    } else {
        let mut content = content;
        for stock in &state.watchlist {
            let label = format!("{}.{}  {}", stock.exchange.prefix(), stock.code, stock.name);
            content = content.push(
                row![
                    button(text(label.clone()).size(14.0))
                        .on_press(Message::SearchResultSelected(stock.clone()))
                        .width(Fill).padding(8),
                    button(text("✕").size(13.0))
                        .on_press(Message::RemoveFromWatchlist(stock.code.clone()))
                        .padding(8),
                ].spacing(4)
            );
        }
        container(scrollable(content)).width(Fill).height(Fill).into()
    }
}
