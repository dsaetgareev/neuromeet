use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Clone)]
pub struct DeviceState {
    pub(super) destroy: Arc<AtomicBool>,
    pub(super) enabled: Arc<AtomicBool>,
    pub(super) selected: Option<String>,
    pub(super) switching: Arc<AtomicBool>,
}

impl PartialEq for DeviceState {
    fn eq(&self, other: &Self) -> bool {
        self.destroy.load(Ordering::Acquire) == other.destroy.load(Ordering::Acquire) &&
        self.enabled.load(Ordering::Acquire) == other.enabled.load(Ordering::Acquire) &&
        self.selected == other.selected && 
        self.switching.load(Ordering::Acquire) == other.switching.load(Ordering::Acquire)
    }
}

impl DeviceState {
    pub fn new() -> Self {
        Self {
            destroy: Arc::new(AtomicBool::new(false)),
            enabled: Arc::new(AtomicBool::new(true)),
            selected: None,
            switching: Arc::new(AtomicBool::new(false)),
        }
    }

    // Sets the enabled bit to a given value, returning true if it was a change.
    pub fn set_enabled(&mut self, value: bool) -> bool {
        if value != self.enabled.load(Ordering::Acquire) {
            self.enabled.store(value, Ordering::Release);
            true
        } else {
            false
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    pub fn select(&mut self, device: String) -> bool {
        match self.selected.clone() {
            Some(selected) => {
                if selected.eq(&device) {
                    false                    
                } else {
                    self.selected = Some(device);
                    if self.is_enabled() {
                        self.switching.store(true, Ordering::Release);
                        true
                    } else {
                        false
                    }   
                }
            },
            None => {
                self.selected = Some(device);
                if self.is_enabled() {
                    self.switching.store(true, Ordering::Release);
                    true
                } else {
                    false
                }
            },
        }
    }

}
