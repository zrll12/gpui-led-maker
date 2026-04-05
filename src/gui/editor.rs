mod rect_component_editor;
mod text_component_editor;

use crate::gui::Render;
use crate::modal::app_state::{AppState, LiveProject};
use crate::modal::project::{ComponentLayer, LedMakerProject, PositionedLayer, RectangleComponent, TextComponent};
use gpui::{AnyElement, AppContext, BorrowAppContext, Context, Entity, IntoElement, ParentElement, Styled, Subscription, Window, div};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme, h_flex, v_flex};
use simple_gpui::component;
use crate::gui::editor::rect_component_editor::RectPropertyEditor;
use crate::gui::editor::text_component_editor::TextPropertyEditor;

fn sync_live_project(cx: &mut impl BorrowAppContext, project: &LedMakerProject) {
    cx.update_global::<LiveProject, _>(|lp, _| lp.0 = project.clone());
}

fn sync_selected_layer_editors(view: &mut Editor, window: &mut Window, cx: &mut Context<Editor>) {
    let (x, y) = view
        .selected_layer
        .and_then(|idx| {
            view.project
                .frames
                .first()
                .and_then(|f| f.contents.get(idx))
                .map(|layer| (layer.x, layer.y))
        })
        .unwrap_or((0, 0));

    view.layer_x_input
        .update(cx, |s, cx| s.set_value(x.to_string(), window, cx));
    view.layer_y_input
        .update(cx, |s, cx| s.set_value(y.to_string(), window, cx));

    view.text_property_editor
        .update(cx, |editor, _| editor.set_selected_layer(view.selected_layer));
    view.rect_property_editor
        .update(cx, |editor, _| editor.set_selected_layer(view.selected_layer));
}

#[component]
pub fn editor(window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    init_with_context!();
    component_property!(project_original: LedMakerProject);
    component_property!(project: LedMakerProject = project_original.clone());
    // 当前选中图层在 frames[0].contents 中的索引
    component_property!(selected_layer: Option<usize> = None);

    // 项目名称输入
    component_entity!(project_name_input: InputState = {
        let mut input = InputState::new(window, cx).placeholder("Unnamed Project");
        input.set_value(project_original.name.clone(), window, cx);
        input
    });

    // 帧尺寸输入
    component_entity!(frame_width_input: InputState = {
        let w = project_original.frames.first().map(|f| f.width).unwrap_or(32);
        let mut s = InputState::new(window, cx).placeholder("宽度");
        s.set_value(w.to_string(), window, cx); s
    });
    component_entity!(frame_height_input: InputState = {
        let h = project_original.frames.first().map(|f| f.height).unwrap_or(8);
        let mut s = InputState::new(window, cx).placeholder("高度");
        s.set_value(h.to_string(), window, cx); s
    });

    // 图层位置输入
    component_entity!(layer_x_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("X");
        s.set_value("0".to_string(), window, cx); s
    });
    component_entity!(layer_y_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("Y");
        s.set_value("0".to_string(), window, cx); s
    });

    component_entity!(text_property_editor: TextPropertyEditor = TextPropertyEditor::new(cx, window));
    component_entity!(rect_property_editor: RectPropertyEditor = RectPropertyEditor::new(cx, window));

    // 监听 AppState 变化（外部打开新项目时同步）
    observe!(AppState, |page, window, cx| {
        let new_project = cx.global::<AppState>().current_project.clone();
        if new_project == page.project {
            return;
        }
        page.project = new_project.clone();
        page.selected_layer = None;
        let name = page.project.name.clone();
        page.project_name_input
            .update(cx, |input, cx| input.set_value(name, window, cx));
        let w = page.project.frames.first().map(|f| f.width).unwrap_or(32);
        let h = page.project.frames.first().map(|f| f.height).unwrap_or(8);
        page.frame_width_input.update(cx, |s, cx| s.set_value(w.to_string(), window, cx));
        page.frame_height_input.update(cx, |s, cx| s.set_value(h.to_string(), window, cx));
        cx.update_global::<LiveProject, _>(|lp, _| lp.0 = new_project);
        cx.notify();
    });

    observe!(LiveProject, |page, _window, cx| {
        let live_project = cx.global::<LiveProject>().0.clone();
        if live_project != page.project {
            page.project = live_project;
            cx.notify();
        }
    });

    // ---- 项目名称 ----
    subscribe!(
        project_name_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let value = project_name_input.read(cx).value().to_string();
                view.project.name = value;
                sync_live_project(cx, &view.project);
                cx.notify();
            }
            _ => {}
        }
    );

    // ---- 帧尺寸 ----
    subscribe!(
        frame_width_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u32 = frame_width_input.read(cx).value().parse().unwrap_or(32);
                if view.project.frames.is_empty() {
                    view.project.frames.push(Default::default());
                }
                if let Some(frame) = view.project.frames.first_mut() {
                    frame.width = v;
                    sync_live_project(cx, &view.project);
                }
                cx.notify();
            }
            _ => {}
        }
    );
    subscribe!(
        frame_height_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u32 = frame_height_input.read(cx).value().parse().unwrap_or(8);
                if view.project.frames.is_empty() {
                    view.project.frames.push(Default::default());
                }
                if let Some(frame) = view.project.frames.first_mut() {
                    frame.height = v;
                    sync_live_project(cx, &view.project);
                }
                cx.notify();
            }
            _ => {}
        }
    );

    // ---- 图层位置 ----
    subscribe!(
        layer_x_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: i32 = layer_x_input.read(cx).value().parse().unwrap_or(0);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(layer) = frame.contents.get_mut(idx) {
                            layer.x = v;
                            sync_live_project(cx, &view.project);
                        }
                    }
                }
                cx.notify();
            }
            _ => {}
        }
    );
    subscribe!(
        layer_y_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: i32 = layer_y_input.read(cx).value().parse().unwrap_or(0);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(layer) = frame.contents.get_mut(idx) {
                            layer.y = v;
                            sync_live_project(cx, &view.project);
                        }
                    }
                }
                cx.notify();
            }
            _ => {}
        }
    );

    // ---- 构建图层列表 ----
    let layers = self
        .project
        .frames
        .first()
        .map(|f| f.contents.clone())
        .unwrap_or_default();

    let layer_items: Vec<AnyElement> = layers
        .iter()
        .enumerate()
        .map(|(idx, layer)| {
            let label = match &layer.content {
                ComponentLayer::Text(t) => {
                    if t.text.is_empty() {
                        format!("文字图层 {}", idx + 1)
                    } else {
                        format!("T: {}", t.text.chars().take(12).collect::<String>())
                    }
                }
                ComponentLayer::Rectangle(r) => format!("矩形 {}×{}", r.width, r.height),
            };
            let is_selected = self.selected_layer == Some(idx);

            let row_base = h_flex().gap_1().px_2().py_1().rounded_md();
            let row_base = if is_selected {
                row_base
                    .bg(cx.theme().accent)
                    .text_color(cx.theme().accent_foreground)
            } else {
                row_base
            };

            row_base
                .child(
                    div().flex_grow().child(
                        Button::new(("layer-select", idx))
                            .label(label)
                            .ghost()
                            .on_click(cx.listener(move |view, _, window, cx| {
                                view.selected_layer = Some(idx);
                                sync_selected_layer_editors(view, window, cx);
                                cx.notify();
                            })),
                    ),
                )
                .child(
                    Button::new(("layer-up", idx))
                        .label("↑")
                        .ghost()
                        .on_click(cx.listener(move |view, _, window, cx| {
                            if let Some(frame) = view.project.frames.first_mut() {
                                if idx > 0 && idx < frame.contents.len() {
                                    frame.contents.swap(idx, idx - 1);
                                    match view.selected_layer {
                                        Some(sel) if sel == idx => view.selected_layer = Some(idx - 1),
                                        Some(sel) if sel == idx - 1 => view.selected_layer = Some(idx),
                                        _ => {}
                                    }
                                }
                            }
                            sync_selected_layer_editors(view, window, cx);
                            sync_live_project(cx, &view.project);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new(("layer-down", idx))
                        .label("↓")
                        .ghost()
                        .on_click(cx.listener(move |view, _, window, cx| {
                            if let Some(frame) = view.project.frames.first_mut() {
                                if idx + 1 < frame.contents.len() {
                                    frame.contents.swap(idx, idx + 1);
                                    match view.selected_layer {
                                        Some(sel) if sel == idx => view.selected_layer = Some(idx + 1),
                                        Some(sel) if sel == idx + 1 => view.selected_layer = Some(idx),
                                        _ => {}
                                    }
                                }
                            }
                            sync_selected_layer_editors(view, window, cx);
                            sync_live_project(cx, &view.project);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new(("layer-remove", idx))
                        .label("✕")
                        .ghost()
                        .on_click(cx.listener(move |view, _, window, cx| {
                            if let Some(frame) = view.project.frames.first_mut() {
                                if idx < frame.contents.len() {
                                    frame.contents.remove(idx);
                                }
                            }
                            if view.selected_layer == Some(idx) {
                                view.selected_layer = None;
                            } else if let Some(sel) = view.selected_layer {
                                if sel > idx {
                                    view.selected_layer = Some(sel - 1);
                                }
                            }
                            sync_selected_layer_editors(view, window, cx);
                            sync_live_project(cx, &view.project);
                            cx.notify();
                        })),
                )
                .into_any_element()
        })
        .collect();

    // ---- 属性面板（根据选中图层类型显示） ----
    let selected_layer_type = self.selected_layer.and_then(|idx| {
        self.project
            .frames
            .first()
            .and_then(|f| f.contents.get(idx))
            .map(|l| match &l.content {
                ComponentLayer::Text(_) => "text",
                ComponentLayer::Rectangle(_) => "rect",
            })
    });

    // ---- 位置行（所有图层类型通用） ----
    let position_row: AnyElement = if selected_layer_type.is_some() {
        h_flex().gap_2().items_center()
            .child(div().w_16().child("位置 X:"))
            .child(Input::new(&self.layer_x_input).w_20())
            .child(div().w_4().child("Y:"))
            .child(Input::new(&self.layer_y_input).w_20())
            .into_any_element()
    } else {
        div().into_any_element()
    };

    self.text_property_editor
        .update(cx, |editor, _| editor.set_selected_layer(self.selected_layer));
    self.rect_property_editor
        .update(cx, |editor, _| editor.set_selected_layer(self.selected_layer));

    let property_panel: AnyElement = match selected_layer_type {
        Some("text") => v_flex()
            .gap_2()
            .px_3()
            .child(position_row)
            .child(self.text_property_editor.clone())
            .into_any_element(),
        Some("rect") => v_flex()
            .gap_2()
            .px_3()
            .child(position_row)
            .child(self.rect_property_editor.clone())
            .into_any_element(),
        _ => div()
            .p_3()
            .text_color(cx.theme().muted_foreground)
            .child("↑ 点击上方的图层以编辑属性")
            .into_any_element(),
    };

    // ---- 整体布局 ----
    v_flex()
        .size_full()
        .gap_2()
        .child(
            h_flex()
                .gap_2()
                .px_3()
                .pt_3()
                .items_center()
                .child("项目名称:")
                .child(Input::new(&self.project_name_input).w_full().flex_grow()),
        )
        .child(
            h_flex()
                .gap_2()
                .px_3()
                .items_center()
                .child(div().w_16().child("画布:"))
                .child(Input::new(&self.frame_width_input).w_16())
                .child(div().child("×"))
                .child(Input::new(&self.frame_height_input).w_16())
                .child(div().text_sm().text_color(cx.theme().muted_foreground).child("(宽 × 高 像素)")),
        )
        .child(
            h_flex()
                .gap_2()
                .px_3()
                .items_center()
                .child(div().flex_grow().text_sm().text_color(cx.theme().muted_foreground).child("图层（第一帧）"))
                .child(
                    Button::new("add-text-layer")
                        .label("+ 文字")
                            .on_click(cx.listener(|view, _, window, cx| {
                            let default_font = cx
                                .global::<AppState>()
                                .config
                                .font_list
                                .first()
                                .map(|f| {
                                    if f.name.is_empty() {
                                        f.path.to_string_lossy().to_string()
                                    } else {
                                        f.name.clone()
                                    }
                                })
                                .unwrap_or_default();
                            if view.project.frames.is_empty() {
                                view.project.frames.push(Default::default());
                            }
                            if let Some(frame) = view.project.frames.first_mut() {
                                frame.contents.push(PositionedLayer {
                                    x: 0,
                                    y: 0,
                                    content: ComponentLayer::Text(TextComponent {
                                        text: String::new(),
                                        font: default_font,
                                        color: (255, 80, 80),
                                        effects: Vec::new(),
                                    }),
                                });
                                view.selected_layer = Some(frame.contents.len() - 1);
                            }
                                sync_selected_layer_editors(view, window, cx);
                            sync_live_project(cx, &view.project);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("add-rect-layer")
                        .label("+ 矩形")
                        .on_click(cx.listener(|view, _, window, cx| {
                            if view.project.frames.is_empty() {
                                view.project.frames.push(Default::default());
                            }
                            if let Some(frame) = view.project.frames.first_mut() {
                                frame.contents.push(PositionedLayer {
                                    x: 0,
                                    y: 0,
                                    content: ComponentLayer::Rectangle(RectangleComponent {
                                        width: 16,
                                        height: 16,
                                        radius: 0,
                                        color: (255, 255, 255),
                                    }),
                                });
                                view.selected_layer = Some(frame.contents.len() - 1);
                            }
                            sync_selected_layer_editors(view, window, cx);
                            sync_live_project(cx, &view.project);
                            cx.notify();
                        })),
                ),
        )
        .child(
            div()
                .px_3()
                .min_h_16()
                .max_h_48()
                .overflow_y_scrollbar()
                .child(
                    v_flex().gap_1().children(if layer_items.is_empty() {
                        vec![div()
                            .p_2()
                            .text_color(cx.theme().muted_foreground)
                            .text_sm()
                            .child("暂无图层，点击上方按钮添加")
                            .into_any_element()]
                    } else {
                        layer_items
                    }),
                ),
        )
        .child(div().h_px().bg(cx.theme().border).mx_3())
        .child(property_panel)
        .child(div().flex_grow())
        .child(
            h_flex().w_full().justify_center().pb_3().child(
                Button::new("editor-save")
                    .label("保存")
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.project.save_project(cx);
                    }))
                    .px_6()
                    .py_2(),
            ),
        )
}

