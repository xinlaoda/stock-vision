use iced::Color;
use std::sync::atomic::{AtomicU8, Ordering};

/// Thread-safe current theme tracking
static CURRENT_THEME: AtomicU8 = AtomicU8::new(0); // 0 = Dark, 1 = Light

pub fn set_current_theme(mode: ThemeMode) {
    CURRENT_THEME.store(match mode { ThemeMode::Dark => 0, ThemeMode::Light => 1 }, Ordering::Relaxed);
}

pub fn current_theme() -> ThemeMode {
    match CURRENT_THEME.load(Ordering::Relaxed) {
        0 => ThemeMode::Dark,
        _ => ThemeMode::Light,
    }
}

/// Current theme colors — always call `colors()` to get the active theme's colors.
/// Direct field access is the preferred way to use colors throughout the app.
/// Example: `style::colors().text_primary` instead of `style::palette::TEXT_PRIMARY`.

static DARK: ThemeColors = ThemeColors {
    bg_dark: Color::from_rgb(0.07, 0.07, 0.10),
    bg_mid: Color::from_rgb(0.10, 0.10, 0.14),
    bg_light: Color::from_rgb(0.14, 0.14, 0.19),
    text_primary: Color::from_rgb(0.92, 0.92, 0.95),
    text_secondary: Color::from_rgb(0.55, 0.55, 0.63),
    text_accent: Color::from_rgb(0.35, 0.60, 0.95),
    border: Color::from_rgb(0.20, 0.20, 0.26),
    rise: Color::from_rgb(0.90, 0.24, 0.24),
    fall: Color::from_rgb(0.15, 0.65, 0.24),
    accent: Color::from_rgb(0.24, 0.50, 0.85),
};

static LIGHT: ThemeColors = ThemeColors {
    bg_dark: Color::from_rgb(0.90, 0.90, 0.92),
    bg_mid: Color::from_rgb(0.95, 0.95, 0.97),
    bg_light: Color::from_rgb(0.85, 0.85, 0.88),
    text_primary: Color::from_rgb(0.10, 0.10, 0.12),
    text_secondary: Color::from_rgb(0.40, 0.40, 0.45),
    text_accent: Color::from_rgb(0.20, 0.45, 0.85),
    border: Color::from_rgb(0.75, 0.75, 0.78),
    rise: Color::from_rgb(0.90, 0.24, 0.24),
    fall: Color::from_rgb(0.15, 0.65, 0.24),
    accent: Color::from_rgb(0.24, 0.50, 0.85),
};

/// Get the currently active theme's colors
pub fn colors() -> &'static ThemeColors {
    match current_theme() {
        ThemeMode::Dark => &DARK,
        ThemeMode::Light => &LIGHT,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub bg_dark: Color,
    pub bg_mid: Color,
    pub bg_light: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_accent: Color,
    pub border: Color,
    pub rise: Color,
    pub fall: Color,
    pub accent: Color,
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



pub fn sidebar(_theme: ThemeMode) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_| iced::widget::container::Style {
        background: Some(colors().bg_dark.into()),
        ..Default::default()
    }
}

pub fn panel(_theme: ThemeMode) -> impl Fn(&iced::Theme) -> iced::widget::container::Style {
    move |_| iced::widget::container::Style {
        background: Some(colors().bg_mid.into()),
        ..Default::default()
    }
}
