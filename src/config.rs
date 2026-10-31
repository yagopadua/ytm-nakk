use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use global_hotkey::hotkey::{Code, Modifiers};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotkeyConfig {
    pub play_pause: (u32, u32),
    pub next: (u32, u32),
    pub prev: (u32, u32),
    pub vol_up: (u32, u32),
    pub vol_down: (u32, u32),
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            play_pause: (Modifiers::ALT.bits() | Modifiers::SHIFT.bits(), Code::KeyP as u32),
            next: (Modifiers::ALT.bits() | Modifiers::SHIFT.bits(), Code::KeyN as u32),
            prev: (Modifiers::ALT.bits() | Modifiers::SHIFT.bits(), Code::KeyB as u32),
            vol_up: (Modifiers::ALT.bits() | Modifiers::SHIFT.bits(), Code::Equal as u32),
            vol_down: (Modifiers::ALT.bits() | Modifiers::SHIFT.bits(), Code::Minus as u32),
        }
    }
}

pub fn load_config() -> HotkeyConfig {
    let path = PathBuf::from("config/hotkeys.json");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(config) = serde_json::from_str(&content) {
            return config;
        }
    }
    let config = HotkeyConfig::default();
    let _ = fs::create_dir_all("config");
    let _ = fs::write(path, serde_json::to_string_pretty(&config).unwrap());
    config
}

#[allow(dead_code)]
pub fn save_config(config: &HotkeyConfig) {
    let path = PathBuf::from("config/hotkeys.json");
    let _ = fs::write(path, serde_json::to_string_pretty(config).unwrap());
}
