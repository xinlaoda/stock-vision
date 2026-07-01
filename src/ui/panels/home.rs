use crate::app::Message;
use crate::state::MarketIndexData;
use crate::ui::style;
use iced::widget::canvas::{self, Frame, Geometry, Program};
use iced::widget::{column, container, responsive, row, text};
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
        let tm = 5.0;
        let bm = 18.0;
        let dw = w - lm - 5.0;
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

        // Y 轴网格 + 标签
        for i in 0..4 {
            let ratio = 1.0 - i as f64 / 3.0;
            let y = tm + dh * (1.0 - ratio) as f32;
            frame.fill_rectangle(Point::new(lm, y), Size::new(dw, 0.5), grid_color);
            frame.fill_text(canvas::Text {
                content: format!("{:.0}", min + range * ratio),
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

        // 填充
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

        // X 轴日期
        if let (Some(first), Some(last)) = (self.bars.first(), self.bars.last()) {
            let fmt = |d: &chrono::NaiveDate| d.format("%m-%d").to_string();
            frame.fill_text(canvas::Text {
                content: fmt(&first.date), position: Point::new(lm, h - bm + 4.0),
                color: txt_color, size: iced::Pixels(9.0), ..Default::default()
            });
            let ll = fmt(&last.date);
            let tw = ll.len() as f32 * 5.5;
            frame.fill_text(canvas::Text {
                content: ll, position: Point::new((to_x(n - 1) - tw).max(lm), h - bm + 4.0),
                color: txt_color, size: iced::Pixels(9.0), ..Default::default()
            });
        }

        vec![frame.into_geometry()]
    }
}

// ── 单张指数卡片 ──

fn index_card<'a>(
    name: &'a str,
    idx: &'a MarketIndexData,
    line_color: Color,
    chart_w: f32,
    chart_h: f32,
) -> Element<'a, Message> {
    let ch_color = if idx.change >= 0.0 { style::colors().rise } else { style::colors().fall };
    let arrow = if idx.change >= 0.0 { "▲" } else { "▼" };
    let sign = if idx.change >= 0.0 { "+" } else { "" };

    let chart = canvas::Canvas::new(IndexChart {
        bars: idx.bars.clone(),
        color: line_color,
    })
    .width(chart_w)
    .height(chart_h);

    let body = row(vec![
        column(vec![
            text(name).size(18.0).color(style::colors().text_primary).into(),
            text(format!("{:.2}", idx.price)).size(28.0).color(ch_color).into(),
            row(vec![
                text(format!("{} {}{:.2}", arrow, sign, idx.change)).size(13.0).color(ch_color).into(),
                text(format!("({:+.2}%)", idx.change_pct)).size(13.0).color(ch_color).into(),
            ]).spacing(4).into(),
        ]).spacing(2).width(140).into(),
        container(chart).into(),
    ])
    .spacing(12)
    .align_y(iced::alignment::Vertical::Center);

    container(body)
        .width(Fill)
        .padding(16)
        .style(|_: &Theme| iced::widget::container::Style {
            background: Some(style::colors().bg_dark.into()),
            border: iced::border::rounded(12),
            ..Default::default()
        })
        .into()
}

// ── 首页视图 ──

pub fn view(state: &crate::state::AppState) -> Element<'_, Message> {
    let now_str = state.current_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let names = ["上证指数", "深证成指", "创业板指", "科创50"];
    let colors = [
        Color::from_rgb(0.9, 0.3, 0.3),
        Color::from_rgb(0.3, 0.6, 0.9),
        Color::from_rgb(0.2, 0.8, 0.4),
        Color::from_rgb(1.0, 0.65, 0.0),
    ];

    let header = column![
        text("Stock Vision").size(32.0).color(Color::from_rgb(1.0, 0.65, 0.0)),
        text("A股行情分析与投资工具").size(16.0).color(style::colors().text_secondary),
        text(now_str).size(14.0).color(style::colors().text_accent),
        text("").size(8.0),
        text("市场概况").size(20.0).color(style::colors().text_primary),
    ];

    let tips = [
        "🔍 在左侧搜索框输入股票代码或名称，搜索并选择股票",
        "📈 查看 K 线走势、成交量、MACD 等技术指标",
        "📋 分析公司基本面财务数据",
        "📊 管理自选股列表，快速切换关注的股票",
        "🖱️ 鼠标滚轮缩放K线，点击K线添加水平画线",
    ];

    let mut tip_col = column![
        text("快速开始").size(20.0).color(style::colors().text_primary),
    ];
    for tip in tips {
        tip_col = tip_col.push(text(tip).size(14.0).color(style::colors().text_secondary));
    }

    // 使用 responsive 实现自适应流式网格
    let market_grid = responsive(move |size: Size| {
        let w = size.width;
        // 宽度 < 900px 时单列，否则双列
        let single_col = w < 900.0;

        let chart_w = if single_col { 160.0 } else { 140.0 };
        let chart_h = if single_col { 80.0 } else { 70.0 };

        if single_col {
            let mut col = column![].spacing(12);
            for (i, name) in names.iter().enumerate() {
                if let Some(idx) = state.market_indices.get(i) {
                    col = col.push(index_card(name, idx, colors[i], chart_w, chart_h));
                }
            }
            col.into()
        } else {
            // 2 列网格
            let mut main_col = column![].spacing(12);
            for chunk in names.chunks(2) {
                let mut row_children: Vec<Element<'_, Message>> = Vec::new();
                for (i, name) in chunk.iter().enumerate() {
                    let global_idx = names.iter().position(|n| n == name).unwrap_or(0);
                    if let Some(idx) = state.market_indices.get(global_idx) {
                        row_children.push(
                            container(index_card(name, idx, colors[global_idx], chart_w, chart_h))
                                .width(Fill)
                                .into()
                        );
                    }
                }
                main_col = main_col.push(row(row_children).spacing(12).width(Fill));
            }
            main_col.into()
        }
    });

    // 组装
    let body = column![
        header,
        market_grid,
        text("").size(12.0),
        tip_col,
    ].spacing(12).padding(24);

    container(body).width(Fill).height(Fill).into()
}
