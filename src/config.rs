//! Saved settings, kept in the app data folder.
//!
//! Hotkeys are stored with the key by name (`"KeyP"`, `"Space"`, `"Equal"`), as
//! `keyboard_types::Code` spells it, instead of a raw keycode: a name can be
//! checked when it's loaded, and an invalid one can't turn into an arbitrary key.

use std::fs;
use std::path::Path;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use serde::{Deserialize, Serialize};

use crate::hotkeys::HotkeyCmd;

/// The file, inside the app data folder, the shortcuts live in.
const HOTKEYS_FILE: &str = "hotkeys.json";

/// One system-wide shortcut: the modifier keys, as `Modifiers` bits, and a key name.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HotkeyBinding {
    pub modifiers: u32,
    pub key: String,
}

impl HotkeyBinding {
    pub fn new(modifiers: Modifiers, key: Code) -> Self {
        Self { modifiers: modifiers.bits(), key: key.to_string() }
    }

    /// The hotkey to register, or `None` when the saved key name is unknown.
    pub fn hotkey(&self) -> Option<HotKey> {
        let key = self.key.parse::<Code>().ok()?;
        Some(HotKey::new(Some(Modifiers::from_bits_truncate(self.modifiers)), key))
    }

    /// `Alt + Shift + P`, for the settings dialog.
    pub fn label(&self) -> String {
        let modifiers = Modifiers::from_bits_truncate(self.modifiers);
        let mut parts: Vec<&str> = Vec::new();
        if modifiers.contains(Modifiers::CONTROL) {
            parts.push("Ctrl");
        }
        if modifiers.contains(Modifiers::ALT) {
            parts.push("Alt");
        }
        if modifiers.contains(Modifiers::SHIFT) {
            parts.push("Shift");
        }
        if modifiers.contains(Modifiers::SUPER) {
            parts.push("Super");
        }
        let key = match self.key.as_str() {
            "Equal" => "=",
            "Minus" => "-",
            "Comma" => ",",
            "Period" => ".",
            other => other.strip_prefix("Key").or_else(|| other.strip_prefix("Digit")).unwrap_or(other),
        };
        parts.push(key);
        parts.join(" + ")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HotkeyConfig {
    pub play_pause: HotkeyBinding,
    pub next: HotkeyBinding,
    pub prev: HotkeyBinding,
    pub volume_up: HotkeyBinding,
    pub volume_down: HotkeyBinding,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        let modifiers = Modifiers::ALT | Modifiers::SHIFT;
        Self {
            play_pause: HotkeyBinding::new(modifiers, Code::KeyP),
            next: HotkeyBinding::new(modifiers, Code::KeyN),
            prev: HotkeyBinding::new(modifiers, Code::KeyB),
            volume_up: HotkeyBinding::new(modifiers, Code::Equal),
            volume_down: HotkeyBinding::new(modifiers, Code::Minus),
        }
    }
}

impl HotkeyConfig {
    pub fn binding(&self, cmd: HotkeyCmd) -> &HotkeyBinding {
        match cmd {
            HotkeyCmd::TogglePlay => &self.play_pause,
            HotkeyCmd::Next => &self.next,
            HotkeyCmd::Prev => &self.prev,
            HotkeyCmd::VolumeUp => &self.volume_up,
            HotkeyCmd::VolumeDown => &self.volume_down,
        }
    }

    pub fn binding_mut(&mut self, cmd: HotkeyCmd) -> &mut HotkeyBinding {
        match cmd {
            HotkeyCmd::TogglePlay => &mut self.play_pause,
            HotkeyCmd::Next => &mut self.next,
            HotkeyCmd::Prev => &mut self.prev,
            HotkeyCmd::VolumeUp => &mut self.volume_up,
            HotkeyCmd::VolumeDown => &mut self.volume_down,
        }
    }
}

/// The saved shortcuts, or the defaults when there's no readable file yet.
pub fn load_hotkeys(data_dir: &Path) -> HotkeyConfig {
    let path = data_dir.join(HOTKEYS_FILE);
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            log::warn!("ignoring unreadable {}: {e}", path.display());
            HotkeyConfig::default()
        }),
        Err(_) => HotkeyConfig::default(),
    }
}

pub fn save_hotkeys(data_dir: &Path, config: &HotkeyConfig) -> std::io::Result<()> {
    fs::create_dir_all(data_dir)?;
    let path = data_dir.join(HOTKEYS_FILE);
    let text = serde_json::to_string_pretty(config).map_err(std::io::Error::other)?;
    // Write beside it and rename, so a crash can't leave half a file behind.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text)?;
    fs::rename(&tmp, &path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindings_round_trip_through_disk() {
        let dir = std::env::temp_dir().join(format!("ytm-rs-hotkeys-{}", std::process::id()));
        let config = HotkeyConfig {
            next: HotkeyBinding::new(Modifiers::CONTROL | Modifiers::ALT, Code::KeyJ),
            ..Default::default()
        };
        save_hotkeys(&dir, &config).unwrap();
        assert_eq!(load_hotkeys(&dir), config);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_saved_binding_makes_a_registrable_hotkey() {
        let binding = HotkeyBinding::new(Modifiers::ALT | Modifiers::SHIFT, Code::KeyP);
        assert!(binding.hotkey().is_some());
        assert_eq!(binding.label(), "Alt + Shift + P");
    }

    #[test]
    fn an_unknown_key_name_is_not_a_hotkey() {
        let binding = HotkeyBinding { modifiers: 0, key: "NotAKey".to_owned() };
        assert!(binding.hotkey().is_none());
    }
}
