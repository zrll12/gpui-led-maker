use crate::gui::Render;
use crate::modal::app_state::{AppState, LiveProject};
use crate::modal::project::ComponentLayer;
use gpui::{AnyElement, AppContext, BorrowAppContext, Context, Entity, Hsla, IntoElement, ParentElement, Rgba, Styled, Subscription, Window, div};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState};
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
pub fn text_property_editor(window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    init_with_context!();
    component_property!(selected_layer: Option<usize> = None);
    component_property!(last_selected_layer: Option<usize> = None);

    component_entity!(text_content_input: InputState = InputState::new(window, cx).placeholder("输入文字"));
    component_entity!(text_font_input: InputState = InputState::new(window, cx).placeholder("字体文件路径 (.bdf)"));
    component_entity!(text_color_picker: ColorPickerState = ColorPickerState::new(window, cx).default_value(cx.theme().primary));

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
                        ComponentLayer::Text(t) => Some((
                            t.text.clone(),
                            t.font.clone(),
                            t.color.0,
                            t.color.1,
                            t.color.2,
                        )),
                        _ => None,
                    })
            })
            .unwrap_or_else(|| (String::new(), String::new(), 255, 80, 80));

        self.text_content_input.update(cx, |s, cx| s.set_value(values.0, window, cx));
        self.text_font_input.update(cx, |s, cx| s.set_value(values.1, window, cx));
        self.text_color_picker.update(cx, |picker, cx| {
            picker.set_value(rgb_u8_to_hsla((values.2, values.3, values.4)), window, cx);
        });
    }

    subscribe!(
        text_content_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let value = text_content_input.read(cx).value().to_string();
                if let Some(idx) = view.selected_layer {
                    cx.update_global::<LiveProject, _>(|lp, _| {
                        if let Some(frame) = lp.0.frames.first_mut() {
                            if let Some(ComponentLayer::Text(t)) =
                                frame.contents.get_mut(idx).map(|l| &mut l.content)
                            {
                                t.text = value;
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
        text_font_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let value = text_font_input.read(cx).value().to_string();
                if let Some(idx) = view.selected_layer {
                    cx.update_global::<LiveProject, _>(|lp, _| {
                        if let Some(frame) = lp.0.frames.first_mut() {
                            if let Some(ComponentLayer::Text(t)) =
                                frame.contents.get_mut(idx).map(|l| &mut l.content)
                            {
                                t.font = value;
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
        text_color_picker,
        |view, _state, event, _window, cx| {
            if let ColorPickerEvent::Change(Some(color)) = event {
                if let Some(idx) = view.selected_layer {
                    let (r, g, b) = hsla_to_rgb_u8(*color);
                    cx.update_global::<LiveProject, _>(|lp, _| {
                        if let Some(frame) = lp.0.frames.first_mut() {
                            if let Some(ComponentLayer::Text(t)) =
                                frame.contents.get_mut(idx).map(|l| &mut l.content)
                            {
                                t.color = (r, g, b);
                            }
                        }
                    });
                }
                cx.notify();
            }
        }
    );

    let font_hint: AnyElement = {
        let fonts = cx.global::<AppState>().config.font_list.clone();
        if !fonts.is_empty() {
            let items: Vec<AnyElement> = fonts
                .iter()
                .enumerate()
                .map(|(fidx, np)| {
                    let path_str = np.path.to_string_lossy().to_string();
                    let label = if np.name.is_empty() {
                        path_str.clone()
                    } else {
                        np.name.clone()
                    };
                    let font_entity = self.text_font_input.clone();
                    Button::new(("font-pick", fidx))
                        .label(label)
                        .ghost()
                        .on_click(cx.listener(move |_view, _, window, cx| {
                            font_entity.update(cx, |s, cx| s.set_value(path_str.clone(), window, cx));
                        }))
                        .into_any_element()
                })
                .collect();
            v_flex()
                .gap_1()
                .child(div().text_sm().text_color(cx.theme().muted_foreground).child("可用字体："))
                .children(items)
                .into_any_element()
        } else {
            div().into_any_element()
        }
    };

    v_flex()
        .gap_2()
        .p_3()
        .child(div().text_sm().text_color(cx.theme().muted_foreground).child("文字图层属性"))
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("文字:"))
                .child(Input::new(&self.text_content_input).w_full().flex_grow()),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().w_16().child("字体:"))
                .child(Input::new(&self.text_font_input).w_full().flex_grow()),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
            .child(div().w_16().child("颜色:"))
            .child(ColorPicker::new(&self.text_color_picker)),
        )
        .child(font_hint)
        .into_any_element()
}

impl TextPropertyEditor {
    pub fn set_selected_layer(&mut self, selected_layer: Option<usize>) {
        self.selected_layer = selected_layer;
    }
}
