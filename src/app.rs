use iced::widget::{button, column, container, row, text, text_input, Column};
use iced::{alignment, Element, Fill, Font, Subscription, Task};

use std::sync::Arc;
use chrono::{DateTime, Utc};

pub const EMOJI_FONT: Font = Font::with_name("Segoe UI Emoji");

use stock_vision_data_model::*;
use stock_vision_analysis_core::FinancialAnalyzer;

use crate::services::data_service::DataService;
use crate::state::{AppState, KlinePeriod, Panel, TimeRange};
use crate::services::indicator_service::IndicatorType;
use stock_vision_data_model::Exchange;
use stock_vision_data_model::{IntradayBar, IntradayPeriod};
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
    SetIntradayPeriod(Option<IntradayPeriod>),
    SetTimeRange(TimeRange),
    ZoomIn,
    ZoomOut,
    HoverBar(Option<usize>),
    PanBy(f32),
    AddDrawingLine(f64),
    ClearDrawingLines,
    MarketIndicesLoaded(Vec<crate::state::MarketIndexData>),
    IntradayBarsLoaded(Vec<IntradayBar>),
    ToggleIndicator(IndicatorType),
    // Indicator parameter adjustments
    SetMAPeriod(usize, usize),         // (index 0-3, new period)
    SetVolMAPeriod(usize),
    SetMACDFast(usize),
    SetMACDSlow(usize),
    SetMACDSignal(usize),
    SetBOLLPeriod(usize),
    SetBOLLStd(f64),
    SetKDJ_N(usize),
    SetKDJ_M1(usize),
    SetKDJ_M2(usize),
    SetRSIPeriod(usize),
    ReloadIndicators,
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

        // Start background load of market index data
        let task = {
            let ds2 = ds.clone();
            Task::perform(
                async move {
                    ds2.load_market_indices().await.unwrap_or_default()
                },
                Message::MarketIndicesLoaded,
            )
        };

        (
            Self {
                state,
                data_service: ds,
                analyzer: FinancialAnalyzer,
            },
            task,
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
                self.state.push_browse_history(stock.clone());
                self.state.active_panel = Panel::Chart;
                self.state.kline_period = KlinePeriod::Daily;
                self.state.intraday_period = None;
                self.state.intraday_bars.clear();
                self.state.time_range = TimeRange::OneYear;
                self.state.daily_bars.clear();
                let code = stock.code.clone();
                let exchange = stock.exchange.clone();
                let ds = self.data_service.clone();

                // Background sync: preload all data into cache
                let ds2 = self.data_service.clone();
                let code2 = code.clone();
                let ex2 = exchange.clone();
                tokio::spawn(async move {
                    let _ = ds2.load_all(&code2, ex2).await;
                });

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
            Message::SetIntradayPeriod(period) => {
                self.state.intraday_period = period;
                self.state.daily_bars.clear();
                if let Some(p) = period {
                    if let Some(ref code) = self.state.selected_stock.clone() {
                        let exchange = self.state.stock_exchange.clone().unwrap_or(Exchange::SZ);
                        let svc = self.data_service.clone();
                        let code = code.clone();
                        return Task::perform(
                            async move { svc.load_intraday_bars(&code, exchange, p).await },
                            |result| match result {
                                Ok(bars) => Message::IntradayBarsLoaded(bars),
                                Err(e) => Message::Error(e.to_string()),
                            },
                        );
                    }
                }
                Task::none()
            }
            Message::IntradayBarsLoaded(bars) => {
                self.state.intraday_bars = bars;
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
            Message::PanelChanged(p) => {
                let needs_financial = p == Panel::Fundamental;
                self.state.active_panel = p;
                // Load financial data automatically when switching to Fundamental panel
                if needs_financial {
                    if let (Some(code), Some(ex)) = (&self.state.selected_stock, &self.state.stock_exchange) {
                        let code = code.clone();
                        let exchange = ex.clone();
                        let ds = self.data_service.clone();
                        return Task::perform(
                            async move {
                                ds.load_financial_data(&code, exchange).await
                                    .unwrap_or((Vec::new(), stock_vision_data_model::ValuationRatios {
                                        code: String::new(),
                                        date: chrono::Utc::now().date_naive(),
                                        pe: None, pb: None, ps: None,
                                        pcf: None, market_cap: None, dividend_yield: None,
                                    })).0
                            },
                            Message::FinancialDataLoaded,
                        );
                    }
                }
                Task::none()
            }
            Message::HoverBar(idx) => { self.state.hovered_bar_index = idx; Task::none() }
            Message::MarketIndicesLoaded(indices) => { self.state.market_indices = indices; Task::none() }
            Message::AddDrawingLine(price) => { self.state.drawing_lines.push(crate::state::DrawingLine { price, color: (0.8, 0.8, 0.3) }); Task::none() }
            Message::ClearDrawingLines => { self.state.drawing_lines.clear(); Task::none() }
            Message::ToggleIndicator(indicator) => {
                if let Some(pos) = self.state.active_indicators.iter().position(|i| *i == indicator) {
                    self.state.active_indicators.remove(pos);
                } else {
                    self.state.active_indicators.push(indicator);
                }
                Task::none()
            }
            Message::PanBy(dx) => { 
                // Drag horizontally: positive dx = drag left = show earlier data
                // Negative dx = drag right = show later data
                let total = self.state.daily_bars.len();
                let max_offset = total.saturating_sub(self.state.zoom_level);
                let min_offset: usize = 0;
                let new_off = (self.state.pan_offset as f32 - dx).round() as i64;
                let clamped = new_off.clamp(min_offset as i64, max_offset as i64) as usize;
                self.state.pan_offset = clamped;
                Task::none() 
            }
            Message::Error(_) => Task::none(),
            // Indicator parameter updates - just store and reload
            Message::SetMAPeriod(idx, period) => {
                if idx < 4 { self.state.indicator_params.ma_periods[idx] = period.max(2); }
                self.reload_chart()
            }
            Message::SetVolMAPeriod(period) => {
                self.state.indicator_params.vol_ma_period = period.max(2);
                self.reload_chart()
            }
            Message::SetMACDFast(period) => {
                self.state.indicator_params.macd_fast = period.max(2);
                self.reload_chart()
            }
            Message::SetMACDSlow(period) => {
                self.state.indicator_params.macd_slow = period.max(5);
                self.reload_chart()
            }
            Message::SetMACDSignal(period) => {
                self.state.indicator_params.macd_signal = period.max(2);
                self.reload_chart()
            }
            Message::SetBOLLPeriod(period) => {
                self.state.indicator_params.boll_period = period.max(2);
                self.reload_chart()
            }
            Message::SetBOLLStd(std) => {
                self.state.indicator_params.boll_std = std.max(0.5).min(5.0);
                self.reload_chart()
            }
            Message::SetKDJ_N(period) => {
                self.state.indicator_params.kdj_n = period.max(2);
                self.reload_chart()
            }
            Message::SetKDJ_M1(period) => {
                self.state.indicator_params.kdj_m1 = period.max(2);
                self.reload_chart()
            }
            Message::SetKDJ_M2(period) => {
                self.state.indicator_params.kdj_m2 = period.max(2);
                self.reload_chart()
            }
            Message::SetRSIPeriod(period) => {
                self.state.indicator_params.rsi_period = period.max(2);
                self.reload_chart()
            }
            Message::ReloadIndicators => {
                self.reload_chart()
            }
        }
    }

    /// Reload the daily bars (triggers redraw with new indicator params)
    fn reload_chart(&mut self) -> Task<Message> {
        if let Some(ref code) = self.state.selected_stock.clone() {
            let exchange = self.state.stock_exchange.clone().unwrap_or(Exchange::SZ);
            let svc = self.data_service.clone();
            let code = code.clone();
            return Task::perform(
                async move { svc.load_daily_bars(&code, exchange).await },
                |result| match result {
                    Ok(bars) => Message::DailyBarsLoaded(bars),
                    Err(e) => Message::Error(e.to_string()),
                },
            );
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = self.view_sidebar();
        let main = self.view_main_content();
        container(row(vec![sidebar, main])).width(Fill).height(Fill).into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        // Compact search bar
        let sr = row![
            text_input("搜索代码/名称", &self.state.search_keyword)
                .on_input(Message::SearchInputChanged)
                .on_submit(Message::SearchSubmitted)
                .padding(6).size(13.0),
            button(text("搜索").size(12.0)).on_press(Message::SearchSubmitted).padding(6),
        ].spacing(4).align_y(alignment::Vertical::Center);

        let search_res: Element<Message> = if !self.state.search_results.is_empty() {
            let list: Vec<Element<Message>> = self.state.search_results.iter().map(|s| {
                let lbl = format!("{}.{}  {}", s.exchange.prefix(), s.code, s.name);
                button(text(lbl).size(12.0))
                    .on_press(Message::SearchResultSelected(s.clone()))
                    .width(Fill).padding(4).into()
            }).collect();
            column(list).spacing(1).into()
        } else { text("").into() };

        // Navigation buttons (compact)
        let mut nav = Column::new().spacing(2);
        nav = nav.push(nav_btn("🏠", "首页", Panel::Home));
        nav = nav.push(nav_btn("📊", "自选股", Panel::Watchlist));

        // Top 5 watchlist quick-nav (between 自选股 and 行情走势)
        let top_watch: Element<Message> = if !self.state.watchlist.is_empty() {
            let items: Vec<Element<Message>> = self.state.watchlist.iter().take(5).map(|stock| {
                let st = stock.clone();
                let lbl = format!("{}.{} {}", st.exchange.prefix(), st.code, st.name);
                button(text(lbl).size(11.0).color(style::palette::TEXT_ACCENT))
                    .on_press(Message::SearchResultSelected(st.clone()))
                    .width(Fill).padding(3)
                    .style(|_t: &iced::Theme, _s: iced::widget::button::Status| iced::widget::button::Style {
                        background: None,
                        text_color: style::palette::TEXT_ACCENT,
                        ..Default::default()
                    })
                    .into()
            }).collect();
            let mut col = column![].spacing(1).padding([2, 8]);
            for item in items { col = col.push(item); }
            col.into()
        } else { text("").into() };
        nav = nav.push(top_watch);

        nav = nav.push(nav_btn("📈", "行情走势", Panel::Chart));
        nav = nav.push(nav_btn("📋", "基本面", Panel::Fundamental));
        nav = nav.push(nav_btn("📐", "技术分析", Panel::Technical));
        nav = nav.push(nav_btn("⚙", "设置", Panel::Settings));

        // Browse history
        let history: Element<Message> = if !self.state.browse_history.is_empty() {
            let mut col = Column::new().spacing(2);
            col = col.push(text("浏览记录").size(11.0).color(style::palette::TEXT_SECONDARY));
            let items: Vec<Element<Message>> = self.state.browse_history.iter().take(15).map(|stock| {
                let lbl = format!("{}.{}", stock.exchange.prefix(), stock.code);
                let display = format!("{} ({})", stock.name, lbl);
                button(text(display).size(10.0))
                    .on_press(Message::SearchResultSelected(stock.clone()))
                    .width(Fill).padding(3)
                    .style(|_t: &iced::Theme, _s: iced::widget::button::Status| iced::widget::button::Style {
                        background: Some(style::palette::BG_MID.into()),
                        text_color: style::palette::TEXT_PRIMARY,
                        ..Default::default()
                    })
                    .into()
            }).collect();
            for item in items { col = col.push(item); }
            col.into()
        } else { text("").into() };

        container(
            Column::new()
                .push(text("Stock Vision").size(18.0))
                .push(text("").size(2.0))
                .push(sr).push(search_res)
                .push(text("").size(4.0)).push(nav)
                .push(text("").size(4.0)).push(history)
                .spacing(2).padding([6, 4])
        )
            .width(220.0)
            .height(Fill)
            .style(style::sidebar())
            .into()
    }

    fn view_main_content(&self) -> Element<'_, Message> {
        let content = match self.state.active_panel {
            Panel::Home => panels::home::view(&self.state),
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
