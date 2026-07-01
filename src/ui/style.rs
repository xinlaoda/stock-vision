use iced::{Background, Border, Color, Theme, Vector};

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

// ── Sidebar ──
pub struct SidebarStyle;

impl iced::widget::container::StyleSheet for SidebarStyle {
    type Style = Theme;
    fn appearance(&self, _theme: &Theme) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(palette::BG_DARK.into()),
            ..Default::default()
        }
    }
}

// ── Main Panel ──
pub struct PanelStyle;

impl iced::widget::container::StyleSheet for PanelStyle {
    type Style = Theme;
    fn appearance(&self, _theme: &Theme) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(palette::BG_MID.into()),
            ..Default::default()
        }
    }
}

// ── Search Input ──
pub struct SearchInputStyle;

impl iced::widget::text_input::StyleSheet for SearchInputStyle {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: palette::BG_LIGHT.into(),
            border: Border { radius: 6.0.into(), width: 1.0, color: palette::BORDER },
            icon_color: palette::TEXT_SECONDARY,
        }
    }
    fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        iced::widget::text_input::Appearance {
            background: palette::BG_LIGHT.into(),
            border: Border { radius: 6.0.into(), width: 1.0, color: palette::ACCENT },
            icon_color: palette::TEXT_SECONDARY,
        }
    }
    fn placeholder_color(&self, _style: &Self::Style) -> Color { palette::TEXT_SECONDARY }
    fn value_color(&self, _style: &Self::Style) -> Color { palette::TEXT_PRIMARY }
    fn disabled_color(&self, _style: &Self::Style) -> Color { palette::TEXT_SECONDARY }
    fn selection_color(&self, _style: &Self::Style) -> Color { palette::ACCENT }
    fn disabled(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        self.active(_style)
    }
}

// ── Primary Button (Search) ──
pub struct PrimaryButton;

impl iced::widget::button::StyleSheet for PrimaryButton {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(palette::ACCENT.into()),
            border: Border { radius: 6.0.into(), width: 0.0, color: Color::TRANSPARENT },
            text_color: palette::TEXT_PRIMARY,
            shadow_offset: Vector::default(),
            shadow: Default::default(),
        }
    }
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(Color::from_rgb(0.30, 0.56, 0.92).into()),
            ..self.active(_style)
        }
    }
    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        self.active(_style)
    }
}

// ── Nav Button (sidebar) ──
pub struct NavButton;

impl iced::widget::button::StyleSheet for NavButton {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(palette::BG_DARK.into()),
            border: Border { radius: 4.0.into(), width: 0.0, color: Color::TRANSPARENT },
            text_color: palette::TEXT_PRIMARY,
            shadow_offset: Vector::default(),
            shadow: Default::default(),
        }
    }
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(palette::BG_HOVER.into()),
            border: Border { radius: 4.0.into(), width: 1.0, color: palette::BORDER },
            ..self.active(_style)
        }
    }
    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(palette::BG_LIGHT.into()),
            ..self.active(_style)
        }
    }
}

// ── Search Result Button ──
pub struct SearchResultButton;

impl iced::widget::button::StyleSheet for SearchResultButton {
    type Style = Theme;
    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(palette::BG_DARK.into()),
            border: Border { radius: 4.0.into(), width: 0.0, color: Color::TRANSPARENT },
            text_color: palette::TEXT_PRIMARY,
            shadow_offset: Vector::default(),
            shadow: Default::default(),
        }
    }
    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(palette::BG_HOVER.into()),
            ..self.active(_style)
        }
    }
    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(palette::BG_LIGHT.into()),
            ..self.active(_style)
        }
    }
}
