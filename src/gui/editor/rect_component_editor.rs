use crate::gui::Render;
use crate::modal::app_state::LiveProject;
use crate::modal::project::ComponentLayer;
use gpui::{AppContext, BorrowAppContext, Context, Entity, Hsla, IntoElement, ParentElement, Rgba, Styled, Subscription, Window, div};
use gpui_component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{ActiveTheme, h_flex, v_flex};
use simple_gpui::component;

fn rgb_u8_to_hsla((r, g, b): (u8, u8, u8)) -> Hsla {
    Hsla::from(Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    })
}

fn hsla_to_rgb_u8(color: Hsla) -> (u8, u8, u8) {
    let rgb = color.to_rgb();
    (
        (rgb.r * 255.0).round().clamp(0.0, 255.0) as u8,
        (rgb.g * 255.0).round().clamp(0.0, 255.0) as u8,
        (rgb.b * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

#[component]
pub fn rect_property_editor(window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    init_with_context!();
    component_property!(selected_layer: Option<usize> = None);
    component_property!(last_selected_layer: Option<usize> = None);

    component_entity!(rect_width_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("宽度");
        s.set_value("16".to_string(), window, cx); s
    });
    component_entity!(rect_height_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("高度");
        s.set_value("16".to_string(), window, cx); s
    });
    component_entity!(rect_radius_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("圆角");
        s.set_value("0".to_string(), window, cx); s
    });
    component_entity!(rect_color_picker: ColorPickerState = ColorPickerState::new(window, cx).default_value(cx.theme().primary));

    if self.last_selected_layer != self.selected_layer {
        self.last_selected_layer = self.selected_layer;
        let values = self
            .selected_layer
            .and_then(|idx| {
                cx.global::<LiveProject>()
                    .0
                    .frames
                    .first()
                    .and_then(|f| f.contents.get(idx))
                    .and_then(|layer| match &layer.content {
                        ComponentLayer::Rectangle(r) => Some((
                            r.width,
                            r.height,
                            r.radius,
                            r.color,
                        )),
                        _ => None,
                    })
            })
            .unwrap_or((16, 16, 0, (255, 255, 255)));

        self.rect_width_input.update(cx, |s, cx| s.set_value(values.0.to_string(), window, cx));
        self.rect_height_input.update(cx, |s, cx| s.set_value(values.1.to_string(), window, cx));
        self.rect_radius_input.update(cx, |s, cx| s.set_value(values.2.to_string(), window, cx));
        self.rect_color_picker.update(cx, |picker, cx| {
            picker.set_value(rgb_u8_to_hsla(values.3), window, cx);
        });
    }

    subscribe!(
        rect_width_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u32 = rect_width_input.read(cx).value().parse().unwrap_or(0);
                if let Some(idx) = view.selected_layer {
                    cx.update_global::<LiveProject, _>(|lp, _| {
                        if let Some(frame) = lp.0.frames.first_mut() {
                            if let Some(ComponentLayer::Rectangle(r)) =
                                frame.contents.get_mut(idx).map(|l| &mut l.content)
                            {
                                r.width = v;
                            }
                        }
                    });
                }
                cx.notify();
            }
            _ => {}
        }
    );

    subscribe!(
        rect_height_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u32 = rect_height_input.read(cx).value().parse().unwrap_or(0);
                if let Some(idx) = view.selected_layer {
                    cx.update_global::<LiveProject, _>(|lp, _| {
                        if let Some(frame) = lp.0.frames.first_mut() {
                            if let Some(ComponentLayer::Rectangle(r)) =
                                frame.contents.get_mut(idx).map(|l| &mut l.content)
                            {
                                r.height = v;
                            }
                        }
                    });
                }
                cx.notify();
            }
            _ => {}
        }
    );

    subscribe!(
        rect_radius_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u32 = rect_radius_input.read(cx).value().parse().unwrap_or(0);
                if let Some(idx) = view.selected_layer {
                    cx.update_global::<LiveProject, _>(|lp, _| {
                        if let Some(frame) = lp.0.frames.first_mut() {
                            if let Some(ComponentLayer::Rectangle(r)) =
                                frame.contents.get_mut(idx).map(|l| &mut l.content)
                            {
                                r.radius = v;
                            }
                        }
                    });
                }
                cx.notify();
            }
            _ => {}
        }
    );

    subscribe!(
        rect_color_picker,
        |view, _state, event, _window, cx| {
            if let ColorPickerEvent::Change(Some(color)) = event {
                if let Some(idx) = view.selected_layer {
                    let rgb = hsla_to_rgb_u8(*color);
                    cx.update_global::<LiveProject, _>(|lp, _| {
                        if let Some(frame) = lp.0.frames.first_mut() {
                            if let Some(ComponentLayer::Rectangle(r)) =
                                frame.contents.get_mut(idx).map(|l| &mut l.content)
                            {
                                r.color = rgb;
                            }
                        }
                    });
                }
                cx.notify();
            }
        }
    );

    v_flex()
        .gap_2()
        .p_3()
        .child(div().text_sm().text_color(cx.theme().muted_foreground).child("矩形图层属性"))
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("宽度:"))
                .child(Input::new(&self.rect_width_input).w_24()),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("高度:"))
                .child(Input::new(&self.rect_height_input).w_24()),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("圆角:"))
                .child(Input::new(&self.rect_radius_input).w_24()),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
            .child(div().w_16().child("颜色:"))
            .child(ColorPicker::new(&self.rect_color_picker)),
        )
        .into_any_element()
}

impl RectPropertyEditor {
    pub fn set_selected_layer(&mut self, selected_layer: Option<usize>) {
        self.selected_layer = selected_layer;
    }
}
