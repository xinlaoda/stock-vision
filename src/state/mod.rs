use chrono::{DateTime, Utc};
use stock_vision_data_model::*;
use stock_vision_storage::Storage;
use std::sync::Arc;
use tokio::sync::RwLock;

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

    // Watchlist
    pub watchlist: Vec<Stock>,

    // UI
    pub active_panel: Panel,
    pub current_time: DateTime<Utc>,

    // Storage
    pub storage: Arc<RwLock<Storage>>,
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
            watchlist: Vec::new(),
            active_panel: Panel::default(),
            current_time: Utc::now(),
            storage: Arc::new(RwLock::new(storage)),
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
