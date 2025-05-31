use freya::prelude::*;
use crate::core::types::Encoding;
use crate::ui::constants::{colors, spacing};

#[component]
pub fn EncodingSelector(encoding: Signal<Encoding>) -> Element {
    rsx! {
        rect {
            direction: "horizontal",
            cross_align: "center",
            rect {
                width: "120",
                margin: "0 0 0 16",
                label {
                    font_weight: "500",
                    color: colors::TEXT_COLOR,
                    "Bảng mã:"
                }
            }
            Dropdown {
                value: encoding.read().to_string(),
                
                DropdownItem {
                    value: "Unicode",
                    onpress: move |_| encoding.set(Encoding::Unicode),
                    label { "Unicode" }
                }
                DropdownItem {
                    value: "TCVN3",
                    onpress: move |_| encoding.set(Encoding::TCVN3), 
                    label { "TCVN3" }
                }
                DropdownItem {
                    value: "VNI-Win",
                    onpress: move |_| encoding.set(Encoding::VNIWin),
                    label { "VNI-Win" }
                }
            }
        }
    }
} 