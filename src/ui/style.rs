use iced::Color;

pub mod palette {
    use iced::Color;
    // Dark theme (default)
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
    
    // Light theme
    pub const LIGHT_BG_DARK: Color = Color::from_rgb(0.90, 0.90, 0.92);
    pub const LIGHT_BG_MID: Color = Color::from_rgb(0.95, 0.95, 0.97);
    pub const LIGHT_BG_LIGHT: Color = Color::from_rgb(0.85, 0.85, 0.88);
    pub const LIGHT_TEXT_PRIMARY: Color = Color::from_rgb(0.10, 0.10, 0.12);
    pub const LIGHT_TEXT_SECONDARY: Color = Color::from_rgb(0.40, 0.40, 0.45);
    pub const LIGHT_TEXT_ACCENT: Color = Color::from_rgb(0.20, 0.45, 0.85);
    pub const LIGHT_BORDER: Color = Color::from_rgb(0.75, 0.75, 0.78);
}

/// Current theme mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    pub fn label(&self) -> &str {
        match self {
            ThemeMode::Dark => "深色",
            ThemeMode::Light => "亮色",
        }
    }
}

/// Apply the current theme to the palette values
pub fn apply_theme(theme: ThemeMode) -> AppliedTheme {
    match theme {
        ThemeMode::Dark => AppliedTheme {
            bg_dark: palette::BG_DARK,
            bg_mid: palette::BG_MID,
            bg_light: palette::BG_LIGHT,
            text_primary: palette::TEXT_PRIMARY,
            text_secondary: palette::TEXT_SECONDARY,
            text_accent: palette::TEXT_ACCENT,
            border: palette::BORDER,
        },
        ThemeMode::Light => AppliedTheme {
            bg_dark: palette::LIGHT_BG_DARK,
            bg_mid: palette::LIGHT_BG_MID,
            bg_light: palette::LIGHT_BG_LIGHT,
            text_primary: palette::LIGHT_TEXT_PRIMARY,
            text_secondary: palette::LIGHT_TEXT_SECONDARY,
            text_accent: palette::LIGHT_TEXT_ACCENT,
            border: palette::LIGHT_BORDER,
        },
    }
}

pub struct AppliedTheme {
    pub bg_dark: Color,
    pub bg_mid: Color,
    pub bg_light: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_accent: Color,
    pub border: Color,
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
