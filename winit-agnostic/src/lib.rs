mod window;
mod window_id;
mod event_loop;
mod keyboard;
mod runner;
mod monitor;

use std::ffi::c_void;
use winit_core::window::PlatformWindowAttributes;
pub use crate::runner::Event;
pub use self::event_loop::{EventLoop, PlatformSpecificEventLoopAttributes};
pub use self::keyboard::{physicalkey_to_scancode, scancode_to_physicalkey};
pub use self::monitor::{MonitorHandle, VideoModeHandle};

#[derive(Clone, Debug)]
pub struct WindowAttributesAgnostic {

}

impl Default for WindowAttributesAgnostic {
    fn default() -> Self {
        Self {}
    }
}

impl WindowAttributesAgnostic {

    pub fn new() -> Self {
        Self::default()
    }

}

impl PlatformWindowAttributes for WindowAttributesAgnostic {
    fn box_clone(&self) -> Box<dyn PlatformWindowAttributes> {
        Box::from(self.clone())
    }
}

/// Additional methods on `EventLoop` that are specific to Agnostic.
pub trait EventLoopBuilderExtAgonstic {
    /// Whether to allow the event loop to be created off of the main thread.
    ///
    /// By default, the window is only allowed to be created on the main
    /// thread, to make platform compatibility easier.
    ///
    /// # `Window` caveats
    ///
    /// Note that any `Window` created on the new thread will be destroyed when the thread
    /// terminates. Attempting to use a `Window` after its parent thread terminates has
    /// unspecified, although explicitly not undefined, behavior.
    fn with_receiver(&mut self, any_thread: std::sync::mpsc::Receiver<Event>) -> &mut Self;
}