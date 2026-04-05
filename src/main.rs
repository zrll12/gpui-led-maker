#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod render;
mod gui;
mod modal;

use gpui::{AppContext, Application, TitlebarOptions, WindowOptions};
use crate::gui::MainPage;
use crate::gui::menu::set_app_menus;
use crate::modal::app_state::{AppState, LiveProject};

const HEX_FILE: &str = "test-data/input.hex";
const OUTPUT_FILE: &str = "led_output.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {

    Application::new().run(move |cx| {
        gpui_component::init(cx);
        cx.set_global(AppState::new());
        cx.set_global(LiveProject::new());
        cx.on_action(gui::menu::quit);
        cx.on_action(gui::menu::new_file);
        cx.on_action(gui::menu::open_file);
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
        set_app_menus(cx);

        let window_options = WindowOptions::default();
        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                window.set_window_title("LED Maker");
                let view = cx.new(|cx| MainPage::new(cx, window));
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            })
                .unwrap();

            Ok::<_, ()>(())
        }).detach();
    });

    Ok(())
}
