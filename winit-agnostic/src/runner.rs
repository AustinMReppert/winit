use std::cell::Cell;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit_core::application::ApplicationHandler;
use winit_core::event_loop::ControlFlow;
use winit_core::window::WindowId;


use winit_core::event::{StartCause, SurfaceSizeWriter, WindowEvent};
use winit_core::event_loop::ActiveEventLoop as RootActiveEventLoop;
pub(crate) struct EventLoopRunner {
    runner_state: Cell<RunnerState>,
    exit: Cell<Option<i32>>,
    control_flow: Cell<ControlFlow>,
}

pub(crate) enum Event {

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

    pub(crate) fn new() -> Self {
        Self {
            runner_state: Cell::new(RunnerState::Uninitialized),
            exit: Cell::new(None),
            control_flow: Cell::new(ControlFlow::Poll),
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

    pub(crate) fn prepare_wait(self: &Rc<Self>) {
        self.move_state_to(RunnerState::Idle);
    }

    pub(crate) fn wakeup(self: &Rc<Self>) {
        self.move_state_to(RunnerState::HandlingMainEvents);
    }

    pub(crate) fn send_event(self: &Rc<Self>, event: Event) {
    }

    pub(crate) fn loop_destroyed(self: &Rc<Self>) {
        self.move_state_to(RunnerState::Destroyed);
    }

    fn call_event_handler(
        self: &Rc<Self>,
        closure: impl FnOnce(&mut dyn ApplicationHandler, &dyn RootActiveEventLoop),
    ) {

    }

    fn dispatch_buffered_events(self: &Rc<Self>) {
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
    fn move_state_to(self: &Rc<Self>, new_runner_state: RunnerState) {
    }

    fn call_new_events(self: &Rc<Self>, init: bool) {
    }

    pub fn control_flow(&self) -> ControlFlow {
        self.control_flow.get()
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
