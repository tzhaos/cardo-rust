use gpui_kit::{component::v_flex, *};

/// The host frame has no knowledge of navigation, pages or product state.
/// Applications compose their own header, navigation and body using this frame.
pub fn app_frame(cx: &App) -> Div {
    v_flex()
        .relative()
        .size_full()
        .min_w_0()
        .min_h_0()
        .bg(rgb(crate::theme::palette(cx).panel))
        .text_color(rgb(crate::theme::palette(cx).text))
        .font(crate::theme::interface_font(cx))
        .text_size(crate::theme::ui_font_size(cx))
        .whitespace_normal()
}
