extern crate vi;

use freya::prelude::*;

const APP_TITLE: &str = concat!("VKey ", env!("CARGO_PKG_VERSION"), " - VKey Bộ Gõ Tiếng Việt");
const APP_WIDTH: f64 = 400.0;
const APP_HEIGHT: f64 = 300.0;

fn main() {
    launch_with_props(app, APP_TITLE, (APP_WIDTH, APP_HEIGHT));
}

fn app() -> Element {
    let mut input_text = use_signal(|| String::new());
    let mut output_text = use_signal(|| String::new());

    let mut update_output = move |text: &str| {
        let mut result = String::new();
        vi::transform_buffer(&vi::VNI,  text.chars(), &mut result);
        output_text.set(result);
    };

    rsx!(
        rect {
            height: "100%",
            width: "100%",
            background: "rgb(30, 30, 30)",
            color: "white",
            padding: "16",
            direction: "column",
            rect {
                direction: "column",
                margin: "0 0 16 0",
                label {
                    font_size: "14",
                    "Input (VNI):"
                }
                rect {
                    background: "rgb(45, 45, 45)",
                    padding: "8",
                    corner_radius: "4",
                    Input {
                        value: input_text,
                        onchange: move |e: String| {
                            let text = e.clone();
                            input_text.set(e);
                            update_output(&text);
                        }
                    }
                }
            }
            rect {
                direction: "column",
                margin: "0 0 16 0",
                label {
                    font_size: "14",
                    "Output (Vietnamese):"
                }
                rect {
                    background: "rgb(45, 45, 45)",
                    padding: "8",
                    corner_radius: "4",
                    label {
                        "{output_text}"
                    }
                }
            }
        }
    )
}
