use gpui_kit::base::ElementExt;
use gpui_kit::{component::{v_flex, Icon}, *};
use std::{cell::Cell, rc::Rc, time::Duration};

pub struct ToastHost {
    message: Option<String>,
    generation: u64,
    timer: Option<Task<()>>,
    offset: Point<Pixels>,
    drag: Option<(Point<Pixels>, Point<Pixels>)>,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    info: Icon,
    close: Icon,
    close_label: SharedString,
}
impl ToastHost {
    pub fn new(info: Icon, close: Icon, close_label: impl Into<SharedString>) -> Self {
        Self { message: None, generation: 0, timer: None, offset: Point::default(), drag: None, bounds: Rc::new(Cell::new(Bounds::default())), info, close, close_label: close_label.into() }
    }
    pub fn show(&mut self, message: String, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.message = Some(message);
        self.offset = Point::default();
        self.drag = None;
        self.timer = Some(cx.spawn(async move |view, cx| {
            cx.background_executor().timer(Duration::from_secs(3)).await;
            let _ = view.update(cx, |this, cx| {
                if this.generation == generation { this.clear(cx); }
            });
        }));
        cx.notify();
    }
    pub fn take(&mut self, cx: &mut Context<Self>) -> Option<String> {
        let result = self.message.take();
        self.clear(cx);
        result
    }
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.message = None;
        self.timer = None;
        self.drag = None;
        self.generation = self.generation.wrapping_add(1);
        cx.notify();
    }
    pub fn set_close_label(&mut self, label: impl Into<SharedString>) { self.close_label = label.into(); }
    pub fn end_drag(&mut self) { self.drag = None; }
    pub fn pointer_move(&mut self, event: &MouseMoveEvent, window: &Window, cx: &mut Context<Self>) {
        if let Some((start, offset)) = self.drag {
            if event.pressed_button != Some(MouseButton::Left) { self.drag = None; return; }
            let bounds = self.bounds.get();
            let panel = Bounds::new(Point::default(), window.viewport_size());
            let base = bounds.origin - self.offset;
            let next = offset + event.position - start;
            self.offset = point(next.x.max(panel.left()+px(12.)-base.x).min(panel.right()-px(12.)-base.x-bounds.size.width), next.y.max(panel.top()+px(12.)-base.y).min(panel.bottom()-px(12.)-base.y-bounds.size.height));
            cx.notify();
        }
    }
}
impl Render for ToastHost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(message) = self.message.clone() else { return div().into_any_element(); };
        let measured = self.bounds.clone();
        let close = crate::controls::icon_button("dismiss-message", self.close.clone(), &self.close_label, false, cx)
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(|this, _, _, cx| this.clear(cx)));
        v_flex().absolute().bottom(px(12.)).left(px(12.)).right(px(12.)).items_center().child(
            crate::notification::notification("message-notification", self.info.clone(), div().whitespace_normal().child(message), close, cx)
                .id("notification-drag").relative().left(self.offset.x).top(self.offset.y)
                .on_prepaint(move |bounds, _, _| measured.set(bounds))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, _, cx| {
                    this.drag = Some((event.position, this.offset)); cx.stop_propagation();
                })).on_click(|_, _, cx| cx.stop_propagation())
        ).into_any_element()
    }
}
