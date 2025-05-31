use freya::prelude::*;
use crate::core::types::InputType;
use crate::ui::constants::{colors, spacing};

#[component]
pub fn InputTypeSelector(input_type: Signal<InputType>) -> Element {
    rsx! {
        rect {
            direction: "horizontal",
            cross_align: "center",
            rect {
                width: "120",
                label {
                    font_weight: "500",
                    color: colors::TEXT_COLOR,
                    font_size: "14",
                    "Kiểu gõ:"
                }
            }
            Dropdown {
                value: input_type.read().to_string(),
                
                DropdownItem {
                    value: "Telex",
                    onpress: move |_| input_type.set(InputType::Telex),
                    label { "Telex" }
                }
                DropdownItem {
                    value: "VNI", 
                    onpress: move |_| input_type.set(InputType::VNI),
                    label { "VNI" }
                }
                DropdownItem {
                    value: "VIQR",
                    onpress: move |_| input_type.set(InputType::VIQR),
                    label { "VIQR" }
                }
            }
        }
    }
} 