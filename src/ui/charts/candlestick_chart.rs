use iced::widget::canvas::{self, Frame, Geometry, Path, Program};
use iced::{Color, Element, Fill, Point, Rectangle, Size};

use stock_vision_data_model::DailyBar;
use crate::state::TimeRange;

const CHART_START_X: f32 = 50.0;
const TOP_PAD: f32 = 15.0;
const BOTTOM_PAD: f32 = 30.0;
const DASH_LEN: f32 = 4.0;
const GAP_LEN: f32 = 3.0;

/// Layout ratios for the 3-section chart
const KLINE_RATIO: f32 = 0.55;
const VOLUME_RATIO: f32 = 0.20;
const MACD_RATIO: f32 = 0.25;

struct SectionLayout {
    top: f32,
    bottom: f32,
    height: f32,
}

struct Layout {
    start_x: f32,
    total_width: f32,
    spacing: f32,
    bar_width: f32,
    start_global: usize,
    // K-line
    kline: SectionLayout,
    min_price: f64,
    max_price: f64,
    price_range: f64,
    // Volume
    volume: SectionLayout,
    max_volume: f64,
    // MACD
    macd: SectionLayout,
    macd_max: f64,
}

fn compute_ma(bars: &[DailyBar], period: usize) -> Vec<Option<f64>> {
    if bars.is_empty() || period == 0 { return vec![]; }
    let mut result = Vec::with_capacity(bars.len());
    let mut sum = 0.0;
    for (i, bar) in bars.iter().enumerate() {
        sum += bar.close;
        if i >= period { sum -= bars[i - period].close; }
        if i >= period - 1 { result.push(Some(sum / period as f64)); }
        else { result.push(None); }
    }
    result
}

fn compute_volume_ma(bars: &[DailyBar], period: usize) -> Vec<Option<f64>> {
    if bars.is_empty() || period == 0 { return vec![]; }
    let mut result = Vec::with_capacity(bars.len());
    let mut sum = 0.0;
    for (i, bar) in bars.iter().enumerate() {
        sum += bar.volume;
        if i >= period { sum -= bars[i - period].volume; }
        if i >= period - 1 { result.push(Some(sum / period as f64)); }
        else { result.push(None); }
    }
    result
}

/// Compute MACD (12, 26, 9).
/// Returns (dif, dea, histogram).
struct MacdLine { dif: Option<f64>, dea: Option<f64>, bar: Option<f64> }

fn compute_macd(bars: &[DailyBar]) -> Vec<MacdLine> {
    let n = bars.len();
    if n == 0 { return vec![]; }

    // EMA helper
    let ema = |values: &[f64], period: usize| -> Vec<Option<f64>> {
        if values.len() < period { return vec![None; values.len()]; }
        let k = 2.0 / (period as f64 + 1.0);
        let mut result = vec![None; values.len()];
        let mut ema_val = values[0];
        for i in 0..values.len() {
            ema_val = if i == 0 { values[i] } else { values[i] * k + ema_val * (1.0 - k) };
            if i >= period - 1 { result[i] = Some(ema_val); }
        }
        result
    };

    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let ema12 = ema(&closes, 12);
    let ema26 = ema(&closes, 26);

    // DIF = EMA12 - EMA26
    let mut difs: Vec<Option<f64>> = Vec::with_capacity(n);
    for i in 0..n {
        difs.push(match (ema12[i], ema26[i]) {
            (Some(e12), Some(e26)) => Some(e12 - e26),
            _ => None,
        });
    }

    // DEA = EMA(DIF, 9)
    let dif_values: Vec<f64> = difs.iter().filter_map(|d| *d).collect();
    let ema_dif = ema(&dif_values, 9);

    let mut result = Vec::with_capacity(n);
    let mut dea_idx = 0;
    for i in 0..n {
        let dif = difs[i];
        let dea = if dif.is_some() {
            let d = ema_dif.get(dea_idx).and_then(|v| *v);
            if dif.is_some() { dea_idx += 1; }
            d
        } else { None };
        let bar = match (dif, dea) {
            (Some(d), Some(e)) => Some(d - e),
            _ => None,
        };
        result.push(MacdLine { dif, dea, bar });
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
    macd: Vec<MacdLine>,
}

impl CandlestickCanvas {
    pub fn new(bars: Vec<DailyBar>, time_range: TimeRange, zoom_level: usize, hovered: Option<usize>, pan_offset: usize) -> Self {
        let visible = zoom_level.max(10).min(bars.len().max(10));
        let ma5 = compute_ma(&bars, 5);
        let ma10 = compute_ma(&bars, 10);
        let ma20 = compute_ma(&bars, 20);
        let ma60 = compute_ma(&bars, 60);
        let vol_ma5 = compute_volume_ma(&bars, 5);
        let macd = compute_macd(&bars);
        Self {
            bars, time_range, scroll_offset: pan_offset, visible_count: visible,
            min_bar_width: 3.0, hovered_index: hovered,
            ma5, ma10, ma20, ma60, vol_ma5, macd,
        }
    }

    pub fn into_element(self) -> Element<'static, crate::app::Message> {
        canvas::Canvas::new(self).width(Fill).height(Fill).into()
    }

    fn section(&self, ratio: f32, total_h: f32, prev_bottom: f32) -> SectionLayout {
        let h = total_h * ratio;
        SectionLayout { top: prev_bottom, bottom: prev_bottom + h, height: h }
    }

    fn compute_layout(&self, width: f32, height: f32) -> Layout {
        let bars = self.get_visible_bars();
        if bars.is_empty() {
            return Layout {
                start_x: CHART_START_X, total_width: 0.0, spacing: 10.0, bar_width: 3.0, start_global: 0,
                kline: SectionLayout { top: 0.0, bottom: 0.0, height: 0.0 },
                min_price: 0.0, max_price: 1.0, price_range: 1.0,
                volume: SectionLayout { top: 0.0, bottom: 0.0, height: 0.0 }, max_volume: 1.0,
                macd: SectionLayout { top: 0.0, bottom: 0.0, height: 0.0 }, macd_max: 1.0,
            };
        }

        let min_price = bars.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let max_price = bars.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let padding = (max_price - min_price) * 0.08;
        let min_price = (min_price - padding).max(0.0);
        let max_price = max_price + padding;
        let price_range = max_price - min_price;

        let max_volume = bars.iter().map(|b| b.volume).fold(0.0f64, f64::max);
        let macd_max = self.macd.iter().filter_map(|m| m.bar.map(|v| v.abs())).fold(0.0f64, f64::max).max(0.001);

        let bar_count = bars.len() as f32;
        let total_width = width - 60.0;
        let spacing = if bar_count > 0.0 { total_width / bar_count } else { 10.0 };
        let bar_width = (spacing * 0.6).max(self.min_bar_width).min(20.0);

        let total_h = height - TOP_PAD - BOTTOM_PAD;
        let kline = self.section(KLINE_RATIO, total_h, TOP_PAD);
        let volume = self.section(VOLUME_RATIO, total_h, kline.bottom);
        let macd = self.section(MACD_RATIO, total_h, volume.bottom);

        let total = self.bars.len();
        let end = total.saturating_sub(self.scroll_offset);
        let start_global = end.saturating_sub(self.visible_count);

        Layout {
            start_x: CHART_START_X, total_width, spacing, bar_width, start_global,
            kline, min_price, max_price, price_range,
            volume, max_volume,
            macd, macd_max,
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
        let dx = x2 - x1; let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 { return; }
        let steps = (len / (DASH_LEN + GAP_LEN)).ceil() as usize;
        let ux = dx / len; let uy = dy / len;
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

    fn draw_ma_line(&self, frame: &mut Frame, ma: &[Option<f64>], start_global: usize, color: Color, sx: f32, sp: f32, bw: f32, to_px: impl Fn(f64) -> f32) {
        if ma.is_empty() { return; }
        let bars = self.get_visible_bars();
        let mut points: Vec<(f32, f32)> = Vec::new();
        for (i, bar) in bars.iter().enumerate() {
            let gi = start_global + i;
            if gi >= ma.len() { break; }
            if let Some(v) = ma[gi] {
                points.push((sx + i as f32 * sp + bw / 2.0, to_px(v)));
            }
        }
        for win in points.windows(2) {
            let (x1, y1) = win[0]; let (x2, y2) = win[1];
            frame.stroke(&Path::line(Point::new(x1, y1), Point::new(x2, y2)),
                canvas::Stroke::default().with_color(color).with_width(1.5));
        }
    }
}

impl Program<crate::app::Message> for CandlestickCanvas {
    type State = ();

    fn draw(&self, _state: &Self::State, renderer: &iced::Renderer, _theme: &iced::Theme, bounds: Rectangle, _cursor: iced::mouse::Cursor) -> Vec<Geometry> {
        let width = bounds.width; let height = bounds.height;
        let mut frame = Frame::new(renderer, Size::new(width, height));
        frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(width, height), Color::from_rgb(0.07, 0.07, 0.11));

        let bars = self.get_visible_bars();
        if bars.is_empty() { return vec![frame.into_geometry()]; }

        let lay = self.compute_layout(width, height);
        let sx = lay.start_x; let tw = lay.total_width; let sp = lay.spacing; let bw = lay.bar_width;
        let kl = &lay.kline; let vl = &lay.volume; let ml = &lay.macd;
        let sg = lay.start_global;

        let k_mp = |p: f64| -> f32 {
            if lay.price_range == 0.0 { return kl.bottom; }
            kl.bottom - ((p - lay.min_price) / lay.price_range as f64 * kl.height as f64) as f32
        };
        let v_mp = |v: f64| -> f32 {
            if lay.max_volume == 0.0 { return vl.bottom; }
            vl.bottom - (v / lay.max_volume as f64 * vl.height as f64) as f32
        };
        let m_mp = |v: f64| -> f32 {
            if lay.macd_max == 0.0 { return ml.bottom; }
            let mid = (ml.top + ml.bottom) / 2.0;
            mid - (v / lay.macd_max as f64 * ml.height as f64 * 0.45) as f32
        };

        let grid_color = Color::from_rgb(0.16, 0.16, 0.19);
        let text_color = Color::from_rgb(0.55, 0.55, 0.63);
        let font_size = 11.0;

        // ── Grid: K-line area ──
        for i in 0..5 {
            let y = kl.top + kl.height * (i as f32 / 4.0);
            frame.fill_rectangle(Point::new(sx, y), Size::new(tw, 1.0), grid_color);
        }
        // Volume grid
        for i in 0..3 {
            let y = vl.top + vl.height * (i as f32 / 2.0);
            frame.fill_rectangle(Point::new(sx, y), Size::new(tw, 1.0), grid_color);
        }
        // MACD grid
        let y_mid = (ml.top + ml.bottom) / 2.0;
        frame.fill_rectangle(Point::new(sx, y_mid), Size::new(tw, 1.0), grid_color);
        // Separators
        frame.fill_rectangle(Point::new(sx, kl.bottom), Size::new(tw, 1.0), Color::from_rgb(0.25, 0.25, 0.3));
        frame.fill_rectangle(Point::new(sx, vl.bottom), Size::new(tw, 1.0), Color::from_rgb(0.25, 0.25, 0.3));

        // ── K-line candlesticks ──
        for (i, bar) in bars.iter().enumerate() {
            let x = sx + i as f32 * sp;
            let oy = k_mp(bar.open); let cy = k_mp(bar.close);
            let hy = k_mp(bar.high); let ly = k_mp(bar.low);
            let color = if bar.close >= bar.open { Color::from_rgb(0.9, 0.24, 0.24) } else { Color::from_rgb(0.15, 0.65, 0.24) };
            let cx = x + bw / 2.0;
            frame.fill_rectangle(Point::new(cx - 1.0, hy), Size::new(2.0, (ly - hy).max(1.0)), color);
            frame.fill_rectangle(Point::new(x, oy.min(cy)), Size::new(bw, (oy - cy).abs().max(1.0)), color);
        }

        // ── MA lines ──
        let ma_to_px = |p: f64| k_mp(p);
        self.draw_ma_line(&mut frame, &self.ma5, sg, Color::from_rgb(1.0, 0.8, 0.0), sx, sp, bw, ma_to_px);
        self.draw_ma_line(&mut frame, &self.ma10, sg, Color::from_rgb(0.3, 0.7, 1.0), sx, sp, bw, ma_to_px);
        self.draw_ma_line(&mut frame, &self.ma20, sg, Color::from_rgb(1.0, 0.4, 0.7), sx, sp, bw, ma_to_px);
        self.draw_ma_line(&mut frame, &self.ma60, sg, Color::from_rgb(0.2, 0.8, 0.4), sx, sp, bw, ma_to_px);

        // ── Y-axis price labels ──
        for i in 0..5 {
            let price = lay.min_price + lay.price_range * (1.0 - i as f64 / 4.0);
            let y = kl.top + kl.height * (i as f32 / 4.0);
            frame.fill_text(canvas::Text {
                content: format!("{:.2}", price), position: Point::new(5.0, y - 6.0), color: text_color, size: iced::Pixels(font_size), ..Default::default()
            });
        }

        // ── Volume bars ──
        for (i, bar) in bars.iter().enumerate() {
            let x = sx + i as f32 * sp;
            let vol_h = (bar.volume / lay.max_volume as f64 * vl.height as f64) as f32;
            let vc = if bar.close >= bar.open { Color::from_rgba(0.9, 0.24, 0.24, 0.6) } else { Color::from_rgba(0.15, 0.65, 0.24, 0.6) };
            let vw = (sp * 0.6).max(1.0);
            frame.fill_rectangle(Point::new(x, vl.bottom - vol_h), Size::new(vw, vol_h.max(1.0)), vc);
        }
        // Volume MA5
        if sg < self.vol_ma5.len() {
            let mut pts: Vec<(f32, f32)> = Vec::new();
            for (i, bar) in bars.iter().enumerate() {
                let gi = sg + i;
                if gi >= self.vol_ma5.len() { break; }
                if let Some(v) = self.vol_ma5[gi] { pts.push((sx + i as f32 * sp + bw / 2.0, v_mp(v))); }
            }
            for w in pts.windows(2) {
                frame.stroke(&Path::line(Point::new(w[0].0, w[0].1), Point::new(w[1].0, w[1].1)),
                    canvas::Stroke::default().with_color(Color::from_rgba(1.0, 1.0, 0.6, 0.8)).with_width(1.5));
            }
        }
        frame.fill_text(canvas::Text {
            content: "VOL".to_string(), position: Point::new(sx + 5.0, vl.top + 3.0), color: text_color, size: iced::Pixels(10.0), ..Default::default()
        });

        // ── MACD ──
        for (i, bar) in bars.iter().enumerate() {
            let gi = sg + i;
            if gi >= self.macd.len() { break; }
            let m = &self.macd[gi];
            let x = sx + i as f32 * sp + bw / 2.0;
            // Histogram
            if let Some(bv) = m.bar {
                let y0 = m_mp(0.0);
                let y1 = m_mp(bv);
                let bar_color = if bv >= 0.0 { Color::from_rgba(0.9, 0.24, 0.24, 0.5) } else { Color::from_rgba(0.15, 0.65, 0.24, 0.5) };
                let w = (sp * 0.4).max(1.0);
                frame.fill_rectangle(Point::new(x - w / 2.0, y1.min(y0)), Size::new(w, (y1 - y0).abs().max(1.0)), bar_color);
            }
        }
        // MACD lines
        let mcolor = Color::from_rgb(0.3, 0.7, 1.0);
        let dcolor = Color::from_rgb(1.0, 0.4, 0.1);
        for (line_name, line_data, line_color) in [("DIF", &self.macd.iter().map(|m| m.dif).collect::<Vec<_>>(), &mcolor), ("DEA", &self.macd.iter().map(|m| m.dea).collect::<Vec<_>>(), &dcolor)] {
            let _ = line_name;
            let mut pts: Vec<(f32, f32)> = Vec::new();
            for (i, bar) in bars.iter().enumerate() {
                let gi = sg + i;
                if gi >= line_data.len() { break; }
                if let Some(v) = line_data[gi] { pts.push((sx + i as f32 * sp + bw / 2.0, m_mp(v))); }
            }
            for w in pts.windows(2) {
                frame.stroke(&Path::line(Point::new(w[0].0, w[0].1), Point::new(w[1].0, w[1].1)),
                    canvas::Stroke::default().with_color(*line_color).with_width(1.5));
            }
        }
        frame.fill_text(canvas::Text {
            content: "MACD(12,26,9)".to_string(), position: Point::new(sx + 5.0, ml.top + 3.0), color: text_color, size: iced::Pixels(10.0), ..Default::default()
        });

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
            let tw2 = label.len() as f32 * 6.5;
            frame.fill_text(canvas::Text {
                content: label, position: Point::new((x - tw2 / 2.0).max(sx), height - BOTTOM_PAD + 5.0),
                color: text_color, size: iced::Pixels(font_size), ..Default::default()
            });
        }
        if let Some(first) = bars.first() {
            frame.fill_text(canvas::Text {
                content: fmt_date(first.date), position: Point::new(sx, height - BOTTOM_PAD + 5.0),
                color: text_color, size: iced::Pixels(font_size), ..Default::default()
            });
        }
        if let Some(last) = bars.last() {
            let ll = fmt_date(last.date);
            let lx = sx + (bars.len() - 1) as f32 * sp;
            let tw2 = ll.len() as f32 * 6.5;
            frame.fill_text(canvas::Text {
                content: ll, position: Point::new((lx - tw2).max(sx), height - BOTTOM_PAD + 5.0),
                color: text_color, size: iced::Pixels(font_size), ..Default::default()
            });
        }

        // ── Crosshair ──
        if let Some(hover_idx) = self.hovered_index {
            let sg2 = lay.start_global;
            if hover_idx >= sg2 && hover_idx < sg2 + bars.len() {
                let li = hover_idx - sg2;
                let cx = sx + li as f32 * sp + bw / 2.0;
                let ch_color = Color::from_rgba(0.8, 0.8, 0.3, 0.7);
                self.draw_dashed_line(&mut frame, cx, kl.top, cx, ml.bottom, ch_color);
                let bar = &bars[li];
                let cy = k_mp(bar.close);
                self.draw_dashed_line(&mut frame, sx, cy, sx + tw, cy, ch_color);
                let hl_color = Color::from_rgba(1.0, 1.0, 0.5, 0.12);
                frame.fill_rectangle(Point::new((sx + li as f32 * sp).max(sx), kl.top), Size::new(sp.min(tw), kl.height), hl_color);
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(&self, _state: &mut Self::State, event: &canvas::Event, bounds: Rectangle, cursor: iced::mouse::Cursor) -> Option<canvas::Action<crate::app::Message>> {
        match event {
            canvas::Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => {
                if let Some(pos) = cursor.position_over(bounds) {
                    if pos.x >= bounds.x && pos.x <= bounds.x + bounds.width && pos.y >= bounds.y && pos.y <= bounds.y + bounds.height {
                        let sa = match delta { iced::mouse::ScrollDelta::Lines { y, .. } => *y, iced::mouse::ScrollDelta::Pixels { y, .. } => *y / 20.0 };
                        return Some(canvas::Action::publish(if sa > 0.0 { crate::app::Message::ZoomIn } else { crate::app::Message::ZoomOut }));
                    }
                }
                None
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                let bars = self.get_visible_bars();
                if bars.is_empty() { return Some(canvas::Action::publish(crate::app::Message::HoverBar(None))); }
                let total_width = bounds.width - 60.0;
                let bar_count = bars.len() as f32;
                let spacing = if bar_count > 0.0 { total_width / bar_count } else { 10.0 };
                if spacing <= 0.0 { return Some(canvas::Action::publish(crate::app::Message::HoverBar(None))); }
                let rel_x = position.x - (bounds.x + CHART_START_X);
                if rel_x < -20.0 || rel_x > total_width + 20.0 { return Some(canvas::Action::publish(crate::app::Message::HoverBar(None))); }
                let idx = (rel_x / spacing).round() as usize;
                if idx >= bars.len() { return Some(canvas::Action::publish(crate::app::Message::HoverBar(None))); }
                let total = self.bars.len();
                let end = total.saturating_sub(self.scroll_offset);
                let sg = end.saturating_sub(self.visible_count);
                Some(canvas::Action::publish(crate::app::Message::HoverBar(Some(sg + idx))))
            }
            canvas::Event::Mouse(iced::mouse::Event::CursorLeft) => Some(canvas::Action::publish(crate::app::Message::HoverBar(None))),
            _ => None,
        }
    }
}
