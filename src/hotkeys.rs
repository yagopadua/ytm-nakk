//! Hotkeys globais do sistema.

use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers}};
use global_hotkey::GlobalHotKeyEvent;

pub enum HotkeyCmd {
    TogglePlay,
    Next,
    Prev,
    VolumeUp,
    VolumeDown,
}

pub struct GlobalHotkeys {
    _manager: GlobalHotKeyManager,
    hk_play: HotKey,
    hk_next: HotKey,
    hk_prev: HotKey,
    hk_vol_up: HotKey,
    hk_vol_down: HotKey,
}

impl GlobalHotkeys {
    pub fn new() -> Option<Self> {
        let manager = GlobalHotKeyManager::new().ok()?;
        let config = crate::config::load_config();

        let hk_play = HotKey::new(
            Some(Modifiers::from_bits_truncate(config.play_pause.0)), 
            unsafe { std::mem::transmute(config.play_pause.1 as u8) }
        );
        let hk_next = HotKey::new(
            Some(Modifiers::from_bits_truncate(config.next.0)), 
            unsafe { std::mem::transmute(config.next.1 as u8) }
        );
        let hk_prev = HotKey::new(
            Some(Modifiers::from_bits_truncate(config.prev.0)), 
            unsafe { std::mem::transmute(config.prev.1 as u8) }
        );
        let hk_vol_up = HotKey::new(
            Some(Modifiers::from_bits_truncate(config.vol_up.0)), 
            unsafe { std::mem::transmute(config.vol_up.1 as u8) }
        );
        let hk_vol_down = HotKey::new(
            Some(Modifiers::from_bits_truncate(config.vol_down.0)), 
            unsafe { std::mem::transmute(config.vol_down.1 as u8) }
        );

        let _ = manager.register(hk_play);
        let _ = manager.register(hk_next);
        let _ = manager.register(hk_prev);
        let _ = manager.register(hk_vol_up);
        let _ = manager.register(hk_vol_down);

        Some(Self {
            _manager: manager,
            hk_play,
            hk_next,
            hk_prev,
            hk_vol_up,
            hk_vol_down,
        })
    }

    pub fn poll(&self) -> Option<HotkeyCmd> {
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == global_hotkey::HotKeyState::Pressed {
                let id = event.id;
                println!("Hotkey event received: id={:?}", id);
                if id == self.hk_play.id() {
                    return Some(HotkeyCmd::TogglePlay);
                } else if id == self.hk_next.id() {
                    return Some(HotkeyCmd::Next);
                } else if id == self.hk_prev.id() {
                    return Some(HotkeyCmd::Prev);
                } else if id == self.hk_vol_up.id() {
                    return Some(HotkeyCmd::VolumeUp);
                } else if id == self.hk_vol_down.id() {
                    return Some(HotkeyCmd::VolumeDown);
                }
            }
        }
        None
    }
}
