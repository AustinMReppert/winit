use std::cell::Cell;
use std::{fmt, mem};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use dpi::PhysicalSize;
use winit_core::application::ApplicationHandler;
use winit_core::event_loop::ControlFlow;
use winit_core::window::WindowId;


use winit_core::event::{DeviceEvent, DeviceId, StartCause, SurfaceSizeWriter, WindowEvent};
use winit_core::event_loop::ActiveEventLoop as RootActiveEventLoop;

type EventHandler = Cell<Option<&'static mut (dyn ApplicationHandler + 'static)>>;


pub(crate) struct EventLoopRunner {
    runner_state: Cell<RunnerState>,
    exit: Cell<Option<i32>>,
    control_flow: Cell<ControlFlow>,
    event_handler: Arc<EventHandler>,
    events: Option<std::sync::mpsc::Receiver<Event>>,
    last_events_cleared: Cell<Instant>,
}

pub enum Event {
    Device { device_id: DeviceId, event: DeviceEvent },
    Window { window_id: WindowId, event: WindowEvent },
    //BufferedScaleFactorChanged(Arc<WindowState>, f64, PhysicalSize<u32>),
    // FIXME(madsmtm): Coalesce these into a flag (or similar) instead of handling them as events.
    // https://github.com/rust-windowing/winit/pull/3687
    WakeUp,
}

/// See `move_state_to` function for details on how the state loop works.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RunnerState {
    /// The event loop has just been created, and an `Init` event must be sent.
    Uninitialized,
    /// The event loop is idling.
    Idle,
    /// The event loop is handling the OS's events and sending them to the user's callback.
    /// `NewEvents` has been sent, and `AboutToWait` hasn't.
    HandlingMainEvents,
    /// The event loop has been destroyed. No other events will be emitted.
    Destroyed,
}

impl EventLoopRunner {

    pub(crate) fn new(events: Option<std::sync::mpsc::Receiver<Event>>) -> Self {
        Self {
            runner_state: Cell::new(RunnerState::Uninitialized),
            exit: Cell::new(None),
            control_flow: Cell::new(ControlFlow::Poll),
            event_handler: Arc::new(Cell::new(None)),
            events,
            last_events_cleared: Cell::new(Instant::now()),
        }
    }

}

impl fmt::Debug for EventLoopRunner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventLoopRunner")
            .finish_non_exhaustive()
    }
}

/// Event dispatch functions.
impl EventLoopRunner {
    pub fn set_exit_code(&self, code: i32) {
        self.exit.set(Some(code))
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit.get()
    }

    pub(crate) fn prepare_wait(self: &Arc<Self>) {
        self.move_state_to(RunnerState::Idle);
    }

    pub(crate) fn wakeup(self: &Arc<Self>) {
        self.move_state_to(RunnerState::HandlingMainEvents);
    }

    pub(crate) fn send_event(self: &Rc<Self>, event: Event) {
    }

    pub(crate) fn loop_destroyed(self: &Arc<Self>) {
        self.move_state_to(RunnerState::Destroyed);
    }

    fn call_event_handler(
        self: &Arc<Self>,
        closure: impl FnOnce(&mut dyn ApplicationHandler, &dyn RootActiveEventLoop),
    ) {

    }

    pub fn clear_exit(&self) {
        self.exit.set(None);
    }

    pub(crate) fn reset_runner(&self) {
        self.exit.set(None);
    }

    fn dispatch_buffered_events(self: &Rc<Self>) {
    }

    pub fn events(&self) -> Option<&std::sync::mpsc::Receiver<Event>> {
        self.events.as_ref()
    }

    /// Associate the application's event handler with the runner.
    ///
    /// # Safety
    ///
    /// The returned type must not be leaked (as that would allow the application to be associated
    /// with the runner for too long).
    pub(crate) unsafe fn set_app<'app>(
        &self,
        app: &'app mut (dyn ApplicationHandler + 'app),
    ) -> impl Drop + 'app {
        // Erase app lifetime, to allow storing on the event loop runner.
        //
        // SAFETY: Caller upholds that the lifetime of the closure is upheld, by not dropping the
        // return type which resets it.
        let f = unsafe {
            mem::transmute::<
                &'app mut (dyn ApplicationHandler + 'app),
                &'static mut (dyn ApplicationHandler + 'static),
            >(app)
        };

        let old_event_handler = self.event_handler.replace(Some(f));

        assert!(old_event_handler.is_none());

        struct Resetter(Arc<EventHandler>);

        impl Drop for Resetter {
            fn drop(&mut self) {
                self.0.set(None);
            }
        }

        Resetter(self.event_handler.clone())
    }

    /// Dispatch control flow events (`NewEvents`, `AboutToWait`, and
    /// `LoopExiting`) as necessary to bring the internal `RunnerState` to the
    /// new runner state.
    ///
    /// The state transitions are defined as follows:
    ///
    /// ```text
    ///    Uninitialized
    ///          |
    ///          V
    ///        Idle
    ///       ^    |
    ///       |    V
    /// HandlingMainEvents
    ///         |
    ///         V
    ///     Destroyed
    /// ```
    ///
    /// Attempting to transition back to `Uninitialized` will result in a panic. Attempting to
    /// transition *from* `Destroyed` will also result in a panic. Transitioning to the current
    /// state is a no-op. Even if the `new_runner_state` isn't the immediate next state in the
    /// runner state machine (e.g. `self.runner_state == HandlingMainEvents` and
    /// `new_runner_state == Idle`), the intermediate state transitions will still be executed.
    fn move_state_to(self: &Arc<Self>, new_runner_state: RunnerState) {
        use RunnerState::{Destroyed, HandlingMainEvents, Idle, Uninitialized};

        match (self.runner_state.replace(new_runner_state), new_runner_state) {
            (Uninitialized, Uninitialized)
            | (Idle, Idle)
            | (HandlingMainEvents, HandlingMainEvents)
            | (Destroyed, Destroyed) => (),

            // State transitions that initialize the event loop.
            (Uninitialized, HandlingMainEvents) => {
                self.call_new_events(true);
            },
            (Uninitialized, Idle) => {
                self.call_new_events(true);
                self.call_event_handler(|app, event_loop| app.about_to_wait(event_loop));
                self.last_events_cleared.set(Instant::now());
            },
            (Uninitialized, Destroyed) => {
                self.call_new_events(true);
                self.call_event_handler(|app, event_loop| app.about_to_wait(event_loop));
                self.last_events_cleared.set(Instant::now());
            },
            (_, Uninitialized) => panic!("cannot move state to Uninitialized"),

            // State transitions that start the event handling process.
            (Idle, HandlingMainEvents) => {
                self.call_new_events(false);
            },
            (Idle, Destroyed) => {},

            (HandlingMainEvents, Idle) => {
                // This is always the last event we dispatch before waiting for new events
                self.call_event_handler(|app, event_loop| app.about_to_wait(event_loop));
                self.last_events_cleared.set(Instant::now());
            },
            (HandlingMainEvents, Destroyed) => {
                self.call_event_handler(|app, event_loop| app.about_to_wait(event_loop));
                self.last_events_cleared.set(Instant::now());
            },

            (Destroyed, _) => panic!("cannot move state from Destroyed"),
        }
    }

    fn call_new_events(self: &Arc<Self>, init: bool) {
    }

    pub fn control_flow(&self) -> ControlFlow {
        self.control_flow.get()
    }

    pub fn set_control_flow(&self, control_flow: ControlFlow) {
        self.control_flow.set(control_flow)
    }
}

impl Event {
    /// Mark ScaleFactorChanged as being buffered (which forces us to re-handle when the user set a
    /// new size).
    pub fn buffer_scale_factor(self) -> Self {
        self
    }

    pub fn dispatch_event(
        self,
        app: &mut dyn ApplicationHandler,
        event_loop: &dyn RootActiveEventLoop,
    ) {
    }

    pub(crate) fn reset_runner(&self) {
    }
}
