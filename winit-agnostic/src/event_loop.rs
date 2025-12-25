use std::rc::Rc;
use std::{fmt, mem, panic, ptr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use winit_core::application::ApplicationHandler;
use winit_core::cursor::{CustomCursor, CustomCursorSource};
use winit_core::error::{EventLoopError, NotSupportedError, RequestError};
use winit_core::event_loop::pump_events::PumpStatus;
use winit_core::window::{Theme, Window as CoreWindow, WindowAttributes, WindowId};
use crate::window::{Window};

use winit_core::monitor::{Fullscreen, MonitorHandle as CoreMonitorHandle};

use winit_core::event_loop::{
    ActiveEventLoop as RootActiveEventLoop, ControlFlow, DeviceEvents,
    EventLoopProxy as RootEventLoopProxy, EventLoopProxyProvider,
    OwnedDisplayHandle as CoreOwnedDisplayHandle,
};
use crate::monitor;
use crate::runner::EventLoopRunner;

pub struct EventLoop {
    runner: Arc<EventLoopRunner>,
}

impl fmt::Debug for EventLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventLoop").finish_non_exhaustive()
    }
}

pub struct PlatformSpecificEventLoopAttributes {
}

impl fmt::Debug for PlatformSpecificEventLoopAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlatformSpecificEventLoopAttributes")
            .finish_non_exhaustive()
    }
}

impl Default for PlatformSpecificEventLoopAttributes {
    fn default() -> Self {
        Self { }
    }
}

impl PartialEq for PlatformSpecificEventLoopAttributes {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}
impl Eq for PlatformSpecificEventLoopAttributes {}

impl std::hash::Hash for PlatformSpecificEventLoopAttributes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    }
}

impl EventLoop {
    pub fn new(
        attributes: &mut PlatformSpecificEventLoopAttributes,
    ) -> Result<Self, EventLoopError> {
        static EVENT_LOOP_CREATED: AtomicBool = AtomicBool::new(false);
        if EVENT_LOOP_CREATED.swap(true, Ordering::Relaxed) {
            // For better cross-platformness.
            return Err(EventLoopError::RecreationAttempt);
        }

        let runner_shared = Arc::new(EventLoopRunner::new());

        Ok(EventLoop {
            runner: runner_shared,
        })
    }

    pub fn window_target(&self) -> &dyn RootActiveEventLoop {
        ActiveEventLoop::from_ref(&self.runner)
    }

    pub fn run_app_on_demand<A: ApplicationHandler>(
        &mut self,
        mut app: A,
    ) -> Result<(), EventLoopError> {
        Ok(())
    }

    pub fn pump_app_events<A: ApplicationHandler>(
        &mut self,
        timeout: Option<Duration>,
        mut app: A,
    ) -> PumpStatus {
        PumpStatus::Continue
    }

    /// Waits until new event messages arrive to be peeked.
    /// Doesn't peek messages itself.
    ///
    /// Parameter timeout is optional. This method would wait for the smaller timeout
    /// between the argument and a timeout from control flow.
    fn wait_for_messages(&mut self, timeout: Option<Duration>) {
    }

    /// Dispatch all queued messages via `PeekMessageW`
    fn dispatch_peeked_messages(&mut self) {
    }

    fn exit_code(&self) -> Option<i32> {
        self.runner.exit_code()
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct ActiveEventLoop(pub Arc<EventLoopRunner>);

impl ActiveEventLoop {
    fn from_ref(shared_runner: &Arc<EventLoopRunner>) -> &Self {
        // SAFETY: `ActiveEventLoop` is `#[repr(transparent)]` over `Rc<EventLoopRunner>`.
        // FIXME(madsmtm): Implement `ActiveEventLoop` for `Rc<EventLoopRunner>` directly.
        unsafe { mem::transmute::<&Arc<EventLoopRunner>, &Self>(shared_runner) }
    }


}

impl RootActiveEventLoop for ActiveEventLoop {
    fn create_proxy(&self) -> RootEventLoopProxy {
        let event_loop_proxy = EventLoopProxy {  };
        RootEventLoopProxy::new(Arc::new(event_loop_proxy))
    }

    fn create_window(
        &self,
        window_attributes: WindowAttributes,
    ) -> Result<Box<dyn CoreWindow>, RequestError> {
        Ok(Box::new(Window::new()))
    }

    fn create_custom_cursor(
        &self,
        source: CustomCursorSource,
    ) -> Result<CustomCursor, RequestError> {
        Err(RequestError::Ignored)
    }

    fn available_monitors(&self) -> Box<dyn Iterator<Item = CoreMonitorHandle>> {
        Box::new(
            monitor::available_monitors()
                .into_iter()
                .map(|monitor| CoreMonitorHandle(Arc::new(monitor))),
        )
    }

    fn primary_monitor(&self) -> Option<CoreMonitorHandle> {
        Some(CoreMonitorHandle(Arc::new(monitor::primary_monitor())))
    }

    fn exiting(&self) -> bool {
        self.0.exit_code().is_some()
    }

    fn system_theme(&self) -> Option<Theme> {
        None
    }

    fn listen_device_events(&self, allowed: DeviceEvents) {
    }

    fn set_control_flow(&self, control_flow: ControlFlow) {
    }

    fn control_flow(&self) -> ControlFlow {
        self.0.control_flow()
    }

    fn exit(&self) {
        self.0.set_exit_code(0)
    }

    fn owned_display_handle(&self) -> CoreOwnedDisplayHandle {
        CoreOwnedDisplayHandle::new(Arc::new(OwnedDisplayHandle))
    }

    fn rwh_06_handle(&self) -> &dyn rwh_06::HasDisplayHandle {
        self
    }
}

impl rwh_06::HasDisplayHandle for ActiveEventLoop {
    fn display_handle(&self) -> Result<rwh_06::DisplayHandle<'_>, rwh_06::HandleError> {
        let raw = rwh_06::RawDisplayHandle::Windows(rwh_06::WindowsDisplayHandle::new());
        unsafe { Ok(rwh_06::DisplayHandle::borrow_raw(raw)) }
    }
}

#[derive(Clone)]
pub(crate) struct OwnedDisplayHandle;

impl rwh_06::HasDisplayHandle for OwnedDisplayHandle {
    fn display_handle(&self) -> Result<rwh_06::DisplayHandle<'_>, rwh_06::HandleError> {
        let raw = rwh_06::RawDisplayHandle::Windows(rwh_06::WindowsDisplayHandle::new());
        unsafe { Ok(rwh_06::DisplayHandle::borrow_raw(raw)) }
    }
}

#[derive(Debug)]
pub struct EventLoopProxy {
}

impl EventLoopProxyProvider for EventLoopProxy {
    fn wake_up(&self) {
    }
}
