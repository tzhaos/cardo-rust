use gpui_kit::*;
use cardo_runtime::shortcuts::Shortcut;
pub fn chord(event: &KeyDownEvent) -> Option<Shortcut> {
    let modifiers = event.keystroke.modifiers;
    if modifiers.platform || modifiers.function {
        return None;
    }
    Some(Shortcut {
        key: event.keystroke.key.to_lowercase(),
        control: modifiers.control,
        alt: modifiers.alt,
        shift: modifiers.shift,
    })
}


pub enum Recording { Ignore, Cancel, Clear, Reserved, Assign(Shortcut) }
pub fn record(event: &KeyDownEvent) -> Recording {
    if event.is_held || matches!(event.keystroke.key.as_str(), "control"|"shift"|"alt"|"platform"|"fn") { return Recording::Ignore; }
    if event.keystroke.key == "escape" { return Recording::Cancel; }
    let Some(key)=chord(event) else { return Recording::Reserved; };
    if key.key == "backspace" && !key.control && !key.alt && !key.shift { Recording::Clear }
    else if key.allowed() { Recording::Assign(key) } else { Recording::Reserved }
}
