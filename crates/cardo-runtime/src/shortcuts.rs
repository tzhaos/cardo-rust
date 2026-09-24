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
        if self.control && !self.alt && matches!(key, "c" | "v" | "x" | "z" | "y") && !self.shift {
            return false;
        }
        if self.alt && matches!(key, "f4" | "space" | "tab" | "down") {
            return false;
        }
        if self.shift && !self.control && !self.alt && key == "f10" {
            return false;
        }
        true
    }
}

pub fn conflict<A: Copy + PartialEq>(
    action: A,
    binding: &Shortcut,
    commands: impl IntoIterator<Item = (A, Option<Shortcut>)>,
) -> Option<A> {
    commands
        .into_iter()
        .find(|(candidate, keys)| *candidate != action && keys.as_ref() == Some(binding))
        .map(|(id, _)| id)
}
impl Shortcut {
    pub fn protects_input(&self) -> bool {
        !self.alt
            && (self.key == "delete"
                || (self.control
                    && matches!(
                        self.key.as_str(),
                        "a" | "c"
                            | "v"
                            | "x"
                            | "z"
                            | "y"
                            | "left"
                            | "right"
                            | "up"
                            | "down"
                            | "home"
                            | "end"
                    )))
    }
}

/// Stable command identity and its default binding. Applications own labels and execution.
pub struct CommandDescriptor<A> {
    pub id: A,
    pub label: &'static str,
    pub default: Option<Shortcut>,
}
impl<A: Ord> CommandDescriptor<A> {
    pub fn binding(
        &self,
        values: &std::collections::BTreeMap<A, Option<Shortcut>>,
    ) -> Option<Shortcut> {
        values
            .get(&self.id)
            .cloned()
            .unwrap_or_else(|| self.default.clone())
    }
}

/// Missing keys use defaults, `"disabled"` explicitly removes a binding.
pub mod config_map {
    use super::*;
    use serde::{Deserializer, Serializer};
    use std::collections::BTreeMap;
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum Disabled {
        Disabled,
    }
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    enum Binding {
        Keys(Shortcut),
        Disabled(Disabled),
    }
    pub fn serialize<A: Ord + Clone + Serialize, S: Serializer>(
        values: &BTreeMap<A, Option<Shortcut>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        values
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    match value {
                        Some(keys) => Binding::Keys(keys.clone()),
                        None => Binding::Disabled(Disabled::Disabled),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>()
            .serialize(serializer)
    }
    pub fn deserialize<'de, A: Ord + Deserialize<'de>, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<A, Option<Shortcut>>, D::Error> {
        Ok(BTreeMap::<A, Binding>::deserialize(deserializer)?
            .into_iter()
            .map(|(key, value)| {
                (
                    key,
                    match value {
                        Binding::Keys(keys) => Some(keys),
                        Binding::Disabled(_) => None,
                    },
                )
            })
            .collect())
    }
}
