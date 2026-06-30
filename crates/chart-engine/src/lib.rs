/// Chart rendering engine using plotters.
/// Handles K-line, line, and bar chart rendering.


pub struct ChartConfig {
    pub width: u32,
    pub height: u32,
    pub show_grid: bool,
    pub theme: ChartTheme,
}

pub struct ChartTheme {
    pub up_color: plotters::style::RGBColor,
    pub down_color: plotters::style::RGBColor,
    pub background: plotters::style::RGBColor,
}

impl Default for ChartTheme {
    fn default() -> Self {
        Self {
            up_color: plotters::style::RGBColor(230, 60, 60),   // Chinese red for up
            down_color: plotters::style::RGBColor(38, 165, 60), // Green for down
            background: plotters::style::RGBColor(18, 18, 24),  // Dark background
        }
    }
}

/// K-line chart renderer
pub mod candlestick;
pub use candlestick::CandlestickChart;
