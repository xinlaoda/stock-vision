use iced::widget::{button, column, container, row, text, text_input, Column};
use iced::{alignment, Alignment, Color, Element, Fill, Font, Subscription, Task};

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
    SearchInputChanged(String),
    SearchSubmitted,
    SearchResultSelected(Stock),
    SearchResultsLoaded(Vec<Stock>),
    PanelChanged(Panel),
    DailyBarsLoaded(Vec<DailyBar>),
    FinancialDataLoaded(Vec<FinancialReport>),
    Error(String),
    AddToWatchlist,
    RemoveFromWatchlist(String),
    Tick(DateTime<Utc>),
}

pub struct StockVision {
    state: AppState,
    search_source: Arc<EastMoneySource>,
    kline_source: Arc<TencentSource>,
    fin_source: Arc<EastMoneySource>,
    analyzer: FinancialAnalyzer,
}

impl StockVision {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: AppState::new(),
                search_source: Arc::new(EastMoneySource::new()),
                kline_source: Arc::new(TencentSource::new()),
                fin_source: Arc::new(EastMoneySource::new()),
                analyzer: FinancialAnalyzer,
            },
            Task::none(),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_secs(1))
            .map(|_| Message::Tick(Utc::now()))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => { self.state.current_time = now; Task::none() }

            Message::SearchInputChanged(keyword) => {
                self.state.search_keyword = keyword;
                self.state.search_results.clear();
                Task::none()
            }
            Message::SearchSubmitted => {
                let raw = self.state.search_keyword.clone();
                let keyword = raw.trim().to_string();
                if keyword.is_empty() { return Task::none(); }
                let cleaned = keyword.trim_start_matches(|c: char| c.is_alphabetic()).to_string();
                let search_term = if cleaned.is_empty() { keyword } else { cleaned };
                let source = self.search_source.clone();
                self.state.search_results.clear();
                Task::perform(
                    async move { source.search_stocks(&search_term).await.unwrap_or_default() },
                    Message::SearchResultsLoaded,
                )
            }
            Message::SearchResultsLoaded(stocks) => {
                self.state.search_results = stocks;
                Task::none()
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
                Task::perform(
                    async move {
                        source.get_daily_bars(&code, exchange, None, None, None)
                            .await.unwrap_or_default()
                    },
                    Message::DailyBarsLoaded,
                )
            }
            Message::PanelChanged(panel) => {
                let is_fundamental = panel == Panel::Fundamental;
                let needs_load = is_fundamental && self.state.selected_stock.is_some() && self.state.financial_reports.is_empty();
                self.state.active_panel = panel;
                if needs_load {
                    let code = self.state.selected_stock.clone().unwrap();
                    let source = self.fin_source.clone();
                    Task::perform(
                        async move {
                            source.get_financial_reports(&code, Exchange::SZ, None)
                                .await.unwrap_or_default()
                        },
                        Message::FinancialDataLoaded,
                    )
                } else { Task::none() }
            }
            Message::DailyBarsLoaded(bars) => { self.state.daily_bars = bars; Task::none() }
            Message::FinancialDataLoaded(reports) => {
                self.state.financial_reports = reports;
                if let Some(report) = self.state.financial_reports.first() {
                    self.state.financial_health = Some(self.analyzer.score(report, &ValuationRatios {
                        code: String::new(), date: Utc::now().date_naive(),
                        pe: Some(15.0), pb: Some(2.0), ps: None, pcf: None, market_cap: None, dividend_yield: None,
                    }));
                }
                Task::none()
            }
            Message::AddToWatchlist => { self.state.add_to_watchlist(); Task::none() }
            Message::RemoveFromWatchlist(code) => { self.state.remove_from_watchlist(&code); Task::none() }
            Message::Error(_) => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let sidebar = self.view_sidebar();
        let main_content = self.view_main_content();
        container(row(vec![sidebar, main_content]))
            .width(Fill).height(Fill)
            .into()
    }

    fn view_sidebar(&self) -> Element<Message> {
        let search_row = row![
            text_input("输入代码(如000001)或名称", &self.state.search_keyword)
                .on_input(Message::SearchInputChanged)
                .on_submit(Message::SearchSubmitted)
                .padding(8).size(13.0),
            button(text("搜索").size(12.0))
                .on_press(Message::SearchSubmitted)
                .padding(8),
        ].spacing(6).align_y(alignment::Vertical::Center);

        let search_results: Element<Message> = if !self.state.search_results.is_empty() {
            let list: Vec<Element<Message>> = self.state.search_results.iter().map(|s| {
                let label = format!("{}.{}  {}", s.exchange.prefix(), s.code, s.name);
                button(text(label).size(13.0))
                    .on_press(Message::SearchResultSelected(s.clone()))
                    .width(Fill).padding(6)
                    .into()
            }).collect();
            column(list).spacing(2).into()
        } else { text("").into() };

        let stock_indicator: Element<Message> = match &self.state.stock_name {
            Some(name) => {
                let in_wl = self.state.selected_stock.as_ref().map_or(false, |c| self.state.watchlist.iter().any(|s| &s.code == c));
                row![
                    text(format!("当前: {}", name)).size(12.0),
                    if in_wl { text(" ★").size(12.0) } else { text("") },
                ].spacing(4).into()
            }
            None => text("").into(),
        };

        let add_wl_btn: Element<Message> = {
            let has = self.state.selected_stock.is_some();
            let already = self.state.selected_stock.as_ref().map_or(false, |c| self.state.watchlist.iter().any(|s| &s.code == c));
            if has && !already {
                button(text("+ 加入自选").size(12.0))
                    .on_press(Message::AddToWatchlist)
                    .padding(6).width(Fill).into()
            } else { text("").into() }
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
                .push(text("Stock Vision").size(20.0))
                .push(text("").size(4.0))
                .push(search_row).push(search_results)
                .push(stock_indicator).push(add_wl_btn)
                .push(text("").size(8.0)).push(panel_buttons)
                .spacing(6).padding(16),
        )
        .width(220).height(Fill)
        .style(style::sidebar())
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
        let time_str = self.state.current_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let clock_bar = row![text("").width(Fill), text(time_str).size(13.0)].padding(8);
        container(column![clock_bar, content].width(Fill).height(Fill))
            .width(Fill).height(Fill)
            .style(style::panel())
            .into()
    }
}

fn nav_button(icon: &'static str, label: &'static str, panel: Panel) -> iced::widget::Button<'static, Message> {
    let content = row![
        text(icon).font(EMOJI_FONT).size(15.0),
        text(label).size(15.0),
    ].spacing(8).align_y(alignment::Vertical::Center);
    button(content)
        .on_press(Message::PanelChanged(panel))
        .width(Fill).padding(8)
}
