use crate::app::Message;
use crate::state::MarketIndexData;
use crate::ui::style;
use iced::widget::canvas::{self, Frame, Geometry, Program};
use iced::widget::{column, container, row, text};
use iced::{Color, Element, Fill, Point, Rectangle, Size, Theme};

// ── 迷你走势图 Canvas ──

struct IndexChart {
    bars: Vec<stock_vision_data_model::DailyBar>,
    color: Color,
}

impl Program<Message> for IndexChart {
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
            frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(w, h), Color::from_rgb(0.07, 0.07, 0.11));
            return vec![frame.into_geometry()];
        }

        frame.fill_rectangle(Point::new(0.0, 0.0), Size::new(w, h), Color::from_rgb(0.07, 0.07, 0.11));

        let lm = 55.0;
        let rm = 5.0;
        let tm = 5.0;
        let bm = 18.0;
        let dw = w - lm - rm;
        let dh = h - tm - bm;

        let prices: Vec<f64> = self.bars.iter().map(|b| b.close).collect();
        let min = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max - min).max(1.0);
        let n = prices.len();
        let step_x = if n > 1 { dw / (n - 1) as f32 } else { dw };

        let to_x = |i: usize| -> f32 { lm + i as f32 * step_x };
        let to_y = |v: f64| -> f32 { tm + dh - ((v - min) / range * dh as f64) as f32 };

        let txt_color = Color::from_rgb(0.55, 0.55, 0.63);
        let grid_color = Color::from_rgba(0.18, 0.18, 0.22, 0.5);

        // Y轴网格线 + 标签
        for i in 0..5 {
            let ratio = 1.0 - i as f64 / 4.0;
            let y = tm + dh * (1.0 - ratio) as f32;
            frame.fill_rectangle(Point::new(lm, y), Size::new(dw, 0.5), grid_color);
            let val = min + range * ratio;
            frame.fill_text(canvas::Text {
                content: format!("{:.0}", val),
                position: Point::new(2.0, y - 5.0),
                color: txt_color,
                size: iced::Pixels(9.0),
                ..Default::default()
            });
        }

        // 折线
        for i in 0..n - 1 {
            let seg = canvas::Path::line(
                Point::new(to_x(i), to_y(prices[i])),
                Point::new(to_x(i + 1), to_y(prices[i + 1])),
            );
            frame.stroke(&seg, canvas::Stroke::default().with_color(self.color).with_width(2.0));
        }

        // 填充区域
        if n > 0 {
            let fill_path = canvas::Path::new(|b| {
                b.move_to(Point::new(lm, tm + dh));
                b.line_to(Point::new(lm, to_y(prices[0])));
                for i in 1..n {
                    b.line_to(Point::new(to_x(i), to_y(prices[i])));
                }
                b.line_to(Point::new(to_x(n - 1), tm + dh));
                b.close();
            });
            frame.fill(&fill_path, Color::from_rgba(self.color.r, self.color.g, self.color.b, 0.12));
        }

        // X轴日期标签
        if let (Some(first), Some(last)) = (self.bars.first(), self.bars.last()) {
            let fmt = |d: &chrono::NaiveDate| d.format("%m-%d").to_string();
            frame.fill_text(canvas::Text {
                content: fmt(&first.date),
                position: Point::new(lm, h - bm + 4.0),
                color: txt_color, size: iced::Pixels(9.0), ..Default::default()
            });
            let ll = fmt(&last.date);
            let tw = ll.len() as f32 * 5.5;
            frame.fill_text(canvas::Text {
                content: ll,
                position: Point::new((to_x(n - 1) - tw).max(lm), h - bm + 4.0),
                color: txt_color, size: iced::Pixels(9.0), ..Default::default()
            });
        }

        vec![frame.into_geometry()]
    }
}

// ── 首页视图 ──

fn index_card<'a>(
    name: &'a str,
    idx: &'a MarketIndexData,
    line_color: Color,
) -> Element<'a, Message> {
    let ch_color = if idx.change >= 0.0 { style::palette::RISE } else { style::palette::FALL };
    let arrow = if idx.change >= 0.0 { "▲" } else { "▼" };
    let sign = if idx.change >= 0.0 { "+" } else { "" };

    let chart = canvas::Canvas::new(IndexChart {
        bars: idx.bars.clone(),
        color: line_color,
    })
    .width(200)
    .height(90);

    let row_content = row(vec![
        column(vec![
            text(name).size(20.0).color(style::palette::TEXT_PRIMARY).into(),
            text("").size(4.0).into(),
            text(format!("{:.2}", idx.price)).size(32.0).color(ch_color).into(),
            row(vec![
                text(format!("{} {}{:.2}", arrow, sign, idx.change)).size(14.0).color(ch_color).into(),
                text(format!("({:+.2}%)", idx.change_pct)).size(14.0).color(ch_color).into(),
            ]).spacing(4).into(),
        ]).spacing(2).width(180).into(),
        container(chart).into(),
    ])
    .spacing(16)
    .align_y(iced::alignment::Vertical::Center);

    container(row_content)
        .width(Fill)
        .padding(20)
        .style(|_: &Theme| iced::widget::container::Style {
            background: Some(style::palette::BG_DARK.into()),
            border: iced::border::rounded(12),
            ..Default::default()
        })
        .into()
}

pub fn view(state: &crate::state::AppState) -> Element<'_, Message> {
    let now_str = state.current_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let names = ["上证指数", "深证成指", "创业板指", "科创50"];
    let colors = [
        Color::from_rgb(0.9, 0.3, 0.3),
        Color::from_rgb(0.3, 0.6, 0.9),
        Color::from_rgb(0.2, 0.8, 0.4),
        Color::from_rgb(1.0, 0.65, 0.0),
    ];

    let mut content = column![].spacing(12).padding(24);

    content = content.push(text("Stock Vision").size(32.0).color(Color::from_rgb(1.0, 0.65, 0.0)));
    content = content.push(text("A股行情分析与投资工具").size(16.0).color(style::palette::TEXT_SECONDARY));
    content = content.push(text(now_str).size(14.0).color(style::palette::TEXT_ACCENT));
    content = content.push(text("").size(8.0));

    content = content.push(text("市场概况").size(20.0).color(style::palette::TEXT_PRIMARY));

    // 遍历指数数据，一行一个卡片
    for (i, name) in names.iter().enumerate() {
        if let Some(idx) = state.market_indices.get(i) {
            content = content.push(index_card(name, idx, colors[i]));
        }
    }

    content = content.push(text("").size(12.0));

    content = content.push(text("快速开始").size(20.0).color(style::palette::TEXT_PRIMARY));
    let tips = [
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
