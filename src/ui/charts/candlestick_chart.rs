use iced::widget::canvas::{self, Frame, Geometry, Path, Program};
use iced::{Color, Element, Fill, Point, Rectangle, Size};

use stock_vision_data_model::DailyBar;
use crate::state::TimeRange;

const CHART_START_X: f32 = 50.0;
const CHART_TOP_Y: f32 = 15.0;
const CHART_BOTTOM_PAD: f32 = 30.0;
const DASH_LEN: f32 = 4.0;
const GAP_LEN: f32 = 3.0;

struct Layout {
    start_x: f32,
    top_y: f32,
    bottom_y: f32,
    chart_height: f32,
    total_width: f32,
    spacing: f32,
    bar_width: f32,
    start_global: usize,
    min_price: f64,
    max_price: f64,
    price_range: f64,
}

pub struct CandlestickCanvas {
    bars: Vec<DailyBar>,
    time_range: TimeRange,
    scroll_offset: usize,
    visible_count: usize,
    min_bar_width: f32,
    hovered_index: Option<usize>,
}

impl CandlestickCanvas {
    pub fn new(bars: Vec<DailyBar>, time_range: TimeRange, zoom_level: usize, hovered: Option<usize>) -> Self {
        let visible = zoom_level.max(10).min(bars.len().max(10));
        Self {
            bars,
            time_range,
            scroll_offset: 0,
            visible_count: visible,
            min_bar_width: 3.0,
            hovered_index: hovered,
        }
    }

    pub fn into_element(self) -> Element<'static, crate::app::Message> {
        canvas::Canvas::new(self)
            .width(Fill).height(Fill)
            .into()
    }

    fn compute_layout(&self, width: f32, height: f32) -> Layout {
        let bars = self.get_visible_bars();
        if bars.is_empty() {
            return Layout {
                start_x: CHART_START_X, top_y: CHART_TOP_Y,
                bottom_y: 0.0, chart_height: 0.0, total_width: 0.0,
                spacing: 10.0, bar_width: 3.0, start_global: 0,
                min_price: 0.0, max_price: 1.0, price_range: 1.0,
            };
        }

        let min_price = bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let max_price = bars.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let padding = (max_price - min_price) * 0.08;
        let min_price = (min_price - padding).max(0.0);
        let max_price = max_price + padding;
        let price_range = max_price - min_price;

        let bar_count = bars.len() as f32;
        let total_width = width - 60.0;
        let spacing = if bar_count > 0.0 { total_width / bar_count } else { 10.0 };
        let bottom_y = height - CHART_BOTTOM_PAD;
        let top_y = CHART_TOP_Y;
        let chart_height = bottom_y - top_y;
        let bar_width = (spacing * 0.6).max(self.min_bar_width).min(20.0);

        let total = self.bars.len();
        let end = total.saturating_sub(self.scroll_offset);
        let start_global = end.saturating_sub(self.visible_count);

        Layout {
            start_x: CHART_START_X, top_y, bottom_y, chart_height,
            total_width, spacing, bar_width, start_global,
            min_price, max_price, price_range,
        }
    }

    fn get_visible_bars(&self) -> &[DailyBar] {
        let total = self.bars.len();
        if total == 0 { return &[]; }
        let end = total.saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(self.visible_count);
        if start >= end { return &[]; }
        &self.bars[start..end]
    }

    fn draw_dashed_line(&self, frame: &mut Frame, x1: f32, y1: f32, x2: f32, y2: f32, color: Color) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 { return; }
        let steps = (len / (DASH_LEN + GAP_LEN)).ceil() as usize;
        let ux = dx / len;
        let uy = dy / len;
        for i in 0..steps {
            let s = i as f32 * (DASH_LEN + GAP_LEN);
            let e = (s + DASH_LEN).min(len);
            let ps = Point::new(x1 + ux * s, y1 + uy * s);
            let pe = Point::new(x1 + ux * e, y1 + uy * e);
            let segment = Path::line(ps, pe);
            frame.stroke(&segment, canvas::Stroke::default().with_color(color).with_width(1.0));
        }
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
        let width = bounds.width;
        let height = bounds.height;
        let mut frame = Frame::new(renderer, Size::new(width, height));

        // Background
        frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(width, height), Color::from_rgb(0.07, 0.07, 0.11));

        let bars = self.get_visible_bars();
        if bars.is_empty() {
            return vec![frame.into_geometry()];
        }

        let lay = self.compute_layout(width, height);
        let start_x = lay.start_x;
        let top_y = lay.top_y;
        let bottom_y = lay.bottom_y;
        let chart_height = lay.chart_height;
        let total_width = lay.total_width;
        let spacing = lay.spacing;
        let bar_width = lay.bar_width;
        let min_price = lay.min_price;
        let price_range = lay.price_range;

        // Grid lines
        let grid_color = Color::from_rgb(0.16, 0.16, 0.19);
        for i in 0..5 {
            let y = top_y + chart_height * (i as f32 / 4.0);
            frame.fill_rectangle(Point::new(start_x, y), Size::new(total_width, 1.0), grid_color);
        }

        let mp = |p: f64| -> f32 {
            if price_range == 0.0 { return bottom_y; }
            bottom_y - ((p - min_price) / price_range as f64 * chart_height as f64) as f32
        };

        // Candlesticks
        for (i, bar) in bars.iter().enumerate() {
            let x = start_x + i as f32 * spacing;
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

        // Y-axis price labels
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

        // X-axis labels
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

        if let Some(first) = bars.first() {
            frame.fill_text(canvas::Text {
                content: fmt_date(first.date),
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

        // ── Crosshair ──
        if let Some(hover_idx) = self.hovered_index {
            let start_global = lay.start_global;
            if hover_idx >= start_global && hover_idx < start_global + bars.len() {
                let local_idx = hover_idx - start_global;
                let cx = start_x + local_idx as f32 * spacing + bar_width / 2.0;
                let crosshair_color = Color::from_rgba(0.8, 0.8, 0.3, 0.7);

                // Vertical dashed line
                self.draw_dashed_line(&mut frame, cx, top_y, cx, bottom_y, crosshair_color);

                // Horizontal dashed line at close price
                let bar = &bars[local_idx];
                let cy = mp(bar.close);
                self.draw_dashed_line(&mut frame, start_x, cy, start_x + total_width, cy, crosshair_color);

                // Highlight bar background
                let hx = start_x + local_idx as f32 * spacing;
                let highlight_color = Color::from_rgba(1.0, 1.0, 0.5, 0.12);
                frame.fill_rectangle(
                    Point::new(hx.max(start_x), top_y),
                    Size::new(spacing.min(total_width), chart_height),
                    highlight_color,
                );
            }
        }

        vec![frame.into_geometry()]
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
                None
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                let bars = self.get_visible_bars();
                if bars.is_empty() {
                    return Some(canvas::Action::publish(crate::app::Message::HoverBar(None)));
                }

                let total_width = bounds.width - 60.0;
                let bar_count = bars.len() as f32;
                let spacing = if bar_count > 0.0 { total_width / bar_count } else { 10.0 };
                if spacing <= 0.0 {
                    return Some(canvas::Action::publish(crate::app::Message::HoverBar(None)));
                }

                let rel_x = position.x - (bounds.x + CHART_START_X);
                if rel_x < -20.0 || rel_x > total_width + 20.0 {
                    return Some(canvas::Action::publish(crate::app::Message::HoverBar(None)));
                }
                let idx = (rel_x / spacing).round() as usize;
                if idx >= bars.len() {
                    return Some(canvas::Action::publish(crate::app::Message::HoverBar(None)));
                }

                let total = self.bars.len();
                let end = total.saturating_sub(self.scroll_offset);
                let start_global = end.saturating_sub(self.visible_count);
                let global_idx = start_global + idx;
                Some(canvas::Action::publish(crate::app::Message::HoverBar(Some(global_idx))))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorLeft) => {
                Some(canvas::Action::publish(crate::app::Message::HoverBar(None)))
            }
            _ => None,
        }
    }
}
