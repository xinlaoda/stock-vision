use iced::{Color, Theme};

pub struct SidebarStyle;

impl iced::widget::container::StyleSheet for SidebarStyle {
    type Style = Theme;

    fn appearance(&self, _theme: &Theme) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(Color::from_rgb(0.08, 0.08, 0.12).into()),
            ..Default::default()
        }
    }
}
