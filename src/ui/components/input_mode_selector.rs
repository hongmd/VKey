use freya::prelude::*;
use crate::core::types::InputMode;
use crate::ui::constants::{colors, spacing};

#[component]
pub fn InputModeSelector(input_mode: Signal<InputMode>) -> Element {
    rsx! {
        rect {
            direction: "horizontal", 
            cross_align: "center",
            padding: "16 0 0 0",
            
            rect {
                width: "120",
                label {
                    font_weight: "500",
                    color: colors::TEXT_COLOR,
                    "Chế độ gõ:"
                }
            }
            
            rect {
                direction: "horizontal",
                spacing: spacing::ELEMENT_SPACING,
                
                // Vietnamese radio
                rect {
                    direction: "horizontal",
                    cross_align: "center", 
                    spacing: "6",
                    onclick: move |_| input_mode.set(InputMode::Vietnamese),
                    
                    rect {
                        width: "16",
                        height: "16",
                        corner_radius: "8",
                        border: format!("2 solid {}", colors::ACCENT_COLOR),
                        background: if input_mode.read().clone() == InputMode::Vietnamese {
                            colors::ACCENT_COLOR
                        } else {
                            "transparent"
                        },
                    }
                    label {
                        color: colors::TEXT_COLOR,
                        "Tiếng Việt"
                    }
                }
                
                // English radio
                rect {
                    direction: "horizontal",
                    cross_align: "center",
                    spacing: "6", 
                    onclick: move |_| input_mode.set(InputMode::English),
                    
                    rect {
                        width: "16",
                        height: "16",
                        corner_radius: "8",
                        border: format!("2 solid {}", colors::ACCENT_COLOR),
                        background: if input_mode.read().clone() == InputMode::English {
                            colors::ACCENT_COLOR
                        } else {
                            "transparent"
                        },
                    }
                    label {
                        color: colors::TEXT_COLOR,
                        "English"
                    }
                }
            }
        }
    }
} 