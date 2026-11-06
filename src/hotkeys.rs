//! System-wide media hotkeys.

use std::sync::{Arc, Mutex, Once, OnceLock};

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

use crate::config::HotkeyConfig;

/// Presses the system delivered, waiting for the next [`GlobalHotkeys::poll`]
/// on the UI thread. They go through this queue instead of the crate's channel
/// so that the event handler can also wake the UI: while the window is hidden
/// or minimized egui runs no frame on its own, and a shortcut press would
/// otherwise sit unread until the window was shown again.
static PENDING: Mutex<Vec<GlobalHotKeyEvent>> = Mutex::new(Vec::new());
/// Wakes the UI when a shortcut is pressed. Set once the context exists.
static WAKE: OnceLock<Arc<dyn Fn() + Send + Sync>> = OnceLock::new();
static INSTALLED: Once = Once::new();

/// Makes shortcut presses wake the UI, so they still work while it's minimized.
/// Call once, as soon as the egui context exists and before registering any key.
pub fn install_wake(wake: impl Fn() + Send + Sync + 'static) {
    INSTALLED.call_once(|| {
        let _ = WAKE.set(Arc::new(wake));
        // Replaces the crate's channel: `poll` reads `PENDING` instead.
        GlobalHotKeyEvent::set_event_handler(Some(|event| {
            if let Ok(mut queue) = PENDING.lock() {
                queue.push(event);
            }
            if let Some(wake) = WAKE.get() {
                wake();
            }
        }));
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HotkeyCmd {
    TogglePlay,
    Next,
    Prev,
    VolumeUp,
    VolumeDown,
}

impl HotkeyCmd {
    pub const ALL: [HotkeyCmd; 5] = [Self::TogglePlay, Self::Next, Self::Prev, Self::VolumeUp, Self::VolumeDown];

    /// What the shortcut does, for the settings dialog.
    pub fn label(self) -> &'static str {
        match self {
            Self::TogglePlay => "Play / pause",
            Self::Next => "Next song",
            Self::Prev => "Previous song",
            Self::VolumeUp => "Volume up",
            Self::VolumeDown => "Volume down",
        }
    }
}

pub struct GlobalHotkeys {
    /// Kept alive: dropping the manager unregisters every shortcut.
    _manager: GlobalHotKeyManager,
    /// Registered hotkey ids and what each one does.
    bound: Vec<(u32, HotkeyCmd)>,
    /// The shortcuts the system wouldn't give us, usually because another app
    /// already owns them. Kept so the settings dialog can say which.
    failed: Vec<HotkeyCmd>,
}

impl GlobalHotkeys {
    pub fn new(config: &HotkeyConfig) -> Option<Self> {
        let manager = match GlobalHotKeyManager::new() {
            Ok(manager) => manager,
            Err(e) => {
                log::warn!("global hotkeys unavailable: {e}");
                return None;
            }
        };
        // A press that arrived while the shortcuts were unregistered (the settings
        // dialog captures a new one with none active) must not fire afterwards.
        if let Ok(mut queue) = PENDING.lock() {
            queue.clear();
        }
        let mut bound = Vec::new();
        let mut failed = Vec::new();
        for cmd in HotkeyCmd::ALL {
            let binding = config.binding(cmd);
            let Some(hotkey) = binding.hotkey() else {
                log::warn!("ignoring {}: {:?} isn't a key name", cmd.label(), binding.key);
                failed.push(cmd);
                continue;
            };
            match manager.register(hotkey) {
                // The same combination another app already owns is a common
                // outcome, not a crash: keep the rest working and say which failed.
                Err(e) => {
                    log::warn!("could not register {} ({}): {e}", cmd.label(), binding.label());
                    failed.push(cmd);
                }
                Ok(()) => bound.push((hotkey.id(), cmd)),
            }
        }
        Some(Self { _manager: manager, bound, failed })
    }

    /// The shortcuts the system refused, for the settings dialog.
    pub fn failed(&self) -> &[HotkeyCmd] {
        &self.failed
    }

    pub fn poll(&self) -> Option<HotkeyCmd> {
        let mut events = PENDING.lock().map(|mut queue| std::mem::take(&mut *queue)).unwrap_or_default();
        // Without `install_wake` the crate still uses its own channel; drain it too.
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            events.push(event);
        }
        let mut pressed = None;
        // Read every event even after a hit: a stale release must not be read as
        // the next press.
        for event in events {
            if event.state != HotKeyState::Pressed || pressed.is_some() {
                continue;
            }
            pressed = self.bound.iter().find(|(id, _)| *id == event.id).map(|(_, cmd)| *cmd);
        }
        pressed
    }
}
