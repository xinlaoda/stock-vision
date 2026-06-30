use iced::widget::{button, container, row, text, text_input, Column};
use iced::{Element, Length, Sandbox};

use crate::state::{AppState, Panel};
use crate::ui::panels;

#[derive(Debug, Clone)]
pub enum Message {
    SearchChanged(String),
    SearchSubmitted,
    PanelChanged(Panel),
    StockSelected(String, String),
}

pub struct StockVisionApp {
    state: AppState,
}

impl Sandbox for StockVisionApp {
    type Message = Message;

    fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }

    fn title(&self) -> String {
        "Stock Vision - A股分析工具".to_string()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SearchChanged(keyword) => {
                self.state.search_keyword = keyword;
            }
            Message::SearchSubmitted => {}
            Message::PanelChanged(panel) => {
                self.state.active_panel = panel;
            }
            Message::StockSelected(code, name) => {
                self.state.selected_stock = Some(code);
                self.state.stock_name = Some(name);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let main_content = self.view_main_content();

        container(
            row(vec![sidebar, main_content])
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

impl StockVisionApp {
    fn view_sidebar(&self) -> Element<'_, Message> {
        let logo = text("Stock Vision").size(20);

        let search_input = text_input("搜索股票代码或名称...", &self.state.search_keyword)
            .on_submit(Message::SearchSubmitted)
            .padding(8)
            .size(14);

        let panel_buttons = Column::new()
            .push(button("📊 自选股").on_press(Message::PanelChanged(Panel::Watchlist)))
            .push(button("📈 行情走势").on_press(Message::PanelChanged(Panel::Chart)))
            .push(button("📋 基本面分析").on_press(Message::PanelChanged(Panel::Fundamental)))
            .push(button("📐 技术分析").on_press(Message::PanelChanged(Panel::Technical)))
            .push(button("⚙ 设置").on_press(Message::PanelChanged(Panel::Settings)))
            .spacing(4);

        container(
            Column::new()
                .push(logo)
                .push(search_input)
                .push(panel_buttons)
                .spacing(12)
                .padding(16),
        )
        .width(220)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(
            crate::ui::style::SidebarStyle,
        )))
        .into()
    }

    fn view_main_content(&self) -> Element<'_, Message> {
        match self.state.active_panel {
            Panel::Watchlist => panels::watchlist::view(&self.state),
            Panel::Chart => panels::chart::view(&self.state),
            Panel::Fundamental => panels::fundamental::view(&self.state),
            Panel::Technical => panels::technical::view(&self.state),
            Panel::Settings => panels::settings::view(&self.state),
        }
    }
}
