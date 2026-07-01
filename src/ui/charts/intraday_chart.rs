/// Independent intraday/minute-level chart renderer.
/// Line chart (price curve) at top, volume bars at bottom.

use iced::widget::canvas::{self, Action, Frame, Geometry, Path, Program};
use iced::{Color, Element, Fill, Point, Rectangle, Size, Theme};
use stock_vision_data_model::IntradayBar;

const CHART_START_X: f32 = 50.0;
const TOP_PAD: f32 = 15.0;
const BOTTOM_PAD: f32 = 25.0;
const VOLUME_RATIO: f32 = 0.20;

const RISE_COLOR: Color = Color::from_rgb(0.90, 0.24, 0.24);
const FALL_COLOR: Color = Color::from_rgb(0.15, 0.65, 0.24);
const LINE_COLOR: Color = Color::from_rgb(0.35, 0.60, 0.95);

pub struct IntradayCanvas {
    bars: Vec<IntradayBar>,
    hovered_index: Option<usize>,
}

impl IntradayCanvas {
    pub fn new(bars: Vec<IntradayBar>, hovered_index: Option<usize>) -> Self {
        Self { bars, hovered_index }
    }

    pub fn into_element(self) -> Element<'static, crate::app::Message> {
        canvas::Canvas::new(self).width(Fill).height(Fill).into()
    }
}

impl Program<crate::app::Message> for IntradayCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;

        if self.bars.is_empty() || w <= 0.0 || h <= 0.0 {
            return vec![frame.into_geometry()];
        }

        let kline_h = h * (1.0 - VOLUME_RATIO as f32);
        let vol_top = kline_h;
        let plot_left = CHART_START_X;
        let plot_right = w - 10.0;
        let plot_width = plot_right - plot_left;
        let n = self.bars.len();
        let step_x = if n > 1 { plot_width / (n - 1) as f32 } else { plot_width };

        let mut min_price = f64::MAX;
        let mut max_price = f64::MIN;
        let mut max_volume: f64 = 0.0;
        for bar in &self.bars {
            min_price = min_price.min(bar.low);
            max_price = max_price.max(bar.high);
            max_volume = max_volume.max(bar.volume as f64);
        }
        let price_range = (max_price - min_price).max(0.01);
        if max_volume <= 0.0 { max_volume = 1.0; }

        let to_x = |i: usize| -> f32 { plot_left + i as f32 * step_x };
        let to_price_y = |v: f64| -> f32 {
            TOP_PAD + (kline_h - TOP_PAD - BOTTOM_PAD) * (1.0 - ((v - min_price) / price_range) as f32)
        };
        let to_vol_y = |v: f64| -> f32 {
            vol_top + 15.0 + (h - vol_top - 5.0) * (1.0 - (v / max_volume) as f32)
        };

        let grid_color = Color::from_rgba(0.18, 0.18, 0.22, 0.5);
        let txt_color = Color::from_rgb(0.55, 0.55, 0.63);
        let bg_color = Color::from_rgb(0.10, 0.10, 0.14);

        frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(w, h), bg_color);

        // Grid + Y labels
        for i in 0..5 {
            let ratio = i as f64 / 4.0;
            let y = to_price_y(min_price + price_range * ratio);
            frame.fill_rectangle(Point::new(plot_left, y), Size::new(plot_width, 0.5), grid_color);
            frame.fill_text(canvas::Text {
                content: format!("{:.2}", min_price + price_range * ratio),
                position: Point::new(2.0, y - 5.0),
                color: txt_color,
                size: iced::Pixels(9.0),
                ..Default::default()
            });
        }

        // Price line
        if n > 1 {
            for i in 0..n - 1 {
                let seg = Path::line(
                    Point::new(to_x(i), to_price_y(self.bars[i].close)),
                    Point::new(to_x(i + 1), to_price_y(self.bars[i + 1].close)),
                );
                frame.stroke(&seg, canvas::Stroke::default().with_color(LINE_COLOR).with_width(1.5));
            }
            let fill_path = Path::new(|b| {
                b.move_to(Point::new(to_x(0), to_price_y(self.bars[0].close)));
                for i in 0..n {
                    b.line_to(Point::new(to_x(i), to_price_y(self.bars[i].close)));
                }
                b.line_to(Point::new(to_x(n - 1), to_price_y(self.bars[n - 1].close)));
                b.line_to(Point::new(to_x(0), to_price_y(self.bars[0].close)));
                b.close();
            });
            frame.fill(&fill_path, Color::from_rgba(LINE_COLOR.r, LINE_COLOR.g, LINE_COLOR.b, 0.08));
        }

        // Reference line
        let prev_close_y = to_price_y(self.bars[0].open);
        let dash_len = 3.0_f32;
        let dash_gap = 2.0_f32;
        let mut x = plot_left;
        while x < plot_right {
            let end = (x + dash_len).min(plot_right);
            let seg = Path::line(Point::new(x, prev_close_y), Point::new(end, prev_close_y));
            frame.stroke(&seg, canvas::Stroke::default().with_color(Color::from_rgba(0.55, 0.55, 0.63, 0.4)).with_width(0.5));
            x += dash_len + dash_gap;
        }

        // Volume bars
        for (i, bar) in self.bars.iter().enumerate() {
            let xx = to_x(i);
            let bar_w = (step_x * 0.6).max(1.0);
            let y1 = to_vol_y(bar.volume);
            let y2 = h - 5.0;
            let color = if bar.close >= bar.open { RISE_COLOR } else { FALL_COLOR };
            frame.fill_rectangle(Point::new(xx - bar_w / 2.0, y1), Size::new(bar_w, y2 - y1), Color::from_rgba(color.r, color.g, color.b, 0.6));
        }

        frame.fill_text(canvas::Text {
            content: format!("{:.0}万", max_volume / 10000.0),
            position: Point::new(2.0, vol_top + 15.0),
            color: txt_color,
            size: iced::Pixels(9.0),
            ..Default::default()
        });

        // Legend at top
        let legend_color = Color::from_rgb(0.7, 0.7, 0.7);
        frame.fill_text(canvas::Text {
            content: format!("分时  开:{:.2} 高:{:.2} 低:{:.2}", self.bars[0].open, max_price, min_price),
            position: Point::new(plot_left + 5.0, TOP_PAD + 2.0),
            color: legend_color,
            size: iced::Pixels(10.0),
            ..Default::default()
        });
        frame.fill_text(canvas::Text {
            content: "均价".to_string(),
            position: Point::new(plot_left + 5.0, TOP_PAD + 14.0),
            color: Color::from_rgba(1.0, 0.8, 0.0, 0.7),
            size: iced::Pixels(9.0),
            ..Default::default()
        });

        // Time labels
        if n > 0 {
            let fmt_time = |dt: &str| -> String {
                if let Some(t) = dt.split(' ').nth(1) { t.to_string() } else { dt.to_string() }
            };
            frame.fill_text(canvas::Text {
                content: fmt_time(&self.bars[0].datetime),
                position: Point::new(plot_left, h - 4.0),
                color: txt_color,
                size: iced::Pixels(9.0),
                ..Default::default()
            });
            let mid = n / 2;
            frame.fill_text(canvas::Text {
                content: fmt_time(&self.bars[mid].datetime),
                position: Point::new(to_x(mid) - 10.0, h - 4.0),
                color: txt_color,
                size: iced::Pixels(9.0),
                ..Default::default()
            });
            if n > 1 {
                frame.fill_text(canvas::Text {
                    content: fmt_time(&self.bars[n - 1].datetime),
                    position: Point::new(to_x(n - 1) - 15.0, h - 4.0),
                    color: txt_color,
                    size: iced::Pixels(9.0),
                    ..Default::default()
                });
            }
        }

        // Crosshair
        if let Some(idx) = self.hovered_index {
            if idx < n {
                let bar = &self.bars[idx];
                let xx = to_x(idx);
                let cross_color = Color::from_rgba(0.7, 0.7, 0.7, 0.6);

                let vline = Path::line(Point::new(xx, to_price_y(max_price)), Point::new(xx, to_price_y(min_price)));
                frame.stroke(&vline, canvas::Stroke::default().with_color(cross_color).with_width(0.5));

                let y = to_price_y(bar.close);
                let hline = Path::line(Point::new(plot_left, y), Point::new(plot_right, y));
                frame.stroke(&hline, canvas::Stroke::default().with_color(cross_color).with_width(0.5));

                let tooltip_text = format!(
                    "{}  O:{:.2} H:{:.2} L:{:.2} C:{:.2} 量:{:.0}万",
                    &bar.datetime[11..16],
                    bar.open, bar.high, bar.low, bar.close,
                    bar.volume / 10000.0,
                );
                let text_w = tooltip_text.len() as f32 * 6.5;
                let box_x = (xx - text_w / 2.0).max(5.0).min(w - text_w - 5.0);
                frame.fill_rectangle(Point::new(box_x, TOP_PAD + 2.0), Size::new(text_w + 6.0, 18.0), Color::from_rgb(0.15, 0.15, 0.20));
                frame.fill_text(canvas::Text {
                    content: tooltip_text,
                    position: Point::new(box_x + 3.0, TOP_PAD + 4.0),
                    color: Color::WHITE,
                    size: iced::Pixels(11.0),
                    ..Default::default()
                });
                let dot_color = if bar.close >= bar.open { RISE_COLOR } else { FALL_COLOR };
                frame.fill(&Path::circle(Point::new(xx, y), 3.0), dot_color);
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
    ) -> Option<Action<crate::app::Message>> {
        match event {
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                if self.bars.is_empty() {
                    return Some(Action::publish(crate::app::Message::HoverBar(None)));
                }
                let n = self.bars.len();
                let plot_left = CHART_START_X;
                let plot_right = bounds.width - 10.0;
                let plot_width = plot_right - plot_left;
                let step_x = if n > 1 { plot_width / (n - 1) as f32 } else { plot_width };
                if step_x <= 0.0 {
                    return Some(Action::publish(crate::app::Message::HoverBar(None)));
                }
                let rel_x = position.x - (bounds.x + plot_left);
                if rel_x < -20.0 || rel_x > plot_width + 20.0 {
                    return Some(Action::publish(crate::app::Message::HoverBar(None)));
                }
                let idx = ((rel_x / step_x).round() as usize).min(n - 1);
                Some(Action::publish(crate::app::Message::HoverBar(Some(idx))))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorLeft) => {
                Some(Action::publish(crate::app::Message::HoverBar(None)))
            }
            _ => None,
        }
    }
}
