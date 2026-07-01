use iced::time;
use iced::widget::{button, column, container, row, text, text_input, Column};
use iced::{Alignment, Application, Command, Element, Font, Length, Subscription};

use std::sync::Arc;

use chrono::{DateTime, Utc};

pub const EMOJI_FONT: Font = Font::with_name("Segoe UI Emoji");

use stock_vision_data_model::*;
use stock_vision_data_source::{DataSource, EastMoneySource, TencentSource};
use stock_vision_analysis_core::FinancialAnalyzer;

use crate::state::{AppState, Panel};
use crate::ui::{panels, style};

#[derive(Debug, Clone)]
pub enum Message {
    // Search
    SearchInputChanged(String),
    SearchSubmitted,
    SearchResultSelected(Stock),
    SearchResultsLoaded(Vec<Stock>),

    // Navigation
    PanelChanged(Panel),

    // Data loading
    DailyBarsLoaded(Vec<DailyBar>),
    FinancialDataLoaded(Vec<FinancialReport>),
    Error(String),

    // Watchlist
    AddToWatchlist,
    RemoveFromWatchlist(String),

    // Clock
    Tick(DateTime<Utc>),
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

    fn subscription(&self) -> Subscription<Message> {
        // 每秒更新一次时钟
        time::every(std::time::Duration::from_secs(1))
            .map(|_| Message::Tick(Utc::now()))
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            // ── Clock ──
            Message::Tick(now) => {
                self.state.current_time = now;
                Command::none()
            }

            // ── Search ──
            Message::SearchInputChanged(keyword) => {
                self.state.search_keyword = keyword;
                self.state.search_results.clear();
                Command::none()
            }
            Message::SearchSubmitted => {
                let raw = self.state.search_keyword.clone();
                let keyword = raw.trim().to_string();
                if keyword.is_empty() {
                    return Command::none();
                }
                let cleaned = keyword
                    .trim_start_matches(|c: char| c.is_alphabetic())
                    .to_string();
                let search_term = if cleaned.is_empty() { keyword } else { cleaned };

                let source = self.search_source.clone();
                self.state.search_results.clear();
                Command::perform(
                    async move { source.search_stocks(&search_term).await.unwrap_or_default() },
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
                self.state.stock_exchange = Some(stock.exchange.clone());
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

            // ── Watchlist ──
            Message::AddToWatchlist => {
                self.state.add_to_watchlist();
                Command::none()
            }
            Message::RemoveFromWatchlist(code) => {
                self.state.remove_from_watchlist(&code);
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
        let search_row = row![
            text_input("输入代码(如000001)或名称", &self.state.search_keyword)
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

        let stock_indicator: Element<Message> = match &self.state.stock_name {
            Some(name) => {
                let in_watchlist = self.state.selected_stock.as_ref().map_or(false, |code| {
                    self.state.watchlist.iter().any(|s| &s.code == code)
                });
                row![
                    text(format!("当前: {}", name))
                        .size(12)
                        .style(style::palette::TEXT_ACCENT),
                    if in_watchlist {
                        text(" ★").size(12).style(style::palette::RISE)
                    } else {
                        text("").into()
                    },
                ]
                .spacing(4)
                .into()
            }
            None => text("").into(),
        };

        // Add to watchlist button
        let add_watchlist_btn: Element<Message> = {
            let has_stock = self.state.selected_stock.is_some();
            let already = self.state.selected_stock.as_ref().map_or(false, |code| {
                self.state.watchlist.iter().any(|s| &s.code == code)
            });
            if has_stock && !already {
                button(text("+ 加入自选").size(12))
                    .on_press(Message::AddToWatchlist)
                    .style(iced::theme::Button::Custom(Box::new(style::PrimaryButton)))
                    .padding(6)
                    .width(Length::Fill)
                    .into()
            } else {
                text("").into()
            }
        };

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
                .push(add_watchlist_btn)
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
        let content = match self.state.active_panel {
            Panel::Watchlist => panels::watchlist::view(&self.state),
            Panel::Chart => panels::chart::view(&self.state),
            Panel::Fundamental => panels::fundamental::view(&self.state),
            Panel::Technical => panels::technical::view(&self.state),
            Panel::Settings => panels::settings::view(&self.state),
        };

        // Wrap with clock bar
        let time_str = self.state.current_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let clock_bar = row![
            text("").width(Length::Fill),
            text(time_str).size(13).style(style::palette::TEXT_SECONDARY),
        ]
        .padding(8);

        container(
            column![clock_bar, content].width(Length::Fill).height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(style::PanelStyle)))
        .into()
    }
}

fn nav_button(
    icon: &'static str,
    label: &'static str,
    panel: Panel,
) -> iced::widget::Button<'static, Message> {
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
