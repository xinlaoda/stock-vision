use crate::state::AppState;
use crate::app::Message;
use iced::widget::{container, text, Column};
use iced::{Element, Length};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content: Column<'_, Message> = Column::new().spacing(8).padding(16);

    let content = match &state.selected_stock {
        Some(code) => {
            let title = format!(
                "{} 基本面分析",
                state.stock_name.as_deref().unwrap_or(code)
            );
            let mut content = content.push(text(title).size(18));

            if let Some(health) = &state.financial_health {
                content = content
                    .push(text(format!("财务健康评分: {}/100", health.score)).size(16))
                    .push(text(&health.summary).size(14));

                for detail in &health.details {
                    content = content.push(text(format!("• {}", detail)).size(12));
                }
            } else {
                content = content.push(
                    text("点击加载财务数据").size(14).style(iced::Color::from_rgb(0.5, 0.5, 0.6)),
                );
            }

            content
        }
        None => content.push(
            text("请选择一只股票查看基本面").size(14).style(iced::Color::from_rgb(0.5, 0.5, 0.6)),
        ),
    };

    container(content).width(Length::Fill).height(Length::Fill).into()
}
