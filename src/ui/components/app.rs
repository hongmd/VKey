use freya::prelude::*;
use crate::core::types::{InputType, Encoding, InputMode};
use crate::core::config::AppConfig;
use crate::ui::constants::{colors, spacing, TEXT_FONT_SIZE};
use super::{InputTypeSelector, EncodingSelector, InputModeSelector, SwitchKeys};

#[component]
pub fn App() -> Element {
    let mut input_type = use_signal(|| InputType::Telex);
    let mut encoding = use_signal(|| Encoding::Unicode);
    let mut input_mode = use_signal(|| InputMode::English);
    
    // Switch key states
    let mut shift_enabled = use_signal(|| false);
    let mut ctrl_enabled = use_signal(|| false);
    let mut cmd_enabled = use_signal(|| true);
    let mut home_enabled = use_signal(|| true);
    let mut beep_enabled = use_signal(|| false);

    rsx! {
        rect {
            width: "100%",
            height: "100%",
            background: colors::BG_COLOR,
            padding: spacing::CONTAINER_PADDING,

            rect {
                direction: "horizontal",
                cross_align: "center",

                // Title
                label {
                    font_size: TEXT_FONT_SIZE,
                    font_weight: "600",
                    color: colors::TEXT_COLOR,
                    "Điều khiển"
                }
            }
            
            rect {
                background: colors::PANEL_BG_COLOR,
                corner_radius: spacing::CORNER_RADIUS,
                padding: spacing::CONTAINER_PADDING,
                width: "600",
                shadow: "0 4 6 0 rgb(0, 0, 0, 0.1)",
                
                InputTypeSelector { input_type }
                EncodingSelector { encoding }
                InputModeSelector { input_mode }
                SwitchKeys {
                    shift_enabled,
                    ctrl_enabled,
                    cmd_enabled,
                    home_enabled,
                    beep_enabled,
                }
            }
        }
    }
} 