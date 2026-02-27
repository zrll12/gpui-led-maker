mod render;
mod gui;
mod modal;

use gpui::{AppContext, Application, WindowOptions};
use crate::gui::MainPage;
use crate::gui::menu::set_app_menus;
use crate::modal::app_state::AppState;

const HEX_FILE: &str = "test-data/input.hex";
const OUTPUT_FILE: &str = "led_output.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let glyphs = load_unifont_hex(HEX_FILE)?;
    // let matrix = text_to_matrix("Some Texts😀", &glyphs);
    //
    // let options = LedRenderOptions {
    //     led_size: 14,
    //     spacing: 4,
    //     ..Default::default()
    // };
    //
    // let img = render_led_matrices(&matrix, &options);
    // img.save(OUTPUT_FILE)?;
    //
    // println!("Saved to {OUTPUT_FILE}");

    Application::new().run(move |cx| {
        gpui_component::init(cx);
        cx.set_global(AppState::new());
        cx.on_action(gui::menu::quit);
        cx.on_action(gui::menu::new_file);
        cx.on_action(gui::menu::open_file);
        cx.activate(true);
        set_app_menus(cx);

        let window_options = WindowOptions::default();
        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| MainPage::new(cx, window));
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            })
                .unwrap();

            Ok::<_, ()>(())
        }).detach();
    });

    Ok(())
}
