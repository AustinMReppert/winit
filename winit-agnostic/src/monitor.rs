use std::collections::VecDeque;
use std::{mem, ptr};
use std::num::{NonZeroU16, NonZeroU32};
use std::sync::{Arc, Mutex};
use dpi::{PhysicalPosition, PhysicalSize};
use winit_core::monitor::{MonitorHandleProvider, VideoMode};
use crate::window_id::next_monitor_id;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MonitorHandle(Arc<MonitorState>);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct MonitorState {
    name: String,
    id: u64,
    position: PhysicalPosition<i32>,
    current_mode: VideoMode,
}

impl MonitorState {

    fn new() -> Self {
        Self {
            name: "monitor".to_string(),
            id: next_monitor_id(),
            position: PhysicalPosition::new(0, 0),
            current_mode: VideoMode::new(PhysicalSize::new(1920, 1080), Some(NonZeroU16::new(16).unwrap()), Some(NonZeroU32::new(60).unwrap())),
        }
    }

}

impl MonitorHandle {

    pub fn new() -> Self {
        Self {
            0: Arc::new(MonitorState::new()),
        }
    }

}

impl MonitorHandleProvider for MonitorHandle {
    fn id(&self) -> u128 {
        self.native_id() as _
    }

    fn native_id(&self) -> u64 {
        self.0.id
    }

    fn name(&self) -> Option<std::borrow::Cow<'_, str>> {
        Some(self.0.name.as_str().into())
    }

    fn position(&self) -> Option<PhysicalPosition<i32>> {
        Some(self.0.position)
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }

    fn current_video_mode(&self) -> Option<winit_core::monitor::VideoMode> {
        Some(self.0.current_mode)
    }

    fn video_modes(&self) -> Box<dyn Iterator<Item = VideoMode>> {
        let modes = vec![self.0.current_mode];
        Box::new(modes.into_iter())
    }
}


#[derive(Clone)]
pub struct VideoModeHandle {

}

pub fn available_monitors() -> VecDeque<MonitorHandle> {
    let mut monitors: VecDeque<MonitorHandle> = VecDeque::new();
    monitors
}

pub fn primary_monitor() -> MonitorHandle {
    MonitorHandle::new()
}