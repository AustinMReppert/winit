//! Simple winit window example.

use std::error::Error;
use std::thread;
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoopBuilder;
use winit::event_loop::{ActiveEventLoop, EventLoop};
#[cfg(web_platform)]
use winit::platform::web::WindowAttributesWeb;
use winit::window::{Window, WindowAttributes, WindowId};
use winit_agnostic::{Event, EventLoopBuilderExtAgonstic, WindowAttributesAgnostic};
use winit_core::event_loop::ControlFlow;
use winit_core::window::Theme;

#[path = "util/fill.rs"]
mod fill;
#[path = "util/tracing.rs"]
mod tracing;

#[derive(Default, Debug)]
struct App {
    window: Option<Box<dyn Window>>,
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        #[cfg(not(web_platform))]
        let window_attributes = WindowAttributes::default();
        #[cfg(web_platform)]
        let window_attributes = WindowAttributes::default()
            .with_platform_attributes(Box::new(WindowAttributesWeb::default().with_append(true)));
        #[cfg(feature = "agnostic")]
        let window_attributes = WindowAttributes::default()
            .with_platform_attributes(Box::new(WindowAttributesAgnostic::new()));
        self.window = match event_loop.create_window(window_attributes) {
            Ok(window) => Some(window),
            Err(err) => {
                eprintln!("error creating window: {err}");
                event_loop.exit();
                return;
            },
        }
    }

    fn window_event(&mut self, event_loop: &dyn ActiveEventLoop, _: WindowId, event: WindowEvent) {
        println!("{event:?}");
        match event {
            WindowEvent::CloseRequested => {
                println!("Close was requested; stopping");
                event_loop.exit();
            },
            WindowEvent::SurfaceResized(_) => {
                self.window.as_ref().expect("resize event without a window").request_redraw();
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                let window = self.window.as_ref().expect("redraw request without a window");

                // Notify that you're about to draw.
                window.pre_present_notify();

                // Draw.
                fill::fill_window(window.as_ref());

                // For contiguous redraw loop you can request a redraw from here.
                // window.request_redraw();
            },
            WindowEvent::ThemeChanged(_) => {
                println!("artificial event dispatched");
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(web_platform)]
    console_error_panic_hook::set_once();

    tracing::init();
    // ::<winit::event::WindowEvent>
    let (sender, receiver) = std::sync::mpsc::channel::<winit::platform::platform::Event>();

    //#[cfg(feature = "agnostic")]
    use winit::platform::platform::WindowAttributesAgnostic;
    let event_loop = EventLoopBuilder::default().with_receiver(receiver).build().unwrap();

    std::thread::spawn(move || {
        loop {
            sender.send(Event::Window {
                window_id: WindowId::from_raw(1),
                event: WindowEvent::ThemeChanged(Theme::Dark),
            }).expect("TODO: panic message");
            thread::sleep(Duration::from_secs(10));
        }
    });

    // For alternative loop run options see `pump_events` and `run_on_demand` examples.
    event_loop.run_app(App::default())?;

    Ok(())
}
