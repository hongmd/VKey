// Platform-specific functionality
// This module contains platform-specific implementations
// for Vietnamese input method integration

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "macos")]
pub use macos::*; 