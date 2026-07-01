use chrono::{DateTime, Datelike, Utc};
use stock_vision_data_model::*;
use stock_vision_storage::Storage;
use crate::services::indicator_service::IndicatorType;
use crate::ui::style::ThemeMode;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Home,
    Watchlist,
    Chart,
    Fundamental,
    Technical,
    Settings,
}

impl Default for Panel {
    fn default() -> Self {
        Panel::Home
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KlinePeriod {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl KlinePeriod {
    pub fn label(&self) -> &str {
        match self {
            KlinePeriod::Daily => "日K",
            KlinePeriod::Weekly => "周K",
            KlinePeriod::Monthly => "月K",
            KlinePeriod::Yearly => "年K",
        }
    }

    /// Convert to Tencent kline type parameter
    pub fn tencent_param(&self) -> &str {
        match self {
            KlinePeriod::Daily => "day",
            KlinePeriod::Weekly => "week",
            KlinePeriod::Monthly => "month",
            KlinePeriod::Yearly => "year",
        }
    }

    pub fn max_bars(&self) -> u32 {
        match self {
            KlinePeriod::Daily => 2000,
            KlinePeriod::Weekly => 500,
            KlinePeriod::Monthly => 200,
            KlinePeriod::Yearly => 50,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeRange {
    OneMonth,
    YearToDate,
    ThreeMonths,
    SixMonths,
    OneYear,
    TwoYears,
    FiveYears,
    Max,
}

impl TimeRange {
    pub fn label(&self) -> &str {
        match self {
            TimeRange::YearToDate => "年初",
            TimeRange::OneMonth => "1月",
            TimeRange::ThreeMonths => "3月",
            TimeRange::SixMonths => "6月",
            TimeRange::OneYear => "1年",
            TimeRange::TwoYears => "2年",
            TimeRange::FiveYears => "5年",
            TimeRange::Max => "全部",
        }
    }

    pub fn days(&self) -> i64 {
        match self {
            TimeRange::YearToDate => {
                let now = chrono::Utc::now().date_naive();
                let jan1 = chrono::NaiveDate::from_ymd_opt(now.year(), 1, 1).unwrap_or(now);
                (now - jan1).num_days()
            },
            TimeRange::OneMonth => 30,
            TimeRange::ThreeMonths => 90,
            TimeRange::SixMonths => 180,
            TimeRange::OneYear => 365,
            TimeRange::TwoYears => 730,
            TimeRange::FiveYears => 1825,
            TimeRange::Max => 99999,
        }
    }
}

/// Types of drawing tools
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawingToolMode {
    /// None / idle
    None,
    /// Horizontal line at a price level
    HorizontalLine,
    /// Two-point trend line (click start, click end)
    TrendLine,
    /// Ray extending from a point
    Ray,
    /// Parallel channel (three clicks)
    ParallelChannel,
}

impl DrawingToolMode {
    pub fn label(&self) -> &str {
        match self {
            DrawingToolMode::None => "选择工具",
            DrawingToolMode::HorizontalLine => "水平线",
            DrawingToolMode::TrendLine => "趋势线",
            DrawingToolMode::Ray => "射线",
            DrawingToolMode::ParallelChannel => "平行通道",
        }
    }
}

/// A line or shape drawn on the chart
#[derive(Debug, Clone)]
pub struct DrawingLine {
    pub tool_type: DrawingToolMode,
    pub color: (f32, f32, f32),
    /// Price 1 (for horizontal line, this is the price)
    pub price1: f64,
    /// Price 2 (for trend line / ray)
    pub price2: f64,
    /// Bar index 1 (for trend line start)
    pub bar_idx1: usize,
    /// Bar index 2 (for trend line end)
    pub bar_idx2: usize,
}

/// Market index real-time data
#[derive(Debug, Clone)]
pub struct MarketIndexData {
    pub name: String,
    pub code: String,
    pub price: f64,
    pub change: f64,
    pub change_pct: f64,
    pub bars: Vec<stock_vision_data_model::DailyBar>,
}

/// Configurable indicator parameters
#[derive(Debug, Clone, Copy)]
pub struct IndicatorParams {
    // MA
    pub ma_periods: [usize; 4],   // MA5, MA10, MA20, MA60
    pub vol_ma_period: usize,
    // MACD
    pub macd_fast: usize,
    pub macd_slow: usize,
    pub macd_signal: usize,
    // BOLL
    pub boll_period: usize,
    pub boll_std: f64,
    // KDJ
    pub kdj_n: usize,
    pub kdj_m1: usize,
    pub kdj_m2: usize,
    // RSI
    pub rsi_period: usize,
}

impl Default for IndicatorParams {
    fn default() -> Self {
        Self {
            ma_periods: [5, 10, 20, 60],
            vol_ma_period: 5,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            boll_period: 20,
            boll_std: 2.0,
            kdj_n: 9,
            kdj_m1: 3,
            kdj_m2: 3,
            rsi_period: 14,
        }
    }
}

pub struct AppState {
    // Search
    pub search_keyword: String,
    pub search_results: Vec<Stock>,

    // Stock context
    pub selected_stock: Option<String>,
    pub stock_name: Option<String>,
    pub stock_exchange: Option<Exchange>,

    // Data
    pub daily_bars: Vec<DailyBar>,
    pub financial_reports: Vec<FinancialReport>,
    pub valuation: Option<ValuationRatios>,
    pub financial_health: Option<FinancialHealth>,

    // Chart settings
    pub kline_period: KlinePeriod,
    pub intraday_period: Option<IntradayPeriod>,
    pub intraday_bars: Vec<IntradayBar>,
    pub time_range: TimeRange,
    pub zoom_level: usize,
    pub pan_offset: usize,

    // Drawing tools
    pub drawing_lines: Vec<DrawingLine>,
    pub drawing_tool_mode: DrawingToolMode,
    pub pending_drawing: Option<(usize, f64)>,  // (bar_idx, price) for first click

    // Technical indicators
    pub active_indicators: Vec<IndicatorType>,
    pub indicator_params: IndicatorParams,

    // Browse history
    pub browse_history: Vec<Stock>,

    // Market index data (home page)
    pub market_indices: Vec<MarketIndexData>,

    // Watchlist
    pub watchlist: Vec<Stock>,

    // UI
    pub theme_mode: ThemeMode,
    pub active_panel: Panel,
    pub current_time: DateTime<Utc>,
    pub hovered_bar_index: Option<usize>,

    // Storage
    pub storage: Arc<Storage>,
}

impl AppState {
    pub fn new() -> Self {
        let storage_path = directories_next::ProjectDirs::from("com", "stock-vision", "StockVision")
            .map(|d| {
                let p = d.data_dir().to_path_buf();
                std::fs::create_dir_all(&p).ok();
                p.join("stock-vision.db")
            });

        let storage = match storage_path {
            Some(ref path) => Storage::new(path.to_str().unwrap_or("stock-vision.db"))
                .unwrap_or_else(|_| Storage::in_memory().unwrap()),
            None => Storage::in_memory().unwrap(),
        };

        // Load persisted watchlist
        let watchlist_from_db = storage.load_watch_stocks().unwrap_or_default();

        let mut state = Self {
            search_keyword: String::new(),
            search_results: Vec::new(),
            selected_stock: None,
            stock_name: None,
            stock_exchange: None,
            daily_bars: Vec::new(),
            financial_reports: Vec::new(),
            valuation: None,
            financial_health: None,
            kline_period: KlinePeriod::Daily,
            intraday_period: None,
            intraday_bars: Vec::new(),
            time_range: TimeRange::OneYear,
            zoom_level: 60,
            pan_offset: 0,
            drawing_lines: Vec::new(),
            drawing_tool_mode: DrawingToolMode::None,
            pending_drawing: None,
            active_indicators: vec![IndicatorType::MACD],  // MACD enabled by default
            indicator_params: IndicatorParams::default(),
            browse_history: Vec::new(),
            market_indices: Vec::new(),
            watchlist: watchlist_from_db,
            theme_mode: ThemeMode::Dark,
            active_panel: Panel::default(),
            current_time: Utc::now(),
            storage: Arc::new(storage),
            hovered_bar_index: None,
        };
        // Load persisted browse history
        let _ = state.load_browse_history_from_db();
        state
    }

    pub fn add_to_watchlist(&mut self) {
        if let (Some(code), Some(name)) = (self.selected_stock.as_ref(), self.stock_name.as_ref()) {
            let exists = self.watchlist.iter().any(|s| s.code == *code);
            if !exists {
                self.watchlist.push(Stock {
                    code: code.clone(),
                    name: name.clone(),
                    exchange: self.stock_exchange.clone().unwrap_or(Exchange::SZ),
                    sector: None,
                    industry: None,
                    list_date: None,
                    total_shares: None,
                    float_shares: None,
                });
                // Persist to database
                let stocks = self.watchlist.clone();
                let s = self.storage.clone();
                tokio::spawn(async move {
                    let _ = s.save_watch_stocks(&stocks);
                });
            }
        }
    }

    pub fn push_browse_history(&mut self, stock: Stock) {
        // Persist to database
        {
            let s = self.storage.clone();
            let st = stock.clone();
            tokio::spawn(async move {
                let _ = s.save_browse_entry(&st);
            });
        }
        // Load fresh history from DB (sorted by count desc)
        if let Ok(history) = self.storage.load_browse_history(20) {
            self.browse_history = history;
        }
    }

    pub fn load_browse_history_from_db(&mut self) {
        if let Ok(history) = self.storage.load_browse_history(20) {
            self.browse_history = history;
        }
    }

    pub fn remove_from_watchlist(&mut self, code: &str) {
        self.watchlist.retain(|s| s.code != code);
        // Persist to database
        let stocks = self.watchlist.clone();
        let s = self.storage.clone();
        tokio::spawn(async move {
            let _ = s.save_watch_stocks(&stocks);
        });
    }
}
