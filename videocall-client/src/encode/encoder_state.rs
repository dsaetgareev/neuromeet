// use std::{rc::Rc, cell::RefCell};



// #[derive(Clone, PartialEq)]
// pub struct EncoderState {
//     pub(super) destroy: Rc<RefCell<bool>>,
//     pub(super) enabled: Rc<RefCell<bool>>,
//     pub(super) selected: Option<String>,
//     pub(super) switching: Rc<RefCell<bool>>,
//     pub(super) is_first: Rc<RefCell<bool>>,
// }

// impl EncoderState {
//     pub fn new() -> Self {
//         Self {
//             destroy: Rc::new(RefCell::new(false)),
//             enabled: Rc::new(RefCell::new(true)),
//             selected: None,
//             switching: Rc::new(RefCell::new(false)),
//             is_first: Rc::new(RefCell::new(true)),
//         }
//     }

//     pub fn set_enabled(&mut self, value: bool) -> bool {
//         if value != self.is_enabled() {
//             *self.enabled.as_ref().borrow_mut() = value;
//             true
//         } else {
//             false
//         }
//     }

//     pub fn is_enabled(&self) -> bool {
//         *self.enabled.borrow()
//     }

//     // pub fn is_first(&self) -> bool {
//     //     *self.is_first.borrow()
//     // }

//     // pub fn set_first(&mut self, is_first: bool) {
//     //     self.is_first = Rc::new(RefCell::new(is_first));
//     // }

//     pub fn select(&mut self, device: String) -> bool {
//         self.selected = Some(device);
//         if self.is_enabled() {
//             *self.switching.as_ref().borrow_mut() = true;
//             true
//         } else {
//             false
//         }
//     }

//     pub fn stop(&mut self) {
//         *self.destroy.as_ref().borrow_mut() = true;
//     }
// }

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

//
// EncoderState struct contains state variables that are common among the encoders, and the logic
// for working with them.
//

#[derive(Clone)]
pub struct EncoderState {
    pub(super) destroy: Arc<AtomicBool>,
    pub(super) enabled: Arc<AtomicBool>,
    pub(super) selected: Option<String>,
    pub(super) switching: Arc<AtomicBool>,
}

impl PartialEq for EncoderState {
    fn eq(&self, other: &Self) -> bool {
        self.destroy.load(Ordering::Acquire) == other.destroy.load(Ordering::Acquire) &&
        self.enabled.load(Ordering::Acquire) == other.enabled.load(Ordering::Acquire) &&
        self.selected == other.selected && 
        self.switching.load(Ordering::Acquire) == other.switching.load(Ordering::Acquire)
    }
}

impl EncoderState {
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
        self.selected = Some(device);
        if self.is_enabled() {
            self.switching.store(true, Ordering::Release);
            true
        } else {
            false
        }
    }

    pub fn stop(&mut self) {
        self.destroy.store(true, Ordering::Release);
    }
}
