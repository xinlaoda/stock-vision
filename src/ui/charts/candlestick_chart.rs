use iced::widget::canvas::{Cache, Canvas, Frame, Geometry, Program};
use iced::{Color, Element, Length, Point, Rectangle, Size};

use stock_vision_data_model::DailyBar;

pub struct CandlestickCanvas {
    bars: Vec<DailyBar>,
    cache: Cache,
    scroll_offset: usize,
    visible_count: usize,
}

impl CandlestickCanvas {
    pub fn new(bars: Vec<DailyBar>) -> Self {
        let visible = bars.len().min(60);
        Self {
            bars,
            cache: Cache::new(),
            scroll_offset: 0,
            visible_count: visible,
        }
    }

    pub fn into_element(self) -> Element<'static, crate::app::Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Program<crate::app::Message> for CandlestickCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_candlesticks(frame);
        });
        vec![geometry]
    }
}

impl CandlestickCanvas {
    fn get_visible_bars(&self) -> &[DailyBar] {
        let end = self.bars.len() - self.scroll_offset;
        let start = end.saturating_sub(self.visible_count);
        &self.bars[start..end]
    }

    fn draw_candlesticks(&self, frame: &mut Frame) {
        let width = frame.width();
        let height = frame.height();
        let bars = self.get_visible_bars();

        if bars.is_empty() {
            return;
        }

        let min_price = bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let max_price = bars.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let padding = (max_price - min_price) * 0.08;
        let min_price = (min_price - padding).max(0.0);
        let max_price = max_price + padding;
        let price_range = max_price - min_price;

        let bar_count = bars.len() as f32;
        let total_width = width - 60.0;
        let spacing = total_width / bar_count;
        let start_x = 50.0;
        let bottom_y = height - 25.0;
        let top_y = 15.0;
        let chart_height = bottom_y - top_y;

        frame.fill_rectangle(
            Point::new(0.0, 0.0),
            Size::new(width, height),
            Color::from_rgb(0.07, 0.07, 0.11),
        );

        let grid_color = Color::from_rgb(0.16, 0.16, 0.19);
        for i in 0..5 {
            let y = top_y + chart_height * (i as f32 / 4.0);
            frame.fill_rectangle(
                Point::new(start_x, y),
                Size::new(total_width, 1.0),
                grid_color,
            );
        }

        let bar_width = (spacing * 0.6).max(2.0).min(20.0);

        for (i, bar) in bars.iter().enumerate() {
            let x = start_x + i as f32 * spacing;
            let map_price = |p: f64| -> f32 {
                bottom_y - ((p - min_price) / price_range * chart_height as f64) as f32
            };

            let open_y = map_price(bar.open);
            let close_y = map_price(bar.close);
            let high_y = map_price(bar.high);
            let low_y = map_price(bar.low);

            let color = if bar.close >= bar.open {
                Color::from_rgb(0.9, 0.24, 0.24)
            } else {
                Color::from_rgb(0.15, 0.65, 0.24)
            };

            let candle_center = x + bar_width / 2.0;

            frame.fill_rectangle(
                Point::new(candle_center - 1.0, high_y),
                Size::new(2.0, (low_y - high_y).max(1.0)),
                color,
            );

            let body_top = open_y.min(close_y);
            let body_height = (open_y - close_y).abs().max(1.0);
            frame.fill_rectangle(
                Point::new(x, body_top),
                Size::new(bar_width, body_height),
                color,
            );
        }

        let font_size = 12.0;
        for i in 0..5 {
            let price = min_price + price_range * (1.0 - i as f64 / 4.0);
            let y = top_y + chart_height * (i as f32 / 4.0);
            frame.fill_text(iced::widget::canvas::Text {
                content: format!("{:.2}", price),
                position: Point::new(5.0, y - font_size / 2.0),
                color: Color::from_rgb(0.55, 0.55, 0.63),
                size: font_size.into(),
                ..iced::widget::canvas::Text::default()
            });
        }

        let labels = [
            (0, bars.first().unwrap().date),
            (bars.len() / 2, bars[bars.len() / 2].date),
            (bars.len() - 1, bars.last().unwrap().date),
        ];
        for &(idx, date) in &labels {
            let x = start_x + idx as f32 * spacing;
            frame.fill_text(iced::widget::canvas::Text {
                content: date.format("%Y-%m").to_string(),
                position: Point::new(x - 20.0, bottom_y + 5.0),
                color: Color::from_rgb(0.55, 0.55, 0.63),
                size: font_size.into(),
                ..iced::widget::canvas::Text::default()
            });
        }
    }
}
