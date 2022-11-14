use winit::event_loop::ControlFlow;

use engine::GameEngine;
use winit::event::{Event, WindowEvent};

mod engine;
fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let game_engine = GameEngine::init(window).unwrap();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        Event::MainEventsCleared => {
            game_engine.window.request_redraw();
        }

        _ => {}
    });
}
