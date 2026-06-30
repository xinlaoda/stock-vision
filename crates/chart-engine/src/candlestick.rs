/// Candlestick (K-line) chart implementation.

use stock_vision_data_model::DailyBar;

use crate::{ChartConfig, ChartTheme};

pub struct CandlestickChart;

impl CandlestickChart {
    pub fn render(
        bars: &[DailyBar],
        config: &ChartConfig,
        _theme: &ChartTheme,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use plotters::prelude::*;

        let mut buffer = vec![0u8; (config.width * config.height * 3) as usize];

        {
            let root = plotters::backend::BitMapBackend::with_buffer(
                &mut buffer,
                (config.width, config.height),
            )
            .into_drawing_area();

            root.fill(&RGBColor(18, 18, 24))?;

            if !bars.is_empty() {
                let min_price = bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
                let max_price = bars.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
                let price_padding = (max_price - min_price) * 0.05;
                let min_price = min_price - price_padding;
                let max_price = max_price + price_padding;

                let start_date = bars[0].date;
                let end_date = bars[bars.len() - 1].date;

                let mut chart = ChartBuilder::on(&root)
                    .margin(20)
                    .x_label_area_size(30)
                    .y_label_area_size(40)
                    .build_cartesian_2d(
                        start_date..end_date,
                        min_price..max_price,
                    )?;

                chart.configure_mesh()
                    .light_line_style(&RGBColor(40, 40, 50))
                    .label_style(("sans-serif", 12).into_font().color(&RGBColor(160, 160, 180)))
                    .x_labels(10)
                    .y_labels(8)
                    .draw()?;

                // Draw candlesticks
                let candle_width = ((end_date - start_date).num_days() as f64 / bars.len() as f64) * 0.6;
                let candle_width = candle_width.max(1.0);

                for bar in bars {
                    let color = if bar.close >= bar.open {
                        &RGBColor(230, 60, 60)   // Up = red (Chinese convention)
                    } else {
                        &RGBColor(38, 165, 60)   // Down = green
                    };

                    chart.draw_series(std::iter::once(
                        plotters::element::CandleStick::new(
                            bar.date,
                            bar.open,
                            bar.high,
                            bar.low,
                            bar.close,
                            color,
                            color,
                            candle_width as u32,
                        ),
                    ))?;
                }
            }

            root.present()?;
        } // root dropped here, releasing borrow on buffer

        Ok(buffer)
    }
}
