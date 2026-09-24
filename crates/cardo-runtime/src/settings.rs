use std::collections::BTreeMap;
pub struct SettingsDraft<T> { value: T, saved: T, pending: Option<T>, pub errors: BTreeMap<String, String> }
impl<T: Clone> SettingsDraft<T> {
    pub fn new(value: T) -> Self { Self { saved: value.clone(), value, pending: None, errors: BTreeMap::new() } }
    pub fn is_saving(&self) -> bool { self.pending.is_some() }
    pub fn begin_save(&mut self) -> Option<T> { if self.is_saving() { return None; } self.errors.clear(); self.pending=Some(self.value.clone()); self.pending.clone() }
    pub fn complete(&mut self) { if let Some(value)=self.pending.take() { self.saved=value; } }
    pub fn fail(&mut self, field: impl Into<String>, error: String) { self.pending=None; self.errors.insert(field.into(),error); }
    pub fn discard(&mut self) { if !self.is_saving() { self.value=self.saved.clone(); self.errors.clear(); } }
}
impl<T> std::ops::Deref for SettingsDraft<T> { type Target=T; fn deref(&self)->&T { &self.value } }
impl<T> std::ops::DerefMut for SettingsDraft<T> { fn deref_mut(&mut self)->&mut T { &mut self.value } }
