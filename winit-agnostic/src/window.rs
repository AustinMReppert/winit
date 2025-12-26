use std::fmt;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use cursor_icon::CursorIcon;
use rwh_06::{HasDisplayHandle, HasWindowHandle};
use dpi::{PhysicalInsets, PhysicalPosition, PhysicalSize, Position, Size};
use winit_core::cursor::Cursor;
use winit_core::error::RequestError;
use winit_core::icon::Icon;
use winit_core::monitor::{Fullscreen, MonitorHandle};
use winit_core::window::{CursorGrabMode, ImeCapabilities, ImeRequest, ImeRequestError, ResizeDirection, Theme, UserAttentionType, Window as RootWindow, WindowAttributes, WindowButtons, WindowId, WindowLevel};
use crate::{window_id, WindowAttributesAgnostic};
use crate::event_loop::{self, ActiveEventLoop};

pub(crate) struct Window(Arc<Mutex<WindowState>>);

static FOCUSED_WINDOW: AtomicUsize = AtomicUsize::new(0);

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window").finish_non_exhaustive()
    }
}


#[derive(Debug)]
struct WindowState {
    id: WindowId,
    title: String,
    scale_factor: f64,
    transparent: bool,
    surface_position: PhysicalPosition<i32>,
    resizeable: bool,
    enabled_buttons: WindowButtons,
    cursor: Cursor,
    level: WindowLevel,
    maximized: bool,
    cursor_positon: Position,
    content_protected: bool,
    theme: Option<Theme>,
    decorated: bool,
    icon: Option<Icon>,
    cursor_icon: Option<Icon>,
    cursor_visible: bool,
}

impl Window {
    pub(crate) fn new(
        event_loop: &ActiveEventLoop,
        w_attr: WindowAttributes,
    ) -> Result<Window, RequestError> {
        let mut w_attr = w_attr;
        let win_attributes = w_attr
            .platform
            .take()
            .and_then(|attrs| attrs.cast::<WindowAttributesAgnostic>().ok())
            .unwrap_or_default();

        Ok(Self {
            0: Arc::new(Mutex::new(WindowState::new()))
        })
    }
}

impl WindowState {

    pub fn new() -> Self {
        Self {
            id: window_id::next_window_id(),
            title: "Agnostic Window".to_owned(),
            scale_factor: 1.0,
            transparent: false,
            surface_position: PhysicalPosition::<i32>::new(0, 0),
            resizeable: true,
            enabled_buttons: WindowButtons::all(),
            cursor: Cursor::Icon(CursorIcon::Default),
            level: WindowLevel::Normal,
            maximized: false,
            cursor_positon: Position::Physical(PhysicalPosition::new(0, 0)),
            content_protected: false,
            theme: None,
            decorated: false,
            icon: None,
            cursor_icon: None,
            cursor_visible: false,
        }
    }

}

impl RootWindow for Window {
    fn id(&self) -> WindowId {
        self.0.lock().unwrap().id
    }

    fn scale_factor(&self) -> f64 {
        self.0.lock().unwrap().scale_factor
    }

    fn request_redraw(&self) {
    }

    fn pre_present_notify(&self) {
    }

    fn reset_dead_keys(&self) {
    }

    fn surface_position(&self) -> PhysicalPosition<i32> {
        self.0.lock().unwrap().surface_position
    }

    fn outer_position(&self) -> Result<PhysicalPosition<i32>, RequestError> {
        todo!()
    }

    fn set_outer_position(&self, position: Position) {
        todo!()
    }

    fn surface_size(&self) -> PhysicalSize<u32> {
        todo!()
    }

    fn request_surface_size(&self, size: Size) -> Option<PhysicalSize<u32>> {
        todo!()
    }

    fn outer_size(&self) -> PhysicalSize<u32> {
        todo!()
    }

    fn safe_area(&self) -> PhysicalInsets<u32> {
        todo!()
    }

    fn set_min_surface_size(&self, min_size: Option<Size>) {
        todo!()
    }

    fn set_max_surface_size(&self, max_size: Option<Size>) {
        todo!()
    }

    fn surface_resize_increments(&self) -> Option<PhysicalSize<u32>> {
        todo!()
    }

    fn set_surface_resize_increments(&self, increments: Option<Size>) {
        todo!()
    }

    fn set_title(&self, title: &str) {
        self.0.lock().unwrap().title = title.to_owned();
    }

    fn set_transparent(&self, transparent: bool) {
        self.0.lock().unwrap().transparent = transparent;
    }

    fn set_blur(&self, blur: bool) {
        todo!()
    }

    fn set_visible(&self, visible: bool) {
        todo!()
    }

    fn is_visible(&self) -> Option<bool> {
        todo!()
    }

    fn set_resizable(&self, resizable: bool) {
        self.0.lock().unwrap().resizeable = resizable;
    }

    fn is_resizable(&self) -> bool {
        self.0.lock().unwrap().resizeable
    }

    fn set_enabled_buttons(&self, buttons: WindowButtons) {
        todo!()
    }

    fn enabled_buttons(&self) -> WindowButtons {
        self.0.lock().unwrap().enabled_buttons
    }

    fn set_minimized(&self, minimized: bool) {
        todo!()
    }

    fn is_minimized(&self) -> Option<bool> {
        todo!()
    }

    fn set_maximized(&self, maximized: bool) {
        self.0.lock().unwrap().maximized = maximized;
    }

    fn is_maximized(&self) -> bool {
        self.0.lock().unwrap().maximized
    }

    fn set_fullscreen(&self, fullscreen: Option<Fullscreen>) {
        todo!()
    }

    fn fullscreen(&self) -> Option<Fullscreen> {
        todo!()
    }

    fn set_decorations(&self, decorations: bool) {
        todo!()
    }

    fn is_decorated(&self) -> bool {
        self.0.lock().unwrap().decorated
    }

    fn set_window_level(&self, level: WindowLevel) {
        self.0.lock().unwrap().level = level;
    }

    fn set_window_icon(&self, window_icon: Option<Icon>) {
        self.0.lock().unwrap().icon = window_icon;
    }

    fn request_ime_update(&self, request: ImeRequest) -> Result<(), ImeRequestError> {
        todo!()
    }

    fn ime_capabilities(&self) -> Option<ImeCapabilities> {
        todo!()
    }

    fn focus_window(&self) {
        FOCUSED_WINDOW.store(self.0.lock().unwrap().id.into_raw(), Ordering::Release);
    }

    fn has_focus(&self) -> bool {
        match FOCUSED_WINDOW.load(Ordering::Acquire) {
            0 => false,
            id => id == self.0.lock().unwrap().id.into_raw(),
        }
    }

    fn request_user_attention(&self, request_type: Option<UserAttentionType>) {
    }

    fn set_theme(&self, theme: Option<Theme>) {
        self.0.lock().unwrap().theme = theme;
    }

    fn theme(&self) -> Option<Theme> {
        self.0.lock().unwrap().theme
    }

    fn set_content_protected(&self, protected: bool) {
        self.0.lock().unwrap().content_protected = protected;
    }

    fn title(&self) -> String {
        self.0.lock().unwrap().title.clone()
    }

    fn set_cursor(&self, cursor: Cursor) {
        self.0.lock().unwrap().cursor = cursor;
    }

    fn set_cursor_position(&self, position: Position) -> Result<(), RequestError> {
        self.0.lock().unwrap().cursor_positon = position;
        Ok(())
    }

    fn set_cursor_grab(&self, mode: CursorGrabMode) -> Result<(), RequestError> {
        todo!()
    }

    fn set_cursor_visible(&self, visible: bool) {
        self.0.lock().unwrap().cursor_visible = visible;
    }

    fn drag_window(&self) -> Result<(), RequestError> {
        todo!()
    }

    fn drag_resize_window(&self, direction: ResizeDirection) -> Result<(), RequestError> {
        todo!()
    }

    fn show_window_menu(&self, position: Position) {
        // Intentionally a no-op
    }

    fn set_cursor_hittest(&self, hittest: bool) -> Result<(), RequestError> {
        todo!()
    }

    fn current_monitor(&self) -> Option<MonitorHandle> {
        todo!()
    }

    fn available_monitors(&self) -> Box<dyn Iterator<Item=MonitorHandle>> {
        todo!()
    }

    fn primary_monitor(&self) -> Option<MonitorHandle> {
        todo!()
    }

    fn rwh_06_display_handle(&self) -> &dyn HasDisplayHandle {
        todo!()
    }

    fn rwh_06_window_handle(&self) -> &dyn HasWindowHandle {
        todo!()
    }
}