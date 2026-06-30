use iced::widget::{button, column, container, row, text, text_input, Column};
use iced::{Application, Color, Command, Element, Length, Subscription};

use stock_vision_data_model::*;
use stock_vision_data_source::{DataSource, EastMoneySource, TencentSource};
use std::sync::Arc;

use crate::state::{AppState, Panel};
use crate::ui::panels;

#[derive(Debug, Clone)]
pub enum Message {
    SearchInputChanged(String),
    SearchSubmitted,
    SearchResultSelected(Stock),
    SearchResultsLoaded(Vec<Stock>),
    PanelChanged(Panel),
    StockSelected(Stock),
    DailyBarsLoaded(Vec<DailyBar>),
    Error(String),
}

pub struct StockVisionApp {
    state: AppState,
    search_source: Arc<EastMoneySource>,
    kline_source: Arc<TencentSource>,
}

impl Application for StockVisionApp {
    type Message = Message;
    type Theme = iced::Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                state: AppState::new(),
                search_source: Arc::new(EastMoneySource::new()),
                kline_source: Arc::new(TencentSource::new()),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        let base = "Stock Vision - A股分析工具";
        match &self.state.stock_name {
            Some(name) => format!("{} - {}", name, base),
            None => base.to_string(),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SearchInputChanged(keyword) => {
                self.state.search_keyword = keyword;
                self.state.search_results.clear();
                Command::none()
            }
            Message::SearchSubmitted => {
                let keyword = self.state.search_keyword.clone();
                let source = self.search_source.clone();
                Command::perform(
                    async move { source.search_stocks(&keyword).await.unwrap_or_default() },
                    Message::SearchResultsLoaded,
                )
            }
            Message::SearchResultsLoaded(stocks) => {
                self.state.search_results = stocks;
                Command::none()
            }
            Message::SearchResultSelected(stock) => {
                self.state.search_keyword = stock.name.clone();
                self.state.search_results.clear();
                self.state.selected_stock = Some(stock.code.clone());
                self.state.stock_name = Some(stock.name.clone());
                self.state.active_panel = Panel::Chart;
                let code = stock.code.clone();
                let exchange = stock.exchange.clone();
                let source = self.kline_source.clone();
                Command::perform(
                    async move {
                        source
                            .get_daily_bars(&code, exchange, None, None, None)
                            .await
                            .unwrap_or_default()
                    },
                    Message::DailyBarsLoaded,
                )
            }
            Message::PanelChanged(panel) => {
                self.state.active_panel = panel;
                Command::none()
            }
            Message::StockSelected(stock) => {
                self.state.selected_stock = Some(stock.code);
                self.state.stock_name = Some(stock.name);
                Command::none()
            }
            Message::DailyBarsLoaded(bars) => {
                self.state.daily_bars = bars;
                Command::none()
            }
            Message::Error(err) => {
                eprintln!("Error: {}", err);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
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
    fn view_sidebar(&self) -> Element<Message> {
        let logo = text("Stock Vision").size(20);

        let has_results = !self.state.search_results.is_empty();
        let search_input = text_input("搜索股票代码或名称...", &self.state.search_keyword)
            .on_input(Message::SearchInputChanged)
            .on_submit(Message::SearchSubmitted)
            .padding(8)
            .size(14);

        let search_results: Element<Message> = if has_results {
            let list: Vec<Element<Message>> = self.state.search_results.iter().map(|s| {
                let label = format!("{}.{} {}", s.exchange.prefix(), s.code, s.name);
                button(text(label).size(13))
                    .on_press(Message::SearchResultSelected(s.clone()))
                    .style(iced::theme::Button::Text)
                    .width(Length::Fill)
                    .padding(4)
                    .into()
            }).collect();
            column(list).spacing(2).into()
        } else {
            text("").into()
        };

        let panel_buttons = Column::new()
            .push(button("📊 自选股").on_press(Message::PanelChanged(Panel::Watchlist)))
            .push(button("📈 行情走势").on_press(Message::PanelChanged(Panel::Chart)))
            .push(button("📋 基本面分析").on_press(Message::PanelChanged(Panel::Fundamental)))
            .push(button("📐 技术分析").on_press(Message::PanelChanged(Panel::Technical)))
            .push(button("⚙ 设置").on_press(Message::PanelChanged(Panel::Settings)))
            .spacing(4);

        let stock_indicator: Element<Message> = match &self.state.stock_name {
            Some(name) => text(format!("当前: {}", name))
                .size(12)
                .style(Color::from_rgb(0.6, 0.8, 0.6))
                .into(),
            None => text("").into(),
        };

        container(
            Column::new()
                .push(logo)
                .push(search_input)
                .push(search_results)
                .push(stock_indicator)
                .push(panel_buttons)
                .spacing(8)
                .padding(16),
        )
        .width(220)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(
            crate::ui::style::SidebarStyle,
        )))
        .into()
    }

    fn view_main_content(&self) -> Element<Message> {
        match self.state.active_panel {
            Panel::Watchlist => panels::watchlist::view(&self.state),
            Panel::Chart => panels::chart::view(&self.state),
            Panel::Fundamental => panels::fundamental::view(&self.state),
            Panel::Technical => panels::technical::view(&self.state),
            Panel::Settings => panels::settings::view(&self.state),
        }
    }
}
