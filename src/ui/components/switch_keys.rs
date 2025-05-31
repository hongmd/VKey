use freya::prelude::*;
use crate::ui::constants::{colors, spacing};

#[component]
pub fn SwitchKeys(
    shift_enabled: Signal<bool>,
    ctrl_enabled: Signal<bool>,
    cmd_enabled: Signal<bool>,
    home_enabled: Signal<bool>,
    beep_enabled: Signal<bool>,
) -> Element {
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
                    "Phím chuyển:"
                }
            }
            
            rect {
                direction: "horizontal",
                spacing: spacing::ELEMENT_SPACING,
                cross_align: "center",
                
                // Shift key checkbox
                rect {
                    direction: "horizontal",
                    cross_align: "center",
                    spacing: "6",
                    
                    Switch {
                        enabled: shift_enabled.read().clone(),
                        ontoggled: move |_| {
                            shift_enabled.toggle();
                        }
                    }
                    label {
                        color: colors::TEXT_COLOR,
                        "⇧"
                    }
                }
                
                // Ctrl key checkbox  
                rect {
                    direction: "horizontal",
                    cross_align: "center", 
                    spacing: "6",
                    
                    Switch {
                        enabled: ctrl_enabled.read().clone(),
                        ontoggled: move |_| {
                            ctrl_enabled.toggle();
                        }
                    }
                    label {
                        color: colors::TEXT_COLOR,
                        "⌃"
                    }
                }
                
                // Cmd key checkbox
                rect {
                    direction: "horizontal",
                    cross_align: "center",
                    spacing: "6",
                    
                    Switch {
                        enabled: cmd_enabled.read().clone(),
                        ontoggled: move |_| {
                            cmd_enabled.toggle();
                        }
                    }
                    label {
                        color: colors::TEXT_COLOR,
                        "⌘"
                    }
                }
                
                // Home key checkbox
                rect {
                    direction: "horizontal", 
                    cross_align: "center",
                    spacing: "6",
                    
                    Switch {
                        enabled: home_enabled.read().clone(),
                        ontoggled: move |_| {
                            home_enabled.toggle();
                        }
                    }
                    label {
                        color: colors::TEXT_COLOR,
                        "🏠"
                    }
                }
                
                // Key display
                rect {
                    background: colors::BG_COLOR,
                    border: format!("2 solid {}", colors::ACCENT_COLOR),
                    corner_radius: spacing::CORNER_RADIUS,
                    padding: "8 16",
                    min_width: "60",
                    main_align: "center",
                    
                    label {
                        color: colors::TEXT_COLOR,
                        font_family: "monospace",
                        "|"
                    }
                }
                
                // Beep checkbox
                rect {
                    direction: "horizontal",
                    cross_align: "center",
                    spacing: "6",
                    
                    Switch {
                        enabled: beep_enabled.read().clone(),
                        ontoggled: move |_| {
                            beep_enabled.toggle();
                        }
                    }
                    label {
                        color: colors::TEXT_COLOR,
                        "Kêu beep"
                    }
                }
            }
        }
    }
} 