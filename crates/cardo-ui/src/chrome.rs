use crate::ConditionalBuilder;
use crate::{controls::subtle_variant, panel::window_grip, metrics::titlebar as metrics};
use gpui_kit::{component::{Disableable, button::ButtonVariants, h_flex, Icon}, *};
pub fn titlebar(title: impl Into<SharedString>, menu: impl IntoElement, close_disabled: bool, labels: [SharedString; 3], icon: impl Fn(&str, f32) -> Icon, window: &Window, cx: &App) -> Div {
    let p = crate::theme::palette(cx);
    let title: SharedString = title.into();
        h_flex()
            .relative()
            .h(px(metrics::HEIGHT))
            .flex_shrink_0()
            .bg(rgb(p.panel))
            .items_center()
            .child(
                h_flex()
                    .h_full()
                    .flex_1()
                    .min_w_0()
                    .pl(px(16.))
                    .gap(px(12.))
                    .items_center()
                    .window_control_area(WindowControlArea::Drag)
                    .child(
                        div()
                            .id("window-heading")
                            .min_w_0()
                            .truncate()
                            .child(title)
                            .flex_1()
                            .font_weight(FontWeight::SEMIBOLD),
                    )
                    .child(menu),
            )
            .child(
                h_flex().flex_shrink_0().gap_0().children(
                    [
                        (
                            WindowControlArea::Min,
                            "minimize",
                            "Subtract",
                            labels[0].clone(),
                        ),
                        (
                            WindowControlArea::Max,
                            "maximize",
                            if window.is_maximized() {
                                "SquareMultiple"
                            } else {
                                "Square"
                            },
                            labels[1].clone(),
                        ),
                        (
                            WindowControlArea::Close,
                            "close",
                            "Dismiss",
                            labels[2].clone(),
                        ),
                    ]
                    .into_iter()
                    .map(|(area, id, name, label)| {
                        let disabled = area == WindowControlArea::Close
                            && (close_disabled);
                        gpui_kit::component::button::Button::new(id)
                            .group("window-control")
                            .custom(if area == WindowControlArea::Close {
                                subtle_variant(cx)
                                    .hover(rgb(0xe81123).into())
                                    .active(rgb(0xc50f1f).into())
                            } else {
                                subtle_variant(cx)
                            })
                            .accessibility_label(label)
                            .child(
                                div()
                                    .when(area == WindowControlArea::Close && !disabled, |el| {
                                        el.group_hover("window-control", |el| {
                                            el.text_color(rgb(0xffffff))
                                        })
                                    })
                                    .child(icon(name, 16.)),
                            )
                            .border_0()
                            .w(px(metrics::CONTROL_WIDTH))
                            .h(px(metrics::HEIGHT))
                            .rounded(px(0.))
                            .disabled(disabled)
                            // The pinned Windows backend's zoom() only maximizes;
                            // native Max hit testing handles both maximize and restore.
                            .when(area == WindowControlArea::Max || (area == WindowControlArea::Close && !disabled), |button| {
                                button.window_control_area(area)
                            })
                            .when(area == WindowControlArea::Min, |button| {
                                button
                                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation()
                                    })
                                    .on_click(|_, window, _| window.minimize_window())
                            })
                    }),
                ),
            )
            .child(window_grip(cx))
}
