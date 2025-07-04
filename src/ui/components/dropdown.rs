use gpui::{
    div, prelude::*, rgb, Context, IntoElement, Render, Styled, Window
};

use gpui_component::{
    dropdown::{Dropdown, DropdownState, DropdownItem},
};

/// Simple dropdown item that wraps a string
#[derive(Clone, Debug)]
pub struct SimpleDropdownItem {
    value: String,
}

impl SimpleDropdownItem {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl DropdownItem for SimpleDropdownItem {
    type Value = String;

    fn title(&self) -> gpui::SharedString {
        self.value.clone().into()
    }

    fn display_title(&self) -> Option<gpui::AnyElement> {
        None // Use default rendering
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

/// Utility function to create a dropdown for VKey app with proper styling
/// This matches the style used in VKeyApp's render_dropdown method
pub fn create_vkey_dropdown(
    label: &str,
    options: &[&str],
    selected_index: usize,
    window: &mut Window,
    cx: &mut Context<impl Render>,
) -> impl IntoElement {
    let label = label.to_string();
    
    // Convert options to our dropdown items
    let dropdown_options: Vec<SimpleDropdownItem> = options.iter()
        .map(|&s| SimpleDropdownItem::new(s))
        .collect();
    
    // Create dropdown state
    let dropdown_state = cx.new(|cx| DropdownState::new(dropdown_options, Some(selected_index), window, cx));
    
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_color(rgb(0xe2e8f0))
                .text_sm()
                .w_16()
                .child(label)
        )
        .child(
            Dropdown::new(&dropdown_state)
                .cleanable()
                .placeholder("Select...")
        )
}
