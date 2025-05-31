/// Application title with version
pub const APP_TITLE: &str = concat!("VKey ", env!("CARGO_PKG_VERSION"), " - VKey Bộ Gõ Tiếng Việt");

/// Default window width
pub const APP_WIDTH: f64 = 600.0;

/// Default window height
pub const APP_HEIGHT: f64 = 300.0;

/// Default text font size
pub const TEXT_FONT_SIZE: &str = "14";

/// Colors
pub mod colors {
    /// Background color for the main window
    pub const BG_COLOR: &str = "rgb(45, 55, 72)";
    
    /// Background color for the control panel
    pub const PANEL_BG_COLOR: &str = "rgb(74, 85, 104)";
    
    /// Text color
    pub const TEXT_COLOR: &str = "rgb(226, 232, 240)";
    
    /// Accent color for interactive elements
    pub const ACCENT_COLOR: &str = "rgb(49, 130, 206)";
}

/// Spacing and sizing
pub mod spacing {
    /// Default padding for containers
    pub const CONTAINER_PADDING: &str = "20";
    
    /// Default spacing between elements
    pub const ELEMENT_SPACING: &str = "16";
    
    /// Default corner radius
    pub const CORNER_RADIUS: &str = "8";
} 