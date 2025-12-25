use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use winit_core::window::WindowId;

static WINDOW_COUNTER: AtomicUsize = AtomicUsize::new(0);
static MONITOR_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn next_window_id() -> WindowId {
    WindowId::from_raw(WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed))
}

pub fn next_monitor_id() -> u64 {
    MONITOR_COUNTER.fetch_add(1, Ordering::Relaxed)
}