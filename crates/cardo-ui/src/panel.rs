use crate::ConditionalBuilder;
use crate::{text::{body_text, compact_text}, controls::command};
use gpui_kit::component::scroll::ScrollableElement;
use crate::metrics::popup as metrics;
use gpui_kit::{
    component::{
        Sizable,
        button::{Button, ButtonVariants},
        h_flex,
        scroll::Scrollable,
        v_flex,
    },
    *,
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Default)]
pub struct TabIndicator {
    target: Option<Pixels>,
    origin: Pixels,
    started: Option<Instant>,
}

impl TabIndicator {
    const DURATION: Duration = Duration::from_millis(200);

    pub fn move_to(&mut self, target: Pixels, reduce_motion: bool) {
        if self.target == Some(target) {
            return;
        }
        let now = Instant::now();
        self.origin = self.position(now).0;
        self.started = if self.target.is_some() && !reduce_motion {
            Some(now)
        } else {
            None
        };
        self.target = Some(target);
    }

    fn position(&self, now: Instant) -> (Pixels, bool) {
        let target = self.target.unwrap_or_default();
        let Some(started) = self.started else {
            return (target, false);
        };
        let progress =
            (now.duration_since(started).as_secs_f32() / Self::DURATION.as_secs_f32()).min(1.);
        let eased = 1. - (1. - progress).powi(3);
        (self.origin + (target - self.origin) * eased, progress < 1.)
    }
}

pub fn panel_frame(cx: &App) -> Div {
    v_flex()
        .size_full()
        .min_w_0()
        .min_h_0()
        .bg(rgb(crate::theme::palette(cx).panel))
        .text_color(rgb(crate::theme::palette(cx).text))
        .font(crate::theme::interface_font(cx))
        .text_size(px(metrics::BODY_TEXT))
        .whitespace_normal()
        .line_height(px(metrics::LINE_HEIGHT))
}

pub fn panel_layout(cx: &App) -> Div {
    panel_frame(cx).bg(rgb(crate::theme::palette(cx).surface))
}

pub fn panel_surface(cx: &App) -> Div {
    let p = crate::theme::palette(cx);
    v_flex()
        .relative()
        .flex_1()
        .min_w_0()
        .min_h_0()
        .rounded(px(crate::metrics::PANEL_RADIUS))
        // GPUI's overflow mask is rectangular; inset children clear the rounded corners.
        .p(px(8.))
        .bg(rgb(p.surface))
        .border_1()
        .border_color(rgb(p.border))
        .shadow_sm()
}

pub fn popup_surface(cx: &App) -> Div {
    let p = crate::theme::palette(cx);
    v_flex()
        .min_w_0()
        .p(px(6.))
        .rounded(px(8.))
        .bg(rgb(p.surface))
        .text_color(rgb(p.text))
        .border_1()
        .border_color(rgb(p.border))
        .shadow_sm()
}

pub fn connected_panel_outline(
    indicator: std::rc::Rc<std::cell::Cell<TabIndicator>>,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, cx| {
            let p = crate::theme::palette(cx);
            let left = bounds.left() + px(0.5);
            let right = bounds.right() - px(0.5);
            let top = bounds.top() + px(0.5);
            let bottom = bounds.bottom() - px(0.5);
            let radius = px(crate::metrics::PANEL_RADIUS);
            let mut state = indicator.get();
            if cx.reduce_motion() {
                state.started = None;
                indicator.set(state);
            }
            let (center, moving) = state.position(Instant::now());
            if moving {
                window.request_animation_frame();
            }
            let x = center
                .max(left + radius + px(18.))
                .min(right - radius - px(18.));
            // The notch and frame are one closed path, so no border crosses their junction.
            for (mut path, color) in [
                (PathBuilder::fill(), p.surface),
                (PathBuilder::stroke(px(1.)), p.border),
            ] {
                path.move_to(point(left + radius, top));
                path.line_to(point(x - px(18.), top));
                path.cubic_bezier_to(
                    point(x, top - px(10.)),
                    point(x - px(8.), top),
                    point(x - px(7.), top - px(10.)),
                );
                path.cubic_bezier_to(
                    point(x + px(18.), top),
                    point(x + px(7.), top - px(10.)),
                    point(x + px(8.), top),
                );
                path.line_to(point(right - radius, top));
                path.curve_to(point(right, top + radius), point(right, top));
                path.line_to(point(right, bottom - radius));
                path.curve_to(point(right - radius, bottom), point(right, bottom));
                path.line_to(point(left + radius, bottom));
                path.curve_to(point(left, bottom - radius), point(left, bottom));
                path.line_to(point(left, top + radius));
                path.curve_to(point(left + radius, top), point(left, top));
                path.close();
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgb(color));
                }
            }
        },
    )
    .absolute()
    .inset_0()
}

fn panel_band(cx: &App) -> Div {
    let p = crate::theme::palette(cx);
    h_flex()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .bg(rgb(p.surface))
        .text_color(rgb(p.text))
        .border_color(rgb(p.border))
}

pub fn panel_header(title: impl Into<SharedString>, drag: bool, cx: &App) -> Div {

    panel_band(cx)
        .relative()
        .bg(rgb(crate::theme::palette(cx).panel))
        .h(px(metrics::TITLE_HEIGHT))
        .pl(px(metrics::PADDING))
        .pr(px(metrics::FIELD_GAP))
        .gap(px(metrics::FIELD_GAP))
        .items_center()
        .child(
            h_flex()
                .flex_1()
                .min_w_0()
                .h_full()
                .when(drag, |el| el.window_control_area(WindowControlArea::Drag))
                .child(
                    compact_text("dialog-heading", title)
                        .flex_1()
                        .text_size(px(metrics::TITLE_TEXT))
                        .font_weight(FontWeight::SEMIBOLD),
                ),
        )
        .when(drag, |el| el.child(window_grip(cx)))
}

pub fn window_grip(cx: &App) -> Div {
    use crate::metrics::titlebar as grip;
    v_flex()
        .absolute()
        .top_0()
        .left(relative(0.5))
        .ml(px(-grip::GRIP_WIDTH / 2.))
        .w(px(grip::GRIP_WIDTH))
        .h_full()
        .items_center()
        .justify_center()
        .gap(px(grip::GRIP_GAP))
        .window_control_area(WindowControlArea::Drag)
        .opacity(grip::GRIP_OPACITY)
        .children((0..2).map(|_| {
            h_flex().gap(px(grip::GRIP_GAP)).children((0..4).map(|_| {
                div()
                    .size(px(grip::GRIP_DOT_SIZE))
                    .rounded(px(grip::GRIP_DOT_SIZE))
                    .bg(rgb(crate::theme::palette(cx).muted))
            }))
        }))
}

pub fn panel_body(id: &'static str) -> Scrollable<Stateful<Div>> {
    body_container(id).overflow_y_scrollbar().id(id)
}

pub fn panel_document(id: &'static str) -> Scrollable<Stateful<Div>> {
    body_container(id).overflow_scrollbar().id(id)
}

pub fn body_container(id: &'static str) -> Stateful<Div> {
    v_flex()
        .id(id)
        .flex_1()
        .min_w_0()
        .min_h_0()
        .p(px(metrics::PADDING))
        .gap(px(metrics::GAP))
}

pub fn panel_actions(cx: &App) -> Div {
    panel_band(cx)
        .flex_wrap()
        .items_center()
        .justify_end()
        .px(px(metrics::PADDING))
        .py(px(metrics::FOOTER_PADDING))
        .gap(px(metrics::FIELD_GAP))
}

pub fn panel_button(id: impl Into<ElementId>, label: &str) -> Button {
    command(id, label)
        .small()
        .rounded(px(crate::settings::metrics::CONTROL_RADIUS))
        .border_0()
        .min_w(px(metrics::ACTION_MIN_WIDTH))
        .flex_shrink_0()
}

pub fn panel_primary(id: impl Into<ElementId>, label: &str) -> Button {
    panel_button(id, label).primary()
}

pub fn panel_danger(id: impl Into<ElementId>, label: &str) -> Button {
    panel_button(id, label).danger()
}

pub fn panel_field(label: impl Into<SharedString>, content: impl IntoElement) -> Div {
    v_flex()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .gap(px(metrics::FIELD_GAP))
        .child(
            body_text(label)
                .w_full()
                .text_size(px(metrics::BODY_TEXT))
                .font_weight(FontWeight::SEMIBOLD),
        )
        .child(content)
}

pub fn panel_notice(image: impl IntoElement, text: impl Into<SharedString>, color: u32) -> Div {
    h_flex()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .items_start()
        .gap(px(12.))
        .child(div().flex_shrink_0().text_color(rgb(color)).child(image))
        .child(body_text(text).flex_1())
}

pub fn panel_artwork_notice(
    artwork: impl IntoElement,
    text: impl Into<SharedString>,
 ) -> Div {
    h_flex()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .gap(px(metrics::GAP))
        .child(artwork)
        .child(
            body_text(text)
                .flex_1()
                .text_size(px(16.))
                .font_weight(FontWeight::SEMIBOLD),
        )
}

pub fn panel_card(cx: &App) -> Div {
    crate::settings::frame(cx)
        .w_auto()
        .p(px(metrics::CARD_PADDING))
        .gap(px(metrics::GAP))
}

pub fn panel_property(
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
    cx: &App,
) -> Div {
    let p = crate::theme::palette(cx);
    h_flex()
        .w_full()
        .min_w_0()
        .items_start()
        .gap(px(16.))
        .py(px(4.))
        .child(
            body_text(label)
                .w(px(104.))
                .flex_shrink_0()
                .text_size(px(12.))
                .text_color(rgb(p.muted)),
        )
        .child(body_text(value).flex_1())
}
