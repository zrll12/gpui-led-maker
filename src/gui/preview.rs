use crate::gui::Render;
use crate::modal::app_state::LiveProject;
use crate::modal::app_state::AppState;
use crate::render::frame::render_frame_to_image;
use gpui::{AnyElement, Context, Entity, Image, ImageFormat, IntoElement, ParentElement, Styled, Subscription, Window, div, img};
use gpui_component::{v_flex, ActiveTheme};
use image::ImageEncoder;
use simple_gpui::component;
use std::io::Cursor;
use std::sync::Arc;

fn rgb_image_to_gpui_image(rgb: image::RgbImage) -> Option<Arc<Image>> {
    // 将 RgbImage 编码为 PNG 字节流，存在内存中
    let mut buf = Cursor::new(Vec::new());
    image::codecs::png::PngEncoder::new(&mut buf)
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ColorType::Rgb8.into(),
        )
        .ok()?;
    Some(Arc::new(Image::from_bytes(ImageFormat::Png, buf.into_inner())))
}

#[component]
pub fn preview_panel(_window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    init_with_context!();
    component_property!(preview_image: Option<Arc<Image>> = None);

    observe!(LiveProject, |page, _window, cx| {
        let project = &cx.global::<LiveProject>().0;
        let font_list = cx.global::<AppState>().config.font_list.clone();
        page.preview_image = project
            .frames
            .first()
            .and_then(|frame| render_frame_to_image(frame, &font_list))
            .and_then(rgb_image_to_gpui_image);
        cx.notify();
    });

    let content: AnyElement = match &self.preview_image {
        Some(image) => img(image.clone())
            .max_w_full()
            .max_h_full()
            .into_any_element(),
        None => div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .child("暂无预览（请在编辑器中添加图层）")
            .into_any_element(),
    };

    v_flex().size_full().p_2().child(content)
}
