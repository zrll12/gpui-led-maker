pub mod menu;

use crate::modal::app_state::AppState;
use crate::modal::project::LedMakerProject;
use gpui::IntoElement;
use gpui::{
    actions, App, Context, ParentElement, Render, Styled, Window,
};
use gpui_component::v_flex;
use simple_gpui::component;

#[component]
pub fn main_page(_window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    v_flex()
        .p_5()
        .gap_2()
        .size_full()
        .items_center()
        .justify_center()
        .child("Hello, GPUI!")
}

actions!(set_menus, [Quit, NewFile, OpenFile]);
pub fn quit(_: &Quit, cx: &mut App) {
    println!("Quitting...");
    cx.quit();
}

pub fn new_file(_: &NewFile, cx: &mut App) {
    println!("Creating new file...");
    cx.global_mut::<AppState>().file_path = None;
    cx.global_mut::<AppState>().current_project = LedMakerProject::default();
}

pub fn open_file(_: &OpenFile, cx: &mut App) {
    println!("Toggling check...");
    // Here you can implement file selection logic and update the global state if needed
}
