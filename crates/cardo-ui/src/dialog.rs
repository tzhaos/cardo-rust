use gpui_kit::{component::Root, *};
use std::{cell::Cell, rc::Rc};
thread_local! { static OPENING: Cell<bool> = const { Cell::new(false) }; }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseReason {
    Submit,
    Cancel,
    Escape,
    Window,
    OwnerReleased,
}
#[derive(Clone, Copy)]
pub struct DialogSpec {
    pub preferred: Size<Pixels>,
    pub minimum: Size<Pixels>,
}
#[derive(Default)]
pub struct DialogHost {
    handle: Option<AnyWindowHandle>,
    generation: Rc<Cell<u64>>,
    previous_focus: Option<FocusHandle>,
    was_active: bool,
    closed: Rc<Cell<bool>>,
    callback: Option<Rc<dyn Fn(CloseReason, &mut App)>>,
}
impl DialogHost {
    pub fn handle(&self) -> Option<AnyWindowHandle> {
        self.handle
    }
    pub fn generation(&self) -> u64 {
        self.generation.get()
    }
    pub fn invalidate(&mut self) {
        self.generation.set(self.generation.get().wrapping_add(1));
    }
    pub fn detach(&mut self) {
        self.handle = None;
    }
    pub fn close(&mut self, reason: CloseReason, cx: &mut App) {
        if let Some(handle) = self.handle.take() {
            if !self.closed.replace(true) {
                if let Some(callback) = self.callback.take() {
                    cx.defer(move |cx| callback(reason, cx));
                }
            }
            let _ = handle.update(cx, |_, window, _| window.remove_window());
        }
    }
    pub fn sync_focus(&mut self, active: bool, window: &mut Window, cx: &mut App) {
        if active && !self.was_active {
            self.previous_focus = window.focused(cx);
        } else if !active && self.was_active {
            if let Some(focus) = self.previous_focus.take() {
                focus.focus(window, cx);
            }
        }
        self.was_active = active;
    }
    /// Creates window-bound content before the first rendered frame. The returned
    /// value lets the caller install its inputs without borrowing the owner again.
    pub fn open<T: 'static, C>(
        &mut self,
        owner: Entity<T>,
        owner_window: AnyWindowHandle,
        content: Bounds<Pixels>,
        title: SharedString,
        spec: DialogSpec,
        focus: FocusHandle,
        create: impl FnOnce(&mut Window, &mut App) -> C,
        render: impl Fn(&mut T, &mut Window, &mut Context<T>) -> Option<AnyElement> + 'static,
        can_close: impl Fn(&T, &App) -> bool + 'static,
        closed: impl Fn(&mut T, CloseReason, &mut App) + 'static,
        more_label: fn() -> SharedString,
        cx: &mut App,
    ) -> anyhow::Result<(AnyWindowHandle, C)> {
        self.close(CloseReason::Cancel, cx);
        self.invalidate();
        let positioned = owner_window.update(cx, |_, window, cx| {
            // Use live client bounds, not restored bounds or the active prompt.
            let client = window.bounds();
            let panel = if content.size.width > px(0.) && content.size.height > px(0.) {
                Bounds::new(client.origin + content.origin, content.size)
            } else {
                client
            };
            let display = window.display(cx);
            let screen = display
                .as_ref()
                .map(|display| display.visible_bounds())
                .unwrap_or(client);
            let available = size(
                (screen.size.width - px(40.)).max(px(1.)),
                (screen.size.height - px(40.)).max(px(1.)),
            );
            let preferred = spec.preferred;
            let actual = size(
                preferred.width.min(available.width),
                preferred.height.min(available.height),
            );
            let mut bounds = Bounds::centered_at(panel.center(), actual);
            bounds.origin.x = bounds
                .origin
                .x
                .max(screen.origin.x + px(20.))
                .min(screen.origin.x + screen.size.width - actual.width - px(20.));
            bounds.origin.y = bounds
                .origin
                .y
                .max(screen.origin.y + px(20.))
                .min(screen.origin.y + screen.size.height - actual.height - px(20.));
            (bounds, display.map(|d| d.id()))
        });
        let (bounds, display_id) = positioned?;

        let generation = self.generation.clone();
        let epoch = generation.get();
        self.closed = Rc::new(Cell::new(false));
        let closing = self.closed.clone();
        let closed = Rc::new(closed);
        let callback: Rc<dyn Fn(CloseReason, &mut App)> = {
            let weak = owner.downgrade();
            let closed = closed.clone();
            let generation = generation.clone();
            Rc::new(move |reason, cx| {
                if generation.get() == epoch {
                    let _ = weak.update(cx, |owner, cx| closed(owner, reason, cx));
                }
            })
        };
        self.callback = Some(callback.clone());
        let mut content = None;
        OPENING.set(true);
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                display_id,
                titlebar: Some(TitlebarOptions {
                    title: Some(title),
                    appears_transparent: true,
                    ..Default::default()
                }),
                kind: WindowKind::Normal,
                is_movable: true,
                is_resizable: true,
                is_minimizable: true,
                focus: true,
                inactive_frame_interval: None,
                window_min_size: Some(size(
                    spec.minimum.width.min(bounds.size.width),
                    spec.minimum.height.min(bounds.size.height),
                )),
                ..Default::default()
            },
            |window, cx| {
                focus.focus(window, cx);
                content = Some(create(window, cx));
                let view = cx.new(|cx| {
                    let weak = owner.downgrade();
                    let closing_window = closing.clone();
                    window.on_window_should_close(cx, {
                        let weak = weak.clone();
                        let closing = closing_window;
                        let callback = callback.clone();
                        let generation = generation.clone();
                        move |_, cx| {
                            if generation.get() != epoch || closing.get() {
                                return true;
                            }
                            let allowed = weak
                                .read_with(cx, |owner, cx| can_close(owner, cx))
                                .unwrap_or(true);
                            if allowed && !closing.replace(true) {
                                callback(CloseReason::Window, cx);
                            }
                            allowed
                        }
                    });
                    let handle = window.window_handle();
                    DialogWindow {
                        menu: crate::menu::MenuHost::install(window, cx, more_label),
                        owner: weak,
                        render: Rc::new(render),
                        _watch: cx.observe(&owner, |_, _, cx| cx.notify()),
                        _release: cx.observe_release(&owner, move |_, owner, cx| {
                            if !closing.replace(true) {
                                closed(owner, CloseReason::OwnerReleased, cx);
                            }
                            let _ = handle.update(cx, |_, window, _| window.remove_window());
                        }),
                    }
                });
                cx.new(|cx| Root::new(view, window, cx))
            },
        );
        OPENING.set(false);
        let handle: AnyWindowHandle = opened?.into();
        self.handle = Some(handle);
        Ok((handle, content.expect("content created with dialog window")))
    }
}
struct DialogWindow<T: 'static> {
    menu: Entity<crate::menu::MenuHost>,
    owner: WeakEntity<T>,
    render: Rc<dyn Fn(&mut T, &mut Window, &mut Context<T>) -> Option<AnyElement>>,
    _watch: Subscription,
    _release: Subscription,
}
impl<T: 'static> Render for DialogWindow<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if OPENING.get() {
            return div().size_full().into_any_element();
        }
        let content = self
            .owner
            .update(cx, |owner, cx| (self.render)(owner, window, cx))
            .ok()
            .flatten();
        if content.is_none() {
            cx.defer_in(window, |_, window, _| window.remove_window());
        }
        div()
            .relative()
            .size_full()
            .children(content)
            .child(self.menu.clone())
            .into_any_element()
    }
}
