/// Stock Vision - A cross-platform desktop stock analysis application
///
/// Phase 1: A-share market, fundamental analysis, Win/Mac/Linux
/// Phase 2: Technical analysis indicators and charts
/// Phase 3: Quantitative backtesting engine
///
/// Built with Rust + Iced (GUI) + Plotters (charts)

use iced::{Sandbox, Settings, Size};

mod app;
mod services;
mod state;
mod ui;

use app::StockVisionApp;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    StockVisionApp::run(Settings {
        window: iced::window::Settings {
            size: Size::new(1280.0, 800.0),
            min_size: Some(Size::new(900.0, 600.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}
