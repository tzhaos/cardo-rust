use cardo_runtime::task::CancellationToken;
use gpui_kit::*;

/// Owns a background operation and its typed delivery to a weak UI owner.
/// Dropping requests cancellation; the worker still runs to its actual end.
pub struct TaskHandle {
    cancel: CancellationToken,
    delivery: Option<Task<()>>,
}
impl TaskHandle {
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
}
impl Drop for TaskHandle {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(task) = self.delivery.take() {
            task.detach();
        }
    }
}

pub fn spawn<T: 'static, R: Send + 'static>(
    cx: &mut Context<T>,
    cancel: CancellationToken,
    work: impl FnOnce(CancellationToken) -> R + Send + 'static,
    complete: impl FnOnce(&mut T, R, &mut Context<T>) + 'static,
) -> TaskHandle {
    let signal = cancel.clone();
    let worker = cx.background_executor().spawn(async move { work(signal) });
    let delivery = cx.spawn(async move |owner, cx| {
        let result = worker.await;
        let _ = owner.update(cx, |owner, cx| complete(owner, result, cx));
    });
    TaskHandle {
        cancel,
        delivery: Some(delivery),
    }
}

/// Refreshes only while the supplied real progress source is active.
pub fn refresh_while<T: 'static>(
    cx: &mut Context<T>,
    active: impl Fn(&T) -> bool + 'static,
) -> Task<()> {
    cx.spawn(async move |owner, cx| {
        loop {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(80))
                .await;
            if !owner
                .update(cx, |owner, cx| {
                    if active(owner) {
                        cx.notify();
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false)
            {
                break;
            }
        }
    })
}
