use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    pub key: String,
    pub control: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Shortcut {
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.control {
            parts.push("Ctrl".to_owned());
        }
        if self.alt {
            parts.push("Alt".to_owned());
        }
        if self.shift {
            parts.push("Shift".to_owned());
        }
        parts.push(match self.key.as_str() {
            "enter" => "Enter".into(),
            "delete" => "Del".into(),
            "left" => "Left".into(),
            "up" => "Up".into(),
            other => other.to_uppercase(),
        });
        parts.join("+")
    }

    pub fn allowed(&self) -> bool {
        let key = self.key.as_str();
        let function = key
            .strip_prefix('f')
            .and_then(|n| n.parse::<u8>().ok())
            .is_some_and(|n| (1..=12).contains(&n));
        let letter = key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric();
        let supported = function
            || letter
            || matches!(
                key,
                "," | "."
                    | "/"
                    | ";"
                    | "'"
                    | "["
                    | "]"
                    | "-"
                    | "="
                    | "delete"
                    | "enter"
                    | "left"
                    | "up"
                    | "right"
                    | "down"
                    | "home"
                    | "end"
                    | "pageup"
                    | "pagedown"
            );
        if !supported {
            return false;
        }
        if self.control && self.alt && key == "delete" {
            return false;
        }
        if !self.control && !self.alt && !function && key != "delete" {
            return false;
        }
        // Editing and native window/navigation keys retain their established behavior.
        if self.control
            && !self.alt
            && matches!(key, "c" | "v" | "x" | "z" | "y" | "s" | "w")
            && !self.shift
        {
            return false;
        }
        if self.alt && matches!(key, "f4" | "space" | "tab" | "down") {
            return false;
        }
        if self.shift && !self.control && !self.alt && key == "f10" {
            return false;
        }
        if self.control && !self.alt && !self.shift && key == "pagedown" {
            return false;
        }
        true
    }
}


pub fn conflict<A: Copy + PartialEq>(action: A, binding: &Shortcut, commands: impl IntoIterator<Item=(A, Option<Shortcut>)>) -> Option<A> {
    commands.into_iter().find(|(candidate, keys)| *candidate != action && keys.as_ref() == Some(binding)).map(|(id,_)| id)
}
impl Shortcut {
    pub fn protects_input(&self) -> bool {
        !self.alt && (self.key == "delete" || (self.control && matches!(self.key.as_str(), "a"|"c"|"v"|"x"|"z"|"y"|"left"|"right"|"up"|"down"|"home"|"end")))
    }
}
