extern crate vi;

use freya::prelude::*;

const APP_TITLE: &str = concat!("VKey ", env!("CARGO_PKG_VERSION"), " - VKey Bộ Gõ Tiếng Việt");
const APP_WIDTH: f64 = 400.0;
const APP_HEIGHT: f64 = 300.0;

fn main() {
    launch_with_props(app, APP_TITLE, (APP_WIDTH, APP_HEIGHT));
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Telex,
    VNI,
    VIQR,
}

impl std::fmt::Display for InputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputType::Telex => write!(f, "Telex"),
            InputType::VNI => write!(f, "VNI"),
            InputType::VIQR => write!(f, "VIQR"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Encoding {
    Unicode,
    TCVN3,
    VNIWin,
}

impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Encoding::Unicode => write!(f, "Unicode"),
            Encoding::TCVN3 => write!(f, "TCVN3"),
            Encoding::VNIWin => write!(f, "VNI-Win"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Vietnamese,
    English,
}

impl std::fmt::Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputMode::Vietnamese => write!(f, "Tiếng Việt"),
            InputMode::English => write!(f, "English"),
        }
    }
}

fn app() -> Element {
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
            background: "rgb(45, 55, 72)",
            padding: "20",
            
            rect {
                background: "rgb(74, 85, 104)",
                corner_radius: "8",
                padding: "20",
                width: "600",
                shadow: "0 4 6 0 rgb(0, 0, 0, 0.1)",
                
                // Title
                label {
                    font_size: "18",
                    font_weight: "600",
                    color: "rgb(226, 232, 240)",
                    "Điều khiển"
                }
                
                // Input Type Row
                rect {
                    direction: "horizontal",
                    cross_align: "center",
                    padding: "16 0 0 0",
                    
                    rect {
                        width: "120",
                        label {
                            font_weight: "500",
                            color: "rgb(226, 232, 240)",
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
                
                // Encoding Row
                rect {
                    direction: "horizontal",
                    cross_align: "center",
                    padding: "16 0 0 0",
                    
                    rect {
                        width: "120",
                        label {
                            font_weight: "500",
                            color: "rgb(226, 232, 240)",
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
                
                // Switch Keys Row
                rect {
                    direction: "horizontal",
                    cross_align: "center",
                    padding: "16 0 0 0",
                    
                    rect {
                        width: "120",
                        label {
                            font_weight: "500",
                            color: "rgb(226, 232, 240)",
                            "Phím chuyển:"
                        }
                    }
                    
                    rect {
                        direction: "horizontal",
                        spacing: "16",
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
                                color: "rgb(226, 232, 240)",
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
                                color: "rgb(226, 232, 240)",
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
                                color: "rgb(226, 232, 240)",
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
                                color: "rgb(226, 232, 240)",
                                "🏠"
                            }
                        }
                        
                        // Key display
                        rect {
                            background: "rgb(45, 55, 72)",
                            border: "2 solid rgb(49, 130, 206)",
                            corner_radius: "4",
                            padding: "8 16",
                            min_width: "60",
                            main_align: "center",
                            
                            label {
                                color: "rgb(226, 232, 240)",
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
                                color: "rgb(226, 232, 240)",
                                "Kêu beep"
                            }
                        }
                    }
                }
                
                // Input Mode Row
                rect {
                    direction: "horizontal", 
                    cross_align: "center",
                    padding: "16 0 0 0",
                    
                    rect {
                        width: "120",
                        label {
                            font_weight: "500",
                            color: "rgb(226, 232, 240)",
                            "Chế độ gõ:"
                        }
                    }
                    
                    rect {
                        direction: "horizontal",
                        spacing: "20",
                        
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
                                border: "2 solid rgb(49, 130, 206)",
                                background: if input_mode.read().clone() == InputMode::Vietnamese {
                                    "rgb(49, 130, 206)"
                                } else {
                                    "transparent"
                                },
                            }
                            label {
                                color: "rgb(226, 232, 240)",
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
                                border: "2 solid rgb(49, 130, 206)",
                                background: if input_mode.read().clone() == InputMode::English {
                                    "rgb(49, 130, 206)"
                                } else {
                                    "transparent"
                                },
                            }
                            label {
                                color: "rgb(226, 232, 240)",
                                "English"
                            }
                        }
                    }
                }
            }
        }
    }
}
