use iced::widget::{button, column, container, row, text, text_input, Column};
use iced::{alignment, Element, Fill, Font, Subscription, Task};

use std::sync::Arc;
use chrono::{DateTime, Utc};

pub const EMOJI_FONT: Font = Font::with_name("Segoe UI Emoji");

use stock_vision_data_model::*;
use stock_vision_analysis_core::FinancialAnalyzer;

use crate::services::data_service::DataService;
use crate::state::{AppState, KlinePeriod, Panel, TimeRange};
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
    // Chart controls
    SetKlinePeriod(KlinePeriod),
    SetTimeRange(TimeRange),
    ZoomIn,
    ZoomOut,
    HoverBar(Option<usize>),
    PanBy(f32),
    AddDrawingLine(f64),
    ClearDrawingLines,
}

pub struct StockVision {
    state: AppState,
    data_service: Arc<DataService>,
    analyzer: FinancialAnalyzer,
}

impl StockVision {
    pub fn new() -> (Self, Task<Message>) {
        let state = AppState::new();
        let ds = Arc::new(DataService::new(state.storage.clone()));
        (
            Self {
                state,
                data_service: ds,
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
            // ── Clock ──
            Message::Tick(now) => { self.state.current_time = now; Task::none() }

            // ── Search ──
            Message::SearchInputChanged(k) => {
                self.state.search_keyword = k;
                self.state.search_results.clear();
                Task::none()
            }
            Message::SearchSubmitted => {
                let raw = self.state.search_keyword.clone();
                let kw = raw.trim().to_string();
                if kw.is_empty() { return Task::none(); }
                let clean = kw.trim_start_matches(|c: char| c.is_alphabetic()).to_string();
                let term = if clean.is_empty() { kw } else { clean };
                let ds = self.data_service.clone();
                self.state.search_results.clear();
                Task::perform(
                    async move { ds.search_stocks(&term).await.unwrap_or_default() },
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
                self.state.kline_period = KlinePeriod::Daily;
                self.state.time_range = TimeRange::OneYear;
                self.state.daily_bars.clear();
                let code = stock.code.clone();
                let exchange = stock.exchange.clone();
                let ds = self.data_service.clone();
                Task::perform(
                    async move { ds.load_daily_bars(&code, exchange).await.unwrap_or_default() },
                    Message::DailyBarsLoaded,
                )
            }

            // ── Chart Controls ──
            Message::SetKlinePeriod(period) => {
                self.state.kline_period = period;
                if let (Some(code), Some(ex)) = (&self.state.selected_stock, &self.state.stock_exchange) {
                    self.state.daily_bars.clear();
                    let code = code.clone();
                    let exchange = ex.clone();
                    let ds = self.data_service.clone();
                    return Task::perform(
                        async move { ds.load_daily_bars(&code, exchange).await.unwrap_or_default() },
                        Message::DailyBarsLoaded,
                    );
                }
                Task::none()
            }
            Message::SetTimeRange(range) => {
                self.state.time_range = range;
                // Don't re-fetch data — time range is a client-side filter
                Task::none()
            }
            Message::ZoomIn => {
                let max_visible = self.state.daily_bars.len().min(120);
                let new_count = (self.state.zoom_level as f32 * 1.3) as usize;
                self.state.zoom_level = new_count.max(10).min(max_visible);
                Task::none()
            }
            Message::ZoomOut => {
                let max_visible = self.state.daily_bars.len().min(2000);
                let new_count = (self.state.zoom_level as f32 / 1.3) as usize;
                self.state.zoom_level = new_count.max(10).min(max_visible);
                Task::none()
            }

            // ── Data ──
            Message::DailyBarsLoaded(bars) => {
                self.state.daily_bars = bars;
                self.state.zoom_level = self.state.daily_bars.len().min(60).max(10);
                Task::none()
            }
            Message::FinancialDataLoaded(reports) => {
                self.state.financial_reports = reports;
                if let Some(r) = self.state.financial_reports.first() {
                    self.state.financial_health = Some(self.analyzer.score(r, &ValuationRatios {
                        code: String::new(), date: Utc::now().date_naive(),
                        pe: Some(15.0), pb: Some(2.0), ps: None, pcf: None, market_cap: None, dividend_yield: None,
                    }));
                }
                Task::none()
            }
            Message::AddToWatchlist => { self.state.add_to_watchlist(); Task::none() }
            Message::RemoveFromWatchlist(c) => { self.state.remove_from_watchlist(&c); Task::none() }
            Message::PanelChanged(p) => { self.state.active_panel = p; Task::none() }
            Message::HoverBar(idx) => { self.state.hovered_bar_index = idx; Task::none() }
            Message::AddDrawingLine(price) => { self.state.drawing_lines.push(crate::state::DrawingLine { price, color: (0.8, 0.8, 0.3) }); Task::none() }
            Message::ClearDrawingLines => { self.state.drawing_lines.clear(); Task::none() }
            Message::PanBy(dx) => { 
                let new_off = (self.state.pan_offset as f32 - dx).max(0.0) as usize;
                self.state.pan_offset = new_off.min(self.state.daily_bars.len().saturating_sub(self.state.zoom_level));
                Task::none() 
            }
            Message::Error(_) => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let main = self.view_main_content();
        container(row(vec![sidebar, main])).width(Fill).height(Fill).into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let sr = row![
            text_input("输入代码(如000001)或名称", &self.state.search_keyword)
                .on_input(Message::SearchInputChanged)
                .on_submit(Message::SearchSubmitted)
                .padding(8).size(13.0),
            button(text("搜索").size(12.0)).on_press(Message::SearchSubmitted).padding(8),
        ].spacing(6).align_y(alignment::Vertical::Center);

        let search_res: Element<Message> = if !self.state.search_results.is_empty() {
            let list: Vec<Element<Message>> = self.state.search_results.iter().map(|s| {
                let lbl = format!("{}.{}  {}", s.exchange.prefix(), s.code, s.name);
                button(text(lbl).size(13.0))
                    .on_press(Message::SearchResultSelected(s.clone()))
                    .width(Fill).padding(6).into()
            }).collect();
            column(list).spacing(2).into()
        } else { text("").into() };

        let indicator: Element<Message> = match &self.state.stock_name {
            Some(name) => {
                let in_wl = self.state.selected_stock.as_ref().map_or(false, |c| self.state.watchlist.iter().any(|s| &s.code == c));
                row![
                    text(format!("当前: {}", name)).size(12.0),
                    if in_wl { text(" ★").size(12.0) } else { text("") },
                ].spacing(4).into()
            }
            None => text("").into(),
        };

        let add_btn: Element<Message> = {
            let has = self.state.selected_stock.is_some();
            let already = self.state.selected_stock.as_ref().map_or(false, |c| self.state.watchlist.iter().any(|s| &s.code == c));
            if has && !already {
                button(text("+ 加入自选").size(12.0)).on_press(Message::AddToWatchlist).padding(6).width(Fill).into()
            } else { text("").into() }
        };

        let nav = Column::new()
            .push(nav_btn("📊", "自选股", Panel::Watchlist))
            .push(nav_btn("📈", "行情走势", Panel::Chart))
            .push(nav_btn("📋", "基本面分析", Panel::Fundamental))
            .push(nav_btn("📐", "技术分析", Panel::Technical))
            .push(nav_btn("⚙", "设置", Panel::Settings))
            .spacing(4);

        container(
            Column::new()
                .push(text("Stock Vision").size(20.0))
                .push(text("").size(4.0))
                .push(sr).push(search_res).push(indicator).push(add_btn)
                .push(text("").size(8.0)).push(nav)
                .spacing(6).padding(16),
        ).width(220).height(Fill).style(style::sidebar()).into()
    }

    fn view_main_content(&self) -> Element<'_, Message> {
        let content = match self.state.active_panel {
            Panel::Watchlist => panels::watchlist::view(&self.state),
            Panel::Chart => panels::chart::view(&self.state),
            Panel::Fundamental => panels::fundamental::view(&self.state),
            Panel::Technical => panels::technical::view(&self.state),
            Panel::Settings => panels::settings::view(&self.state),
        };
        let ts = self.state.current_time.format("%Y-%m-%d %H:%M:%S").to_string();
        let clock = row![text("").width(Fill), text(ts).size(13.0)].padding(8);
        container(column![clock, content].width(Fill).height(Fill))
            .width(Fill).height(Fill).style(style::panel()).into()
    }
}

fn nav_btn<'a>(icon: &'static str, label: &'static str, panel: Panel) -> iced::widget::Button<'a, Message> {
    let c = row![
        text(icon).font(EMOJI_FONT).size(15.0),
        text(label).size(15.0),
    ].spacing(8).align_y(alignment::Vertical::Center);
    button(c).on_press(Message::PanelChanged(panel)).width(Fill).padding(8)
}
