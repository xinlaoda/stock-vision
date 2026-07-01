use iced::widget::canvas::{self, Frame, Geometry, Path, Program};
use iced::{Color, Element, Fill, Point, Rectangle, Size};

use stock_vision_data_model::DailyBar;
use crate::state::TimeRange;

const CHART_START_X: f32 = 50.0;
const TOP_PAD: f32 = 15.0;
const BOTTOM_PAD: f32 = 30.0;
const VOLUME_RATIO: f32 = 0.25;  // bottom 25% for volume
const DASH_LEN: f32 = 4.0;
const GAP_LEN: f32 = 3.0;

struct Layout {
    start_x: f32,
    total_width: f32,
    spacing: f32,
    bar_width: f32,
    start_global: usize,
    // K-line area
    k_top: f32,
    k_bottom: f32,
    k_height: f32,
    min_price: f64,
    max_price: f64,
    price_range: f64,
    // Volume area
    v_top: f32,
    v_bottom: f32,
    v_height: f32,
    max_volume: f64,
}

fn compute_ma(bars: &[DailyBar], period: usize) -> Vec<Option<f64>> {
    if bars.is_empty() || period == 0 { return vec![]; }
    let mut result = Vec::with_capacity(bars.len());
    let mut sum = 0.0;
    for (i, bar) in bars.iter().enumerate() {
        sum += bar.close;
        if i >= period {
            sum -= bars[i - period].close;
        }
        if i >= period - 1 {
            result.push(Some(sum / period as f64));
        } else {
            result.push(None);
        }
    }
    result
}

fn compute_volume_ma(bars: &[DailyBar], period: usize) -> Vec<Option<f64>> {
    if bars.is_empty() || period == 0 { return vec![]; }
    let mut result = Vec::with_capacity(bars.len());
    let mut sum = 0.0;
    for (i, bar) in bars.iter().enumerate() {
        sum += bar.volume;
        if i >= period {
            sum -= bars[i - period].volume;
        }
        if i >= period - 1 {
            result.push(Some(sum / period as f64));
        } else {
            result.push(None);
        }
    }
    result
}

pub struct CandlestickCanvas {
    bars: Vec<DailyBar>,
    time_range: TimeRange,
    scroll_offset: usize,
    visible_count: usize,
    min_bar_width: f32,
    hovered_index: Option<usize>,
    ma5: Vec<Option<f64>>,
    ma10: Vec<Option<f64>>,
    ma20: Vec<Option<f64>>,
    ma60: Vec<Option<f64>>,
    vol_ma5: Vec<Option<f64>>,
}

impl CandlestickCanvas {
    pub fn new(bars: Vec<DailyBar>, time_range: TimeRange, zoom_level: usize, hovered: Option<usize>, pan_offset: usize) -> Self {
        let visible = zoom_level.max(10).min(bars.len().max(10));
        let ma5 = compute_ma(&bars, 5);
        let ma10 = compute_ma(&bars, 10);
        let ma20 = compute_ma(&bars, 20);
        let ma60 = compute_ma(&bars, 60);
        let vol_ma5 = compute_volume_ma(&bars, 5);
        Self {
            bars,
            time_range,
            scroll_offset: pan_offset,
            visible_count: visible,
            min_bar_width: 3.0,
            hovered_index: hovered,
            ma5, ma10, ma20, ma60,
            vol_ma5,
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
                start_x: CHART_START_X, total_width: 0.0, spacing: 10.0, bar_width: 3.0, start_global: 0,
                k_top: 0.0, k_bottom: 0.0, k_height: 0.0,
                min_price: 0.0, max_price: 1.0, price_range: 1.0,
                v_top: 0.0, v_bottom: 0.0, v_height: 0.0, max_volume: 1.0,
            };
        }

        let min_price = bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let max_price = bars.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let padding = (max_price - min_price) * 0.08;
        let min_price = (min_price - padding).max(0.0);
        let max_price = max_price + padding;
        let price_range = max_price - min_price;

        let max_volume = bars.iter().map(|b| b.volume).fold(0.0f64, f64::max);

        let bar_count = bars.len() as f32;
        let total_width = width - 60.0;
        let spacing = if bar_count > 0.0 { total_width / bar_count } else { 10.0 };
        let bar_width = (spacing * 0.6).max(self.min_bar_width).min(20.0);

        let total_height = height - TOP_PAD - BOTTOM_PAD;
        let v_height = total_height * VOLUME_RATIO;
        let k_height = total_height - v_height;
        let k_top = TOP_PAD;
        let k_bottom = k_top + k_height;
        let v_top = k_bottom;
        let v_bottom = height - BOTTOM_PAD;

        let total = self.bars.len();
        let end = total.saturating_sub(self.scroll_offset);
        let start_global = end.saturating_sub(self.visible_count);

        Layout {
            start_x: CHART_START_X, total_width, spacing, bar_width, start_global,
            k_top, k_bottom, k_height, min_price, max_price, price_range,
            v_top, v_bottom, v_height, max_volume,
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
            let segment = Path::line(
                Point::new(x1 + ux * s, y1 + uy * s),
                Point::new(x1 + ux * e, y1 + uy * e),
            );
            frame.stroke(&segment, canvas::Stroke::default().with_color(color).with_width(1.0));
        }
    }

    fn draw_ma_line(&self, frame: &mut Frame, ma: &[Option<f64>], start_global: usize, color: Color, lay: &Layout) {
        if ma.is_empty() { return; }
        let bars = self.get_visible_bars();
        let mp = |p: f64| -> f32 {
            if lay.price_range == 0.0 { return lay.k_bottom; }
            lay.k_bottom - ((p - lay.min_price) / lay.price_range as f64 * lay.k_height as f64) as f32
        };

        let mut points: Vec<(f32, f32)> = Vec::new();
        for (i, bar) in bars.iter().enumerate() {
            let global_i = start_global + i;
            if global_i >= ma.len() { break; }
            if let Some(v) = ma[global_i] {
                let x = lay.start_x + i as f32 * lay.spacing + lay.bar_width / 2.0;
                let y = mp(v);
                points.push((x, y));
            }
        }

        for win in points.windows(2) {
            let (x1, y1) = win[0];
            let (x2, y2) = win[1];
            let segment = Path::line(Point::new(x1, y1), Point::new(x2, y2));
            frame.stroke(&segment, canvas::Stroke::default().with_color(color).with_width(1.5));
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
        let sx = lay.start_x;
        let tw = lay.total_width;
        let sp = lay.spacing;
        let bw = lay.bar_width;
        let kt = lay.k_top;
        let kb = lay.k_bottom;
        let kh = lay.k_height;
        let vt = lay.v_top;
        let vb = lay.v_bottom;

        let mp = |p: f64| -> f32 {
            if lay.price_range == 0.0 { return kb; }
            kb - ((p - lay.min_price) / lay.price_range as f64 * kh as f64) as f32
        };

        let vp = |v: f64| -> f32 {
            if lay.max_volume == 0.0 { return vb; }
            vb - (v / lay.max_volume as f64 * lay.v_height as f64) as f32
        };

        let grid_color = Color::from_rgb(0.16, 0.16, 0.19);
        let text_color = Color::from_rgb(0.55, 0.55, 0.63);
        let font_size = 11.0;

        // ── K-line area grid ──
        for i in 0..5 {
            let y = kt + kh * (i as f32 / 4.0);
            frame.fill_rectangle(Point::new(sx, y), Size::new(tw, 1.0), grid_color);
        }
        // Separator between K-line and volume
        frame.fill_rectangle(Point::new(sx, vt), Size::new(tw, 1.0), Color::from_rgb(0.25, 0.25, 0.3));

        // ── Candlesticks ──
        for (i, bar) in bars.iter().enumerate() {
            let x = sx + i as f32 * sp;
            let oy = mp(bar.open);
            let cy = mp(bar.close);
            let hy = mp(bar.high);
            let ly = mp(bar.low);
            let color = if bar.close >= bar.open { Color::from_rgb(0.9, 0.24, 0.24) } else { Color::from_rgb(0.15, 0.65, 0.24) };
            let cx = x + bw / 2.0;
            frame.fill_rectangle(Point::new(cx - 1.0, hy), Size::new(2.0, (ly - hy).max(1.0)), color);
            let bt = oy.min(cy);
            frame.fill_rectangle(Point::new(x, bt), Size::new(bw, (oy - cy).abs().max(1.0)), color);
        }

        // ── MA lines ──
        let sg = lay.start_global;
        self.draw_ma_line(&mut frame, &self.ma5, sg, Color::from_rgb(1.0, 0.8, 0.0), &lay);    // yellow
        self.draw_ma_line(&mut frame, &self.ma10, sg, Color::from_rgb(0.3, 0.7, 1.0), &lay);   // blue
        self.draw_ma_line(&mut frame, &self.ma20, sg, Color::from_rgb(1.0, 0.4, 0.7), &lay);   // pink
        self.draw_ma_line(&mut frame, &self.ma60, sg, Color::from_rgb(0.2, 0.8, 0.4), &lay);   // green

        // ── Y-axis price labels ──
        for i in 0..5 {
            let price = lay.min_price + lay.price_range * (1.0 - i as f64 / 4.0);
            let y = kt + kh * (i as f32 / 4.0);
            frame.fill_text(canvas::Text {
                content: format!("{:.2}", price),
                position: Point::new(5.0, y - 6.0),
                color: text_color,
                size: font_size.into(),
                ..Default::default()
            });
        }

        // ── Volume bars ──
        for (i, bar) in bars.iter().enumerate() {
            let x = sx + i as f32 * sp;
            let vol_h = (bar.volume / lay.max_volume as f64 * lay.v_height as f64) as f32;
            let vcolor = if bar.close >= bar.open { Color::from_rgba(0.9, 0.24, 0.24, 0.6) } else { Color::from_rgba(0.15, 0.65, 0.24, 0.6) };
            let vw = (sp * 0.6).max(1.0);
            frame.fill_rectangle(Point::new(x, vb - vol_h), Size::new(vw, vol_h.max(1.0)), vcolor);
        }

        // Volume MA5 line
        let vm5_color = Color::from_rgba(1.0, 1.0, 0.6, 0.8);
        if sg < self.vol_ma5.len() {
            let mut vpoints: Vec<(f32, f32)> = Vec::new();
            for (i, bar) in bars.iter().enumerate() {
                let gi = sg + i;
                if gi >= self.vol_ma5.len() { break; }
                if let Some(v) = self.vol_ma5[gi] {
                    let x = sx + i as f32 * sp + bw / 2.0;
                    let y = vp(v);
                    vpoints.push((x, y));
                }
            }
            for win in vpoints.windows(2) {
                let (x1, y1) = win[0];
                let (x2, y2) = win[1];
                frame.stroke(&Path::line(Point::new(x1, y1), Point::new(x2, y2)),
                    canvas::Stroke::default().with_color(vm5_color).with_width(1.5));
            }
        }

        // Volume label
        if bars.len() > 1 {
            let vol_label = format!("VOL {:.0}", lay.max_volume);
            frame.fill_text(canvas::Text {
                content: vol_label,
                position: Point::new(sx + 5.0, vt + 5.0),
                color: text_color,
                size: 10.0.into(),
                ..Default::default()
            });
        }

        // ── X-axis labels ──
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
            let x = sx + i as f32 * sp;
            let label = fmt_date(bar.date);
            let text_width = label.len() as f32 * 6.5;
            frame.fill_text(canvas::Text {
                content: label,
                position: Point::new((x - text_width / 2.0).max(sx), height - BOTTOM_PAD + 5.0),
                color: text_color,
                size: font_size.into(),
                ..Default::default()
            });
        }

        if let Some(first) = bars.first() {
            frame.fill_text(canvas::Text {
                content: fmt_date(first.date),
                position: Point::new(sx, height - BOTTOM_PAD + 5.0),
                color: text_color,
                size: font_size.into(),
                ..Default::default()
            });
        }
        if let Some(last) = bars.last() {
            let last_label = fmt_date(last.date);
            let last_x = sx + (bars.len() - 1) as f32 * sp;
            let text_width = last_label.len() as f32 * 6.5;
            frame.fill_text(canvas::Text {
                content: last_label,
                position: Point::new((last_x - text_width).max(sx), height - BOTTOM_PAD + 5.0),
                color: text_color,
                size: font_size.into(),
                ..Default::default()
            });
        }

        // ── Crosshair ──
        if let Some(hover_idx) = self.hovered_index {
            let sg2 = lay.start_global;
            if hover_idx >= sg2 && hover_idx < sg2 + bars.len() {
                let local_idx = hover_idx - sg2;
                let cx = sx + local_idx as f32 * sp + bw / 2.0;
                let crosshair_color = Color::from_rgba(0.8, 0.8, 0.3, 0.7);

                // Vertical full-height dashed line
                self.draw_dashed_line(&mut frame, cx, kt, cx, vb, crosshair_color);

                // Horizontal at close price
                let bar = &bars[local_idx];
                let cy = mp(bar.close);
                self.draw_dashed_line(&mut frame, sx, cy, sx + tw, cy, crosshair_color);

                // Highlight bar
                let hx = sx + local_idx as f32 * sp;
                let highlight_color = Color::from_rgba(1.0, 1.0, 0.5, 0.12);
                frame.fill_rectangle(
                    Point::new(hx.max(sx), kt),
                    Size::new(sp.min(tw), kh),
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
