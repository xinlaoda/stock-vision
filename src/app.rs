use iced::widget::{button, column, container, row, text, text_input, Column};
use iced::{Alignment, Application, Color, Command, Element, Font, Length};

/// Emoji font for Windows (Segoe UI Emoji), Linux fallback
pub const EMOJI_FONT: Font = Font::with_name("Segoe UI Emoji");

use stock_vision_data_model::*;
use stock_vision_data_source::{DataSource, EastMoneySource, TencentSource};
use stock_vision_analysis_core::FinancialAnalyzer;
use std::sync::Arc;

use crate::state::{AppState, Panel};
use crate::ui::{panels, style};

#[derive(Debug, Clone)]
pub enum Message {
    SearchInputChanged(String),
    SearchSubmitted,
    SearchResultSelected(Stock),
    SearchResultsLoaded(Vec<Stock>),
    PanelChanged(Panel),
    DailyBarsLoaded(Vec<DailyBar>),
    FinancialDataLoaded(Vec<FinancialReport>),
    Error(String),
}

pub struct StockVisionApp {
    state: AppState,
    search_source: Arc<EastMoneySource>,
    kline_source: Arc<TencentSource>,
    fin_source: Arc<EastMoneySource>,
    analyzer: FinancialAnalyzer,
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
                fin_source: Arc::new(EastMoneySource::new()),
                analyzer: FinancialAnalyzer,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        let base = "Stock Vision";
        match &self.state.stock_name {
            Some(name) => format!("{} - {}", name, base),
            None => base.to_string(),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            // ── Search ──
            Message::SearchInputChanged(keyword) => {
                self.state.search_keyword = keyword;
                self.state.search_results.clear();
                Command::none()
            }
            Message::SearchSubmitted => {
                let keyword = self.state.search_keyword.clone();
                if keyword.trim().is_empty() {
                    return Command::none();
                }
                let source = self.search_source.clone();
                self.state.search_results.clear();
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
                self.state.daily_bars.clear();
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

            // ── Navigation ──
            Message::PanelChanged(panel) => {
                let is_fundamental = panel == Panel::Fundamental;
                let needs_load = is_fundamental
                    && self.state.selected_stock.is_some()
                    && self.state.financial_reports.is_empty();
                self.state.active_panel = panel;
                if needs_load {
                    let code = self.state.selected_stock.clone().unwrap();
                    let exchange = Exchange::SZ;
                    let source = self.fin_source.clone();
                    Command::perform(
                        async move {
                            source
                                .get_financial_reports(&code, exchange, None)
                                .await
                                .unwrap_or_default()
                        },
                        Message::FinancialDataLoaded,
                    )
                } else {
                    Command::none()
                }
            }

            // ── Data ──
            Message::DailyBarsLoaded(bars) => {
                self.state.daily_bars = bars;
                Command::none()
            }
            Message::FinancialDataLoaded(reports) => {
                self.state.financial_reports = reports;
                if let Some(report) = self.state.financial_reports.first() {
                    let health = self.analyzer.score(
                        report,
                        &ValuationRatios {
                            code: String::new(),
                            date: chrono::Utc::now().date_naive(),
                            pe: Some(15.0),
                            pb: Some(2.0),
                            ps: None,
                            pcf: None,
                            market_cap: None,
                            dividend_yield: None,
                        },
                    );
                    self.state.financial_health = Some(health);
                }
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

        container(row(vec![sidebar, main_content]))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl StockVisionApp {
    fn view_sidebar(&self) -> Element<Message> {
        // ── Search input + button row ──
        let search_row = row![
            text_input("输入代码或名称，回车搜索", &self.state.search_keyword)
                .on_input(Message::SearchInputChanged)
                .on_submit(Message::SearchSubmitted)
                .padding(8)
                .size(13)
                .style(iced::theme::TextInput::Custom(Box::new(style::SearchInputStyle))),
            button(text("搜索").size(12))
                .on_press(Message::SearchSubmitted)
                .style(iced::theme::Button::Custom(Box::new(style::PrimaryButton)))
                .padding(8),
        ]
        .spacing(6)
        .align_items(Alignment::Center);

        // ── Search results ──
        let search_results: Element<Message> = if !self.state.search_results.is_empty() {
            let list: Vec<Element<Message>> = self.state.search_results.iter().map(|s| {
                let label = format!("{}.{}  {}", s.exchange.prefix(), s.code, s.name);
                button(text(label).size(13))
                    .on_press(Message::SearchResultSelected(s.clone()))
                    .style(iced::theme::Button::Custom(Box::new(style::SearchResultButton)))
                    .width(Length::Fill)
                    .padding(6)
                    .into()
            }).collect();
            column(list).spacing(2).into()
        } else {
            text("").into()
        };

        // ── Current stock indicator ──
        let stock_indicator: Element<Message> = match &self.state.stock_name {
            Some(name) => text(format!("当前: {}", name))
                .size(12)
                .style(style::palette::TEXT_ACCENT)
                .into(),
            None => text("").into(),
        };

        // ── Navigation buttons ──
        let panel_buttons = Column::new()
            .push(nav_button("📊", "自选股", Panel::Watchlist))
            .push(nav_button("📈", "行情走势", Panel::Chart))
            .push(nav_button("📋", "基本面分析", Panel::Fundamental))
            .push(nav_button("📐", "技术分析", Panel::Technical))
            .push(nav_button("⚙", "设置", Panel::Settings))
            .spacing(4);

        container(
            Column::new()
                .push(text("Stock Vision").size(20).style(iced::Color::from_rgb(0.9, 0.5, 0.1)))
                .push(text("").size(4))
                .push(search_row)
                .push(search_results)
                .push(stock_indicator)
                .push(text("").size(8))
                .push(panel_buttons)
                .spacing(6)
                .padding(16),
        )
        .width(220)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(style::SidebarStyle)))
        .into()
    }

    fn view_main_content(&self) -> Element<Message> {
        container(match self.state.active_panel {
            Panel::Watchlist => panels::watchlist::view(&self.state),
            Panel::Chart => panels::chart::view(&self.state),
            Panel::Fundamental => panels::fundamental::view(&self.state),
            Panel::Technical => panels::technical::view(&self.state),
            Panel::Settings => panels::settings::view(&self.state),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(style::PanelStyle)))
        .into()
    }
}

/// Navigation button with emoji icon + label
fn nav_button(icon: &'static str, label: &'static str, panel: Panel) -> iced::widget::Button<'static, Message> {
    let content = row![
        text(icon).font(EMOJI_FONT).size(15),
        text(label).size(15),
    ]
    .spacing(8)
    .align_items(Alignment::Center);
    button(content)
        .on_press(Message::PanelChanged(panel))
        .style(iced::theme::Button::Custom(Box::new(style::NavButton)))
        .width(Length::Fill)
        .padding(8)
}
