
use crate::{text::{body_text, compact_text}, tooltip::bubble_tooltip};
use crate::metrics::*;
use gpui_kit::component::{
    Sizable,
    button::{Button, ButtonCustomVariant, ButtonVariants},
    checkbox::Checkbox,
    input::{Input, InputState},
};

use gpui_kit::*;

pub fn subtle_variant(cx: &App) -> ButtonCustomVariant {
    let p = crate::theme::palette(cx);
    ButtonCustomVariant::new(cx)
        .hover(rgb(p.hover).into())
        .active(rgb(p.selected).into())
}

pub fn header_variant(cx: &App) -> ButtonCustomVariant {
    let p = crate::theme::palette(cx);
    ButtonCustomVariant::new(cx)
        .hover(rgb(p.border).into())
        .active(rgb(p.selected).into())
}

pub fn command(id: impl Into<ElementId>, label: &str) -> Button {
    // Button sizes control the inner label; an outer text_size is overridden.
    Button::new(id)
        .secondary()
        .xsmall()
        .accessibility_label(label.to_owned())
        .child(compact_text("command-label", label.to_owned()).line_height(relative(1.)))
        .min_w_0()
        .max_w_full()
        .h(px(CONTROL_HEIGHT))
        .px(px(12.))
        .rounded(px(CONTROL_RADIUS))
        .border_1()
        .shadow_none()
}

pub fn checkbox(id: impl Into<ElementId>, label: &str) -> Checkbox {
    Checkbox::new(id)
        .accessibility_label(label.to_owned())
        .w_full()
        .min_w_0()
        .child(body_text(label.to_owned()))
}

pub fn text_input(state: &Entity<InputState>) -> Input {
    Styled::h(Input::new(state).xsmall(), px(CONTROL_HEIGHT))
}

pub fn primary(id: impl Into<ElementId>, label: &str) -> Button {
    command(id, label).primary()
}

pub fn icon_button(
    id: impl Into<ElementId>,
    image: gpui_kit::component::Icon,
    title: &str,
    hint: bool,
    cx: &App,
) -> Button {
    let button = Button::new(id)
        .custom(subtle_variant(cx))
        .compact()
        .icon(image)
        .accessibility_label(title.to_owned())
        .w(px(ICON_BUTTON_SIZE))
        // Tooltip measurement adds a child; override the compact button's 32px minimum.
        .min_w_0()
        .h(px(ICON_BUTTON_SIZE))
        .p_0()
        .flex_shrink_0()
        .rounded(px(CONTROL_RADIUS))
        .border_0()
        .shadow_none();
    if hint {
        bubble_tooltip(button, title.to_owned())
    } else {
        button
    }
}
