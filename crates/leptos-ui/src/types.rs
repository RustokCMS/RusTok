use serde::{Deserialize, Serialize};

/// Размеры компонентов
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Size {
    Sm,
    Md,
    Lg,
}

impl Default for Size {
    fn default() -> Self {
        Self::Md
    }
}

/// Варианты цветовых схем для Button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Destructive,
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Primary
    }
}

impl ButtonVariant {
    pub fn classes(&self) -> &'static str {
        match self {
            Self::Primary => "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500",
            Self::Secondary => "bg-gray-600 text-white hover:bg-gray-700 focus:ring-gray-500",
            Self::Outline => "border border-gray-300 bg-white text-gray-700 hover:bg-gray-50 focus:ring-blue-500",
            Self::Ghost => "text-gray-700 hover:bg-gray-100 focus:ring-blue-500",
            Self::Destructive => "bg-red-600 text-white hover:bg-red-700 focus:ring-red-500",
        }
    }
}

/// Варианты цветовых схем для Badge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadgeVariant {
    Default,
    Primary,
    Success,
    Warning,
    Danger,
}

impl Default for BadgeVariant {
    fn default() -> Self {
        Self::Default
    }
}

impl BadgeVariant {
    pub fn classes(&self) -> &'static str {
        match self {
            Self::Default => "bg-gray-100 text-gray-800",
            Self::Primary => "bg-blue-100 text-blue-800",
            Self::Success => "bg-green-100 text-green-800",
            Self::Warning => "bg-yellow-100 text-yellow-800",
            Self::Danger => "bg-red-100 text-red-800",
        }
    }
}
