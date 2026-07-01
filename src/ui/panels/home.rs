use crate::app::Message;
use crate::ui::style;
use iced::widget::canvas::{self, Frame, Geometry, Program};
use iced::widget::{column, container, row, text};
use iced::{Color, Element, Fill, Point, Rectangle, Size, Theme};

/// Generate simulated index sparkline data (price points)
fn mock_sparkline(seed: f64, count: usize) -> Vec<f64> {
    let mut pts = Vec::with_capacity(count);
    let mut val = 3000.0 + seed * 2000.0; // base around 3000-5000
    for i in 0..count {
        // Simulate some noise
        let noise = ((i as f64 * 1.7).sin() * 30.0
            + (i as f64 * 0.3).cos() * 50.0
            + (i as f64 * 5.0).sin() * 10.0);
        val += noise * 0.5;
        pts.push(val);
    }
    pts
}

/// Generate mock index price info for display
fn mock_index_info(name: &str, seed: f64) -> (Vec<f64>, f64, f64, f64) {
    let data = mock_sparkline(seed, 60);
    let latest = *data.last().unwrap_or(&3000.0);
    let first = data.first().copied().unwrap_or(3000.0);
    let change = latest - first;
    let change_pct = change / first * 100.0;
    (data, latest, change, change_pct)
}

// ── Sparkline Canvas ──

struct Sparkline {
    points: Vec<f64>,
    color: Color,
}

impl Program<Message> for Sparkline {
    type State = ();

    fn draw(&self, _state: &Self::State, renderer: &iced::Renderer, _theme: &Theme, bounds: Rectangle, _cursor: iced::mouse::Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;
        if self.points.len() < 2 || w <= 0.0 || h <= 0.0 {
            return vec![frame.into_geometry()];
        }

        let min = self.points.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.points.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max - min).max(1.0);

        let padding = 2.0;
        let draw_w = w - padding * 2.0;
        let draw_h = h - padding * 2.0;

        let step_x = draw_w / (self.points.len() - 1) as f32;
        let to_y = |v: f64| -> f32 { padding + draw_h - ((v - min) / range * draw_h as f64) as f32 };

        // Draw line segments
        for i in 0..self.points.len() - 1 {
            let x1 = padding + i as f32 * step_x;
            let x2 = padding + (i + 1) as f32 * step_x;
            let y1 = to_y(self.points[i]);
            let y2 = to_y(self.points[i + 1]);
            let segment = canvas::Path::line(Point::new(x1, y1), Point::new(x2, y2));
            frame.stroke(&segment, canvas::Stroke::default().with_color(self.color).with_width(1.5));
        }

        // Fill area under the line with a gradient-like transparent fill
        // Last point to bottom-right, then bottom-left
        let last_x = padding + (self.points.len() - 1) as f32 * step_x;
        let last_y = to_y(*self.points.last().unwrap());
        let first_y = to_y(self.points[0]);

        let fill_path = canvas::Path::new(|b| {
            b.move_to(Point::new(padding, draw_h + padding));
            b.line_to(Point::new(padding, first_y));
            for i in 1..self.points.len() {
                let x = padding + i as f32 * step_x;
                let y = to_y(self.points[i]);
                b.line_to(Point::new(x, y));
            }
            b.line_to(Point::new(last_x, draw_h + padding));
            b.close();
        });

        let fill_color = Color::from_rgba(self.color.r, self.color.g, self.color.b, 0.1);
        frame.fill(&fill_path, fill_color);

        vec![frame.into_geometry()]
    }
}

// ── Home Page View ──

pub fn view(state: &crate::state::AppState) -> Element<'_, Message> {
    let now_str = state.current_time.format("%Y-%m-%d %H:%M:%S").to_string();

    // Generate mock data for each index
    let indices = vec![
        ("上证指数", mock_index_info("上证指数", 0.0), Color::from_rgb(0.9, 0.3, 0.3)),
        ("深证成指", mock_index_info("深证成指", 0.5), Color::from_rgb(0.3, 0.6, 0.9)),
        ("创业板指", mock_index_info("创业板指", -0.2), Color::from_rgb(0.2, 0.8, 0.4)),
        ("科创50",   mock_index_info("科创50", 0.3), Color::from_rgb(1.0, 0.65, 0.0)),
    ];

    let mut content = column![].spacing(12).padding(24);

    // ── Header ──
    content = content.push(
        text("Stock Vision").size(32.0).color(Color::from_rgb(1.0, 0.65, 0.0))
    );
    content = content.push(
        text("A股行情分析与投资工具").size(16.0).color(style::palette::TEXT_SECONDARY)
    );
    content = content.push(text(now_str).size(14.0).color(style::palette::TEXT_ACCENT));
    content = content.push(text("").size(8.0));

    // ── Market Overview Cards ──
    content = content.push(
        text("市场概况").size(20.0).color(style::palette::TEXT_PRIMARY)
    );

    // Layout: 2x2 grid of index cards
    for chunk in indices.chunks(2) {
        let mut card_row = row![].spacing(16);
        for (name, (data, price, change, change_pct), line_color) in chunk {
            let ch_color = if *change >= 0.0 { style::palette::RISE } else { style::palette::FALL };
            let arrow = if *change >= 0.0 { "▲" } else { "▼" };

            // Sparkline canvas widget (120x50)
            let sparkline = canvas::Canvas::new(Sparkline {
                points: (*data).clone(),
                color: *line_color,
            })
            .width(120).height(50);

            let top_row: Element<'_, Message> = {
                let info_col = column![
                    text(*name).size(16.0).color(style::palette::TEXT_PRIMARY),
                    row![
                        text(format!("{:.2}", price)).size(22.0).color(ch_color),
                        text("").width(6),
                        text(format!("{} {}", arrow, if *change >= 0.0 { "+" } else { "" })).size(13.0).color(ch_color),
                        text(format!("{:.2}%", change_pct)).size(13.0).color(ch_color),
                    ].spacing(2).align_y(iced::alignment::Vertical::Center),
                ].spacing(4);
                let spacer: Element<'_, Message> = text("").width(12).into();
                let chart: Element<'_, Message> = iced::widget::container(sparkline).into();
                row(vec![info_col.into(), spacer, chart])
                    .align_y(iced::alignment::Vertical::Center)
                    .into()
            };

            let card = container(top_row)
            .width(Fill)
            .padding(16)
            .style(|_t: &Theme| iced::widget::container::Style {
                background: Some(style::palette::BG_DARK.into()),
                border: iced::border::rounded(8),
                ..Default::default()
            });

            card_row = card_row.push(card);
        }
        content = content.push(card_row);
    }

    content = content.push(text("").size(12.0));

    // ── Quick start guide ──
    content = content.push(
        text("快速开始").size(20.0).color(style::palette::TEXT_PRIMARY)
    );

    let tips = vec![
        "🔍 在左侧搜索框输入股票代码或名称，搜索并选择股票",
        "📈 查看 K 线走势、成交量、MACD 等技术指标",
        "📋 分析公司基本面财务数据",
        "📊 管理自选股列表，快速切换关注的股票",
        "🖱️ 鼠标滚轮缩放K线，点击K线添加水平画线",
    ];
    for tip in tips {
        content = content.push(text(tip).size(14.0).color(style::palette::TEXT_SECONDARY));
    }

    container(content).width(Fill).height(Fill).into()
}
