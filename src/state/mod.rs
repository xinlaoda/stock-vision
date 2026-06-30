/// Application state management

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

    // Data
    pub daily_bars: Vec<DailyBar>,
    pub financial_reports: Vec<FinancialReport>,
    pub valuation: Option<ValuationRatios>,
    pub financial_health: Option<FinancialHealth>,

    // UI
    pub active_panel: Panel,
    pub watchlists: Vec<Watchlist>,

    // Storage
    pub storage: Arc<RwLock<Storage>>,
}

impl AppState {
    pub fn new() -> Self {
        let storage_path = dirs_data_dir().map(|p| {
            std::fs::create_dir_all(&p).ok();
            p.join("stock-vision.db")
        });

        let storage = match storage_path {
            Some(path) => Storage::new(path.to_str().unwrap_or("stock-vision.db"))
                .unwrap_or_else(|_| Storage::in_memory().unwrap()),
            None => Storage::in_memory().unwrap(),
        };

        Self {
            search_keyword: String::new(),
            search_results: Vec::new(),
            selected_stock: None,
            stock_name: None,
            daily_bars: Vec::new(),
            financial_reports: Vec::new(),
            valuation: None,
            financial_health: None,
            active_panel: Panel::default(),
            watchlists: Vec::new(),
            storage: Arc::new(RwLock::new(storage)),
        }
    }
}

fn dirs_data_dir() -> Option<std::path::PathBuf> {
    directories_next::ProjectDirs::from("com", "stock-vision", "StockVision")
        .map(|d| d.data_dir().to_path_buf())
}
