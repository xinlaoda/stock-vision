use iced::widget::canvas::{self, Cache, Frame, Geometry, Program};
use iced::{Color, Element, Fill, Point, Rectangle, Size};

use stock_vision_data_model::DailyBar;
use crate::state::TimeRange;

pub struct CandlestickCanvas {
    bars: Vec<DailyBar>,
    cache: Cache,
    time_range: TimeRange,
    scroll_offset: usize,
    visible_count: usize,
    min_bar_width: f32,
}

impl CandlestickCanvas {
    pub fn new(bars: Vec<DailyBar>, time_range: TimeRange, zoom_level: usize) -> Self {
        let visible = zoom_level.max(10).min(bars.len().max(10));
        Self {
            bars,
            cache: Cache::new(),
            time_range,
            scroll_offset: 0,
            visible_count: visible,
            min_bar_width: 3.0,
        }
    }

    pub fn into_element(self) -> Element<'static, crate::app::Message> {
        canvas::Canvas::new(self)
            .width(Fill).height(Fill)
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
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_candlesticks(frame);
        });
        vec![geometry]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<crate::app::Message>> {
        match event {
            canvas::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                if let Some(pos) = cursor.position_over(bounds) {
                    if pos.x >= bounds.x && pos.x <= bounds.x + bounds.width
                        && pos.y >= bounds.y && pos.y <= bounds.y + bounds.height
                    {
                        let scroll_amount = match delta {
                            iced::mouse::ScrollDelta::Lines { y, .. } => *y,
                            iced::mouse::ScrollDelta::Pixels { y, .. } => *y / 20.0,
                        };

                        let msg = if scroll_amount > 0.0 {
                            crate::app::Message::ZoomIn
                        } else {
                            crate::app::Message::ZoomOut
                        };
                        return Some(canvas::Action::publish(msg));
                    }
                }
            }
            _ => {}
        }
        None
    }
}

impl CandlestickCanvas {
    fn get_visible_bars(&self) -> &[DailyBar] {
        let total = self.bars.len();
        if total == 0 { return &[]; }
        // Show the most recent bars
        let end = total.saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(self.visible_count);
        if start >= end { return &[]; }
        &self.bars[start..end]
    }

    fn draw_candlesticks(&self, frame: &mut Frame) {
        let width = frame.width();
        let height = frame.height();
        let bars = self.get_visible_bars();
        if bars.is_empty() { return; }

        let min_price = bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let max_price = bars.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let padding = (max_price - min_price) * 0.08;
        let min_price = (min_price - padding).max(0.0);
        let max_price = max_price + padding;
        let price_range = max_price - min_price;

        let bar_count = bars.len() as f32;
        let total_width = width - 60.0;
        let spacing = if bar_count > 0.0 { total_width / bar_count } else { 10.0 };
        let start_x = 50.0;
        let bottom_y = height - 30.0;
        let top_y = 15.0;
        let chart_height = bottom_y - top_y;

        frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(width, height), Color::from_rgb(0.07, 0.07, 0.11));

        let grid_color = Color::from_rgb(0.16, 0.16, 0.19);
        for i in 0..5 {
            let y = top_y + chart_height * (i as f32 / 4.0);
            frame.fill_rectangle(Point::new(start_x, y), Size::new(total_width, 1.0), grid_color);
        }

        let bar_width = (spacing * 0.6).max(self.min_bar_width).min(20.0);
        for (i, bar) in bars.iter().enumerate() {
            let x = start_x + i as f32 * spacing;
            let mp = |p: f64| -> f32 { bottom_y - ((p - min_price) / price_range * chart_height as f64) as f32 };
            let oy = mp(bar.open);
            let cy = mp(bar.close);
            let hy = mp(bar.high);
            let ly = mp(bar.low);
            let color = if bar.close >= bar.open { Color::from_rgb(0.9, 0.24, 0.24) } else { Color::from_rgb(0.15, 0.65, 0.24) };
            let cx = x + bar_width / 2.0;
            frame.fill_rectangle(Point::new(cx - 1.0, hy), Size::new(2.0, (ly - hy).max(1.0)), color);
            let bt = oy.min(cy);
            frame.fill_rectangle(Point::new(x, bt), Size::new(bar_width, (oy - cy).abs().max(1.0)), color);
        }

        let font_size = 11.0;
        let text_color = Color::from_rgb(0.55, 0.55, 0.63);
        for i in 0..5 {
            let price = min_price + price_range * (1.0 - i as f64 / 4.0);
            let y = top_y + chart_height * (i as f32 / 4.0);
            frame.fill_text(canvas::Text {
                content: format!("{:.2}", price),
                position: Point::new(5.0, y - 6.0),
                color: text_color,
                size: font_size.into(),
                ..Default::default()
            });
        }

        // X-axis labels - more detailed based on visible range
        let x_label_count = bars.len().min(10).max(4);
        let label_steps = (bars.len() / x_label_count).max(1);
        let fmt_date = |d: chrono::NaiveDate| -> String {
            match self.time_range {
                TimeRange::OneMonth | TimeRange::ThreeMonths => d.format("%m-%d").to_string(),
                _ => d.format("%Y-%m").to_string(),
            }
        };

        for i in (0..bars.len()).step_by(label_steps) {
            let bar = &bars[i];
            let x = start_x + i as f32 * spacing;
            let label = fmt_date(bar.date);
            let text_width = label.len() as f32 * 6.5;
            frame.fill_text(canvas::Text {
                content: label,
                position: Point::new((x - text_width / 2.0).max(start_x), bottom_y + 5.0),
                color: text_color,
                size: font_size.into(),
                ..Default::default()
            });
        }

        // Also show first and last date
        if let Some(first) = bars.first() {
            let first_label = fmt_date(first.date);
            frame.fill_text(canvas::Text {
                content: first_label,
                position: Point::new(start_x, bottom_y + 5.0),
                color: text_color,
                size: font_size.into(),
                ..Default::default()
            });
        }
        if let Some(last) = bars.last() {
            let last_label = fmt_date(last.date);
            let last_x = start_x + (bars.len() - 1) as f32 * spacing;
            let text_width = last_label.len() as f32 * 6.5;
            frame.fill_text(canvas::Text {
                content: last_label,
                position: Point::new((last_x - text_width).max(start_x), bottom_y + 5.0),
                color: text_color,
                size: font_size.into(),
                ..Default::default()
            });
        }
    }
}
