use crate::gui::Render;
use crate::modal::app_state::AppState;
use crate::render::font_preview::{
    read_bdf_font_name, remove_bdf_preview_cache, render_bdf_preview_image,
};
use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, Window, div, img};
use gpui_component::button::Button;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme, h_flex, v_flex};
use rfd::AsyncFileDialog;
use simple_gpui::component;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

#[component]
pub fn fonts(_window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let fonts = cx.global::<AppState>().config.font_list.clone();

    let font_list: AnyElement = if fonts.is_empty() {
        div().p_3().child("No fonts").into_any_element()
    } else {
        v_flex()
            .gap_3()
            .children(fonts.into_iter().map(|item| {
                let font_path = item.path.clone();
                let title = if item.name.is_empty() {
                    item.path.display().to_string()
                } else {
                    item.name
                };

                let preview: AnyElement = match render_bdf_preview_image(&item.path) {
                    Some(preview_path) => img(preview_path)
                        .h_10()
                        .w_16()
                        .rounded_md()
                        .into_any_element(),
                    None => div().child("(A preview unavailable)").into_any_element(),
                };

                v_flex()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_md()
                    .p_2()
                    .gap_1()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(title)
                            .child(
                                Button::new(("font-remove", font_remove_button_key(&font_path)))
                                    .label("Remove")
                                    .on_click(cx.listener(move |_, _, _, cx| {
                                        remove_font(cx, font_path.clone());
                                    })),
                            ),
                    )
                    .child(div().p_1().child(preview))
                    .into_any_element()
            }))
            .into_any_element()
    };

    v_flex()
        .size_full()
        .p_2()
        .gap_2()
        .child(
            h_flex().justify_end().child(
                Button::new("font-add-bdf")
                    .label("Add BDF Font")
                    .on_click(cx.listener(|_, _, _, cx| {
                        add_bdf_font(cx);
                    })),
            ),
        )
        .child(div().flex_1().overflow_y_scrollbar().child(font_list))
}

fn add_bdf_font<T: 'static>(cx: &mut Context<T>) {
    cx.spawn(move |_: gpui::WeakEntity<T>, cx: &mut gpui::AsyncApp| {
        let cx = cx.clone();
        async move {
            let file = AsyncFileDialog::new()
                .add_filter("BDF font", &["bdf"])
                .add_filter("all files", &["*"])
                .pick_file()
                .await;

            let Some(file) = file else {
                return;
            };

            let path = file.path().to_path_buf();
            let name = read_bdf_font_name(&path).unwrap_or_else(|| {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unnamed BDF Font".to_string())
            });

            let path_for_state = path.clone();
            let name_for_state = name.clone();
            let _ = cx.update_global::<AppState, _>(|app_state, _| {
                app_state.config.add_font(path_for_state, name_for_state);
                if let Err(err) = app_state.config.save() {
                    println!("Error saving config: {}", err);
                }
            });

            let _ = cx.refresh();
        }
    })
    .detach();
}

fn remove_font<T: 'static>(cx: &mut Context<T>, path: std::path::PathBuf) {
    let app_state = cx.global_mut::<AppState>();
    app_state.config.remove_font(&path);
    if let Err(err) = app_state.config.save() {
        println!("Error saving config: {}", err);
    }

    remove_bdf_preview_cache(&path);

    cx.notify();
}

fn font_remove_button_key(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}
