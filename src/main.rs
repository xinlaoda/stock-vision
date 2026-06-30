use iced::{Application, Settings, Size};

mod app;
mod services;
mod state;
mod ui;

use app::StockVisionApp;

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
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
