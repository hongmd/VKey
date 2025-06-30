use gpui::{
    App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size
};

mod core;
mod error;
mod platform;
mod ui;

use ui::VKeyApp;

#[cfg(target_os = "macos")]
use platform::system_integration;

fn main() {
    eprintln!("Starting VKey application...");
    
    let result = std::panic::catch_unwind(|| {
        // Check and request permissions before starting the application
        #[cfg(target_os = "macos")]
        {
            eprintln!("Checking accessibility permissions...");
            if !system_integration::has_accessibility_permissions() {
                eprintln!("Accessibility permissions not granted. Requesting permissions...");
                match system_integration::request_accessibility_permissions() {
                    Ok(_) => {
                        if system_integration::has_accessibility_permissions() {
                            eprintln!("Accessibility permissions granted successfully!");
                        } else {
                            eprintln!("Accessibility permissions were not granted. The app will work with limited functionality.");
                            eprintln!("To enable full functionality, please:");
                            eprintln!("1. Go to System Preferences > Security & Privacy > Privacy > Accessibility");
                            eprintln!("2. Enable VKey in the list");
                            eprintln!("3. Restart the application");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to request accessibility permissions: {}", e);
                        eprintln!("The app will work with limited functionality.");
                    }
                }
            } else {
                eprintln!("Accessibility permissions already granted!");
            }
        }

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
                    cx.new(|_| {
                        let mut app = VKeyApp::new();
                        
                        // Mark permissions as checked since we did it in main
                        app.set_permissions_checked(true);
                        
                        eprintln!("VKeyApp initialized successfully");
                        app
                    })
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