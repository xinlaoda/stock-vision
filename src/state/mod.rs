use chrono::{DateTime, Datelike, Utc};
use stock_vision_data_model::*;
use stock_vision_storage::Storage;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Watchlist,
    Chart,
    Fundamental,
    Technical,
    Settings,
}

impl Default for Panel {
    fn default() -> Self {
        Panel::Watchlist
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

/// A horizontal or trend line drawn on the chart
#[derive(Debug, Clone, Copy)]
pub struct DrawingLine {
    pub price: f64,
    pub color: (f32, f32, f32),
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
    pub time_range: TimeRange,
    pub zoom_level: usize,
    pub pan_offset: usize,

    // Drawing tools
    pub drawing_lines: Vec<DrawingLine>,

    // Watchlist
    pub watchlist: Vec<Stock>,

    // UI
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

        Self {
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
            time_range: TimeRange::OneYear,
            zoom_level: 60,
            pan_offset: 0,
            drawing_lines: Vec::new(),
            watchlist: Vec::new(),
            active_panel: Panel::default(),
            current_time: Utc::now(),
            storage: Arc::new(storage),
            hovered_bar_index: None,
        }
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
            }
        }
    }

    pub fn remove_from_watchlist(&mut self, code: &str) {
        self.watchlist.retain(|s| s.code != code);
    }
}
