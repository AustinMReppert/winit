use crate::window::Window;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{fmt, mem};
use winit_core::application::ApplicationHandler;
use winit_core::cursor::{CustomCursor, CustomCursorSource};
use winit_core::error::{EventLoopError, RequestError};
use winit_core::event_loop::pump_events::PumpStatus;
use winit_core::window::{PlatformWindowAttributes, Theme, Window as CoreWindow, WindowAttributes};

use winit_core::monitor::MonitorHandle as CoreMonitorHandle;

use crate::monitor;
use crate::runner::{Event, EventLoopRunner};
use winit_core::event_loop::{
    ActiveEventLoop as RootActiveEventLoop, ControlFlow, DeviceEvents,
    EventLoopProxy as RootEventLoopProxy, EventLoopProxyProvider,
    OwnedDisplayHandle as CoreOwnedDisplayHandle,
};

/// Set upper limit for waiting time to avoid overflows.
/// I chose 50 days as a limit because it is used in dur2timeout.
const FIFTY_DAYS: Duration = Duration::from_secs(50_u64 * 24 * 60 * 60);
/// Waitable timers use 100 ns intervals to indicate due time.
/// <https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimer#parameters>
/// And there is no point waiting using other ways for such small timings
/// because they are even less precise (can overshoot by few ms).
const MIN_WAIT: Duration = Duration::from_nanos(100);

pub struct EventLoop {
    runner: Arc<EventLoopRunner>,
}

impl fmt::Debug for EventLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventLoop").finish_non_exhaustive()
    }
}

pub struct PlatformSpecificEventLoopAttributes {
    pub events: Option<std::sync::mpsc::Receiver<Event>>
}

impl fmt::Debug for PlatformSpecificEventLoopAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlatformSpecificEventLoopAttributes")
            .finish_non_exhaustive()
    }
}

impl Default for PlatformSpecificEventLoopAttributes {
    fn default() -> Self {
        Self { events: None }
    }
}

impl PartialEq for PlatformSpecificEventLoopAttributes {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for PlatformSpecificEventLoopAttributes {}

impl std::hash::Hash for PlatformSpecificEventLoopAttributes {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
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

        let runner_shared = Arc::new(EventLoopRunner::new(attributes.events.take()));

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
        self.runner.clear_exit();

        // SAFETY: The resetter is not leaked.
        let _app_resetter = unsafe { self.runner.set_app(&mut app) };

        let exit_code = loop {
            self.wait_for_messages(None);
            // wait_for_messages calls user application before and after waiting
            // so it may have decided to exit.
            if let Some(code) = self.exit_code() {
                break code;
            }

            self.dispatch_peeked_messages();

            if let Some(code) = self.exit_code() {
                break code;
            }
        };

        self.runner.loop_destroyed();

        self.runner.reset_runner();

        if exit_code == 0 { Ok(()) } else { Err(EventLoopError::ExitFailure(exit_code)) }
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
        // We aim to be consistent with the MacOS backend which has a RunLoop
        // observer that will dispatch AboutToWait when about to wait for
        // events, and NewEvents after the RunLoop wakes up.
        //
        // We emulate similar behaviour by treating `MsgWaitForMultipleObjectsEx` as our wait
        // point and wake up point (when it returns) and we drain all other
        // pending messages via `PeekMessage` until we come back to "wait" via
        // `MsgWaitForMultipleObjectsEx`.
        //
        self.runner.prepare_wait();/*wait_for_messages_impl(
            &mut self.high_resolution_timer,
            self.runner.control_flow(),
            timeout,
        );*/

        /*let timeout = {
            let control_flow_timeout = match self.runner.control_flow() {
                ControlFlow::Wait => None,
                ControlFlow::Poll => Some(Duration::ZERO),
                ControlFlow::WaitUntil(wait_deadline) => {
                    let start = Instant::now();
                    Some(wait_deadline.saturating_duration_since(start))
                },
            };
            let timeout = min_timeout(timeout, control_flow_timeout);
            if timeout == Some(Duration::ZERO) {
                // Do not wait if we don't have time.
                return;
            }
            // Now we decided to wait so need to do some clamping
            // to avoid problems with overflow and calling WinAPI with invalid parameters.
            timeout
                .map(|t| t.min(FIFTY_DAYS))
                // If timeout is less than minimally supported by Windows,
                // increase it to that minimum. Who want less than microsecond delays anyway?
                .map(|t| t.max(MIN_WAIT))
        };*/

        if let Some(events) = self.runner.events() {
            if let Some(timeout) = timeout {
                println!("Waiting with timeout");
                if let Ok(event) = events.recv_timeout(timeout) {
                    self.runner.send_event(event);
                }
            } else {
                println!("Waiting");
                if let Ok(event) = events.recv() {
                    self.runner.send_event(event);
                }
            }
        }
        // Before we potentially exit, make sure to consistently emit an event for the wake up
        self.runner.wakeup();
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
    pub(crate) fn from_ref(shared_runner: &Arc<EventLoopRunner>) -> &Self {
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
        Ok(Box::new(Window::new(self, window_attributes)?))
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

    fn listen_device_events(&self, allowed: DeviceEvents) {
    }

    fn system_theme(&self) -> Option<Theme> {
        None
    }

    fn set_control_flow(&self, control_flow: ControlFlow) {
        self.0.set_control_flow(control_flow);
    }

    fn control_flow(&self) -> ControlFlow {
        self.0.control_flow()
    }

    fn exit(&self) {
        self.0.set_exit_code(0)
    }

    fn exiting(&self) -> bool {
        self.0.exit_code().is_some()
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

/// Returns the minimum `Option<Duration>`, taking into account that `None`
/// equates to an infinite timeout, not a zero timeout (so can't just use
/// `Option::min`)
fn min_timeout(a: Option<Duration>, b: Option<Duration>) -> Option<Duration> {
    a.map_or(b, |a_timeout| b.map_or(Some(a_timeout), |b_timeout| Some(a_timeout.min(b_timeout))))
}
