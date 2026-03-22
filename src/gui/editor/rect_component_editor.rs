use gpui::{AnyElement, Context, Entity, IntoElement, ParentElement, Styled, div};
use gpui_component::input::{Input, InputState};
use gpui_component::{ActiveTheme, h_flex, v_flex};

pub fn build_rect_property_editor<T>(
    cx: &mut Context<T>,
    position_row: AnyElement,
    rect_width_input: &Entity<InputState>,
    rect_height_input: &Entity<InputState>,
    rect_radius_input: &Entity<InputState>,
    rect_color_r_input: &Entity<InputState>,
    rect_color_g_input: &Entity<InputState>,
    rect_color_b_input: &Entity<InputState>,
) -> AnyElement {
    v_flex()
        .gap_2()
        .p_3()
        .child(div().text_sm().text_color(cx.theme().muted_foreground).child("矩形图层属性"))
        .child(position_row)
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("宽度:"))
                .child(Input::new(rect_width_input).w_24()),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("高度:"))
                .child(Input::new(rect_height_input).w_24()),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("圆角:"))
                .child(Input::new(rect_radius_input).w_24()),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("颜色 RGB:"))
                .child(Input::new(rect_color_r_input).w_16())
                .child(Input::new(rect_color_g_input).w_16())
                .child(Input::new(rect_color_b_input).w_16()),
        )
        .into_any_element()
}
