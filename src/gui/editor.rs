mod rect_component_editor;
mod text_component_editor;

use crate::gui::Render;
use crate::modal::app_state::{AppState, LiveProject};
use crate::modal::project::{ComponentLayer, LedMakerProject, PositionedLayer, RectangleComponent, TextComponent};
use gpui::{AnyElement, AppContext, BorrowAppContext, Context, Entity, IntoElement, ParentElement, Styled, Subscription, Window, div};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{ActiveTheme, h_flex, v_flex};
use simple_gpui::component;
use crate::gui::editor::rect_component_editor::build_rect_property_editor;
use crate::gui::editor::text_component_editor::build_text_property_editor;

fn sync_live_project(cx: &mut impl BorrowAppContext, project: &LedMakerProject) {
    cx.update_global::<LiveProject, _>(|lp, _| lp.0 = project.clone());
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

    // 文字图层属性输入
    component_entity!(text_content_input: InputState = InputState::new(window, cx).placeholder("输入文字"));
    component_entity!(text_font_input: InputState = InputState::new(window, cx).placeholder("字体文件路径 (.bdf)"));
    component_entity!(text_color_r_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("255");
        s.set_value("255".to_string(), window, cx); s
    });
    component_entity!(text_color_g_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("80");
        s.set_value("80".to_string(), window, cx); s
    });
    component_entity!(text_color_b_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("80");
        s.set_value("80".to_string(), window, cx); s
    });

    // 矩形图层属性输入
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
    component_entity!(rect_color_r_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("255");
        s.set_value("255".to_string(), window, cx); s
    });
    component_entity!(rect_color_g_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("255");
        s.set_value("255".to_string(), window, cx); s
    });
    component_entity!(rect_color_b_input: InputState = {
        let mut s = InputState::new(window, cx).placeholder("255");
        s.set_value("255".to_string(), window, cx); s
    });

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

    // ---- 文字图层输入订阅 ----
    subscribe!(
        text_content_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let value = text_content_input.read(cx).value().to_string();
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Text(t)) = frame.contents.get_mut(idx).map(|l| &mut l.content) {
                            t.text = value;
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
        text_font_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let value = text_font_input.read(cx).value().to_string();
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Text(t)) = frame.contents.get_mut(idx).map(|l| &mut l.content) {
                            t.font = value;
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
        text_color_r_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u8 = text_color_r_input.read(cx).value().parse().unwrap_or(255);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Text(t)) = frame.contents.get_mut(idx).map(|l| &mut l.content) {
                            t.color.0 = v;
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
        text_color_g_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u8 = text_color_g_input.read(cx).value().parse().unwrap_or(80);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Text(t)) = frame.contents.get_mut(idx).map(|l| &mut l.content) {
                            t.color.1 = v;
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
        text_color_b_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u8 = text_color_b_input.read(cx).value().parse().unwrap_or(80);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Text(t)) = frame.contents.get_mut(idx).map(|l| &mut l.content) {
                            t.color.2 = v;
                            sync_live_project(cx, &view.project);
                        }
                    }
                }
                cx.notify();
            }
            _ => {}
        }
    );

    // ---- 矩形图层输入订阅 ----
    subscribe!(
        rect_width_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u32 = rect_width_input.read(cx).value().parse().unwrap_or(0);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Rectangle(r)) =
                            frame.contents.get_mut(idx).map(|l| &mut l.content)
                        {
                            r.width = v;
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
        rect_height_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u32 = rect_height_input.read(cx).value().parse().unwrap_or(0);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Rectangle(r)) =
                            frame.contents.get_mut(idx).map(|l| &mut l.content)
                        {
                            r.height = v;
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
        rect_radius_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u32 = rect_radius_input.read(cx).value().parse().unwrap_or(0);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Rectangle(r)) =
                            frame.contents.get_mut(idx).map(|l| &mut l.content)
                        {
                            r.radius = v;
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
        rect_color_r_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u8 = rect_color_r_input.read(cx).value().parse().unwrap_or(255);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Rectangle(r)) =
                            frame.contents.get_mut(idx).map(|l| &mut l.content)
                        {
                            r.color.0 = v;
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
        rect_color_g_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u8 = rect_color_g_input.read(cx).value().parse().unwrap_or(255);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Rectangle(r)) =
                            frame.contents.get_mut(idx).map(|l| &mut l.content)
                        {
                            r.color.1 = v;
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
        rect_color_b_input,
        |view, _state, event, _window, cx| match event {
            InputEvent::Change => {
                let v: u8 = rect_color_b_input.read(cx).value().parse().unwrap_or(255);
                if let Some(idx) = view.selected_layer {
                    if let Some(frame) = view.project.frames.first_mut() {
                        if let Some(ComponentLayer::Rectangle(r)) =
                            frame.contents.get_mut(idx).map(|l| &mut l.content)
                        {
                            r.color.2 = v;
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
            let text_input = self.text_content_input.clone();
            let font_input = self.text_font_input.clone();
            let tr_input = self.text_color_r_input.clone();
            let tg_input = self.text_color_g_input.clone();
            let tb_input = self.text_color_b_input.clone();
            let rw_input = self.rect_width_input.clone();
            let rh_input = self.rect_height_input.clone();
            let rr_input = self.rect_radius_input.clone();
            let rrr_input = self.rect_color_r_input.clone();
            let rrg_input = self.rect_color_g_input.clone();
            let rrb_input = self.rect_color_b_input.clone();
            let x_input = self.layer_x_input.clone();
            let y_input = self.layer_y_input.clone();

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
                                if let Some(frame) = view.project.frames.first() {
                                    if let Some(layer) = frame.contents.get(idx) {
                                        let (px, py) = (layer.x, layer.y);
                                        x_input.update(cx, |s, cx| s.set_value(px.to_string(), window, cx));
                                        y_input.update(cx, |s, cx| s.set_value(py.to_string(), window, cx));
                                        match &layer.content {
                                            ComponentLayer::Text(t) => {
                                                let txt = t.text.clone();
                                                let fnt = t.font.clone();
                                                let (r, g, b) = t.color;
                                                text_input.update(cx, |s, cx| s.set_value(txt, window, cx));
                                                font_input.update(cx, |s, cx| s.set_value(fnt, window, cx));
                                                tr_input.update(cx, |s, cx| s.set_value(r.to_string(), window, cx));
                                                tg_input.update(cx, |s, cx| s.set_value(g.to_string(), window, cx));
                                                tb_input.update(cx, |s, cx| s.set_value(b.to_string(), window, cx));
                                            }
                                            ComponentLayer::Rectangle(r) => {
                                                let (w, h, rad) = (r.width, r.height, r.radius);
                                                let (cr, cg, cb) = r.color;
                                                rw_input.update(cx, |s, cx| s.set_value(w.to_string(), window, cx));
                                                rh_input.update(cx, |s, cx| s.set_value(h.to_string(), window, cx));
                                                rr_input.update(cx, |s, cx| s.set_value(rad.to_string(), window, cx));
                                                rrr_input.update(cx, |s, cx| s.set_value(cr.to_string(), window, cx));
                                                rrg_input.update(cx, |s, cx| s.set_value(cg.to_string(), window, cx));
                                                rrb_input.update(cx, |s, cx| s.set_value(cb.to_string(), window, cx));
                                            }
                                        }
                                    }
                                }
                                cx.notify();
                            })),
                    ),
                )
                .child(
                    Button::new(("layer-up", idx))
                        .label("↑")
                        .ghost()
                        .on_click(cx.listener(move |view, _, _, cx| {
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
                            sync_live_project(cx, &view.project);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new(("layer-down", idx))
                        .label("↓")
                        .ghost()
                        .on_click(cx.listener(move |view, _, _, cx| {
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
                            sync_live_project(cx, &view.project);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new(("layer-remove", idx))
                        .label("✕")
                        .ghost()
                        .on_click(cx.listener(move |view, _, _, cx| {
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

    let property_panel: AnyElement = match selected_layer_type {
        Some("text") => build_text_property_editor(
            cx,
            position_row,
            &self.text_content_input,
            &self.text_font_input,
            &self.text_color_r_input,
            &self.text_color_g_input,
            &self.text_color_b_input,
        ),
        Some("rect") => build_rect_property_editor(
            cx,
            position_row,
            &self.rect_width_input,
            &self.rect_height_input,
            &self.rect_radius_input,
            &self.rect_color_r_input,
            &self.rect_color_g_input,
            &self.rect_color_b_input,
        ),
        _ => div()
            .p_3()
            .text_color(cx.theme().muted_foreground)
            .child("← 点击左侧图层以编辑属性")
            .into_any_element(),
    };

    // ---- 字体快速选择（仅文字图层） ----
    let font_hint: AnyElement = {
        let fonts = cx.global::<AppState>().config.font_list.clone();
        if !fonts.is_empty() && selected_layer_type == Some("text") {
            let items: Vec<AnyElement> = fonts
                .iter()
                .enumerate()
                .map(|(fidx, np)| {
                    let path_str = np.path.to_string_lossy().to_string();
                    let label = if np.name.is_empty() { path_str.clone() } else { np.name.clone() };
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
                .px_3()
                .child(div().text_sm().text_color(cx.theme().muted_foreground).child("可用字体："))
                .children(items)
                .into_any_element()
        } else {
            div().into_any_element()
        }
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
                        .on_click(cx.listener(|view, _, _, cx| {
                            if view.project.frames.is_empty() {
                                view.project.frames.push(Default::default());
                            }
                            if let Some(frame) = view.project.frames.first_mut() {
                                frame.contents.push(PositionedLayer {
                                    x: 0,
                                    y: 0,
                                    content: ComponentLayer::Text(TextComponent {
                                        text: String::new(),
                                        font: String::new(),
                                        color: (255, 80, 80),
                                    }),
                                });
                                view.selected_layer = Some(frame.contents.len() - 1);
                            }
                            sync_live_project(cx, &view.project);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("add-rect-layer")
                        .label("+ 矩形")
                        .on_click(cx.listener(|view, _, _, cx| {
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
                            sync_live_project(cx, &view.project);
                            cx.notify();
                        })),
                ),
        )
        .child(
            v_flex()
                .gap_1()
                .px_3()
                .min_h_16()
                .max_h_48()
                .children(if layer_items.is_empty() {
                    vec![div()
                        .p_2()
                        .text_color(cx.theme().muted_foreground)
                        .text_sm()
                        .child("暂无图层，点击上方按钮添加")
                        .into_any_element()]
                } else {
                    layer_items
                }),
        )
        .child(div().h_px().bg(cx.theme().border).mx_3())
        .child(property_panel)
        .child(font_hint)
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

