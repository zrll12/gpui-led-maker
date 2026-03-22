use gpui::{AnyElement, Context, Entity, IntoElement, ParentElement, Styled, div};
use gpui_component::input::{Input, InputState};
use gpui_component::{ActiveTheme, h_flex, v_flex};

pub fn build_text_property_editor<T>(
    cx: &mut Context<T>,
    position_row: AnyElement,
    text_content_input: &Entity<InputState>,
    text_font_input: &Entity<InputState>,
    text_color_r_input: &Entity<InputState>,
    text_color_g_input: &Entity<InputState>,
    text_color_b_input: &Entity<InputState>,
) -> AnyElement {
    v_flex()
        .gap_2()
        .p_3()
        .child(div().text_sm().text_color(cx.theme().muted_foreground).child("文字图层属性"))
        .child(position_row)
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("文字:"))
                .child(Input::new(text_content_input).w_full().flex_grow()),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("字体:"))
                .child(Input::new(text_font_input).w_full().flex_grow()),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("颜色 RGB:"))
                .child(Input::new(text_color_r_input).w_16())
                .child(Input::new(text_color_g_input).w_16())
                .child(Input::new(text_color_b_input).w_16()),
        )
        .into_any_element()
}
