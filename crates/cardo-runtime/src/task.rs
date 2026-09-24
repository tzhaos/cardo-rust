use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn cancel(&self) { self.0.store(true, Ordering::Relaxed); }
    pub fn is_cancelled(&self) -> bool { self.0.load(Ordering::Relaxed) }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase { #[default] Idle, Running, Cancelling, Finished }
pub struct TaskState<P> { phase: Phase, label: String, cancel: CancellationToken, progress: Option<P> }
impl<P> Default for TaskState<P> {
    fn default() -> Self { Self { phase: Phase::Idle, label: String::new(), cancel: Default::default(), progress: None } }
}
impl<P> TaskState<P> {
    pub fn begin(&mut self, label: &str) -> CancellationToken { self.phase=Phase::Running; self.label=label.into(); self.cancel=Default::default(); self.progress=None; self.cancel.clone() }
    pub fn is_busy(&self) -> bool { matches!(self.phase, Phase::Running | Phase::Cancelling) }
    pub fn phase(&self) -> Phase { self.phase }
    pub fn status(&self) -> &str { &self.label }
    pub fn cancelled(&self) -> bool { self.cancel.is_cancelled() }
    pub fn cancel(&mut self, label: &str) { if self.is_busy() { self.cancel.cancel(); self.phase=Phase::Cancelling; self.label=label.into(); } }
    pub fn finish(&mut self) -> Option<P> { self.phase=Phase::Finished; self.progress.take() }
    pub fn set_progress(&mut self, progress: P) { self.progress=Some(progress); }
    pub fn progress(&self) -> Option<&P> { self.progress.as_ref() }
}
impl<P> Drop for TaskState<P> { fn drop(&mut self) { self.cancel.cancel(); } }
