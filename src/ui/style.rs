pub mod palette {
    use iced::Color;
    pub const BG_DARK: Color = Color::from_rgb(0.07, 0.07, 0.10);
    pub const BG_MID: Color = Color::from_rgb(0.10, 0.10, 0.14);
    pub const BG_LIGHT: Color = Color::from_rgb(0.14, 0.14, 0.19);
    pub const BG_HOVER: Color = Color::from_rgb(0.18, 0.18, 0.24);
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.92, 0.92, 0.95);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.55, 0.63);
    pub const TEXT_ACCENT: Color = Color::from_rgb(0.35, 0.60, 0.95);
    pub const BORDER: Color = Color::from_rgb(0.20, 0.20, 0.26);
    pub const RISE: Color = Color::from_rgb(0.90, 0.24, 0.24);
    pub const FALL: Color = Color::from_rgb(0.15, 0.65, 0.24);
    pub const ACCENT: Color = Color::from_rgb(0.24, 0.50, 0.85);
}

pub fn sidebar() -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    |_| iced::widget::container::Style {
        background: Some(palette::BG_DARK.into()),
        ..Default::default()
    }
}

pub fn panel() -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    |_| iced::widget::container::Style {
        background: Some(palette::BG_MID.into()),
        ..Default::default()
    }
}
