use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};

mod core;
mod error;
mod platform;
mod ui;

use ui::VKeyApp;

fn main() {
    eprintln!("Starting VKey application...");
    
    let result = std::panic::catch_unwind(|| {
        Application::new().run(|cx: &mut App| {
            eprintln!("Creating window...");
            let bounds = Bounds::centered(None, size(px(650.), px(560.)), cx);
            match cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    eprintln!("Initializing VKeyApp...");
                                         cx.new(|_| VKeyApp::new())
                },
            ) {
                Ok(_) => eprintln!("Window created successfully"),
                Err(e) => eprintln!("Failed to create window: {:?}", e),
            }
        });
    });
    
    match result {
        Ok(_) => eprintln!("Application finished normally"),
        Err(e) => eprintln!("Application panicked: {:?}", e),
    }
}