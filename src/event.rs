// Handling window events, such as:
// - Resizing request
// - Mouse input
// - Keyboard input
// - Suspending and resuming requests
// - Closing request

use glium::glutin::dpi::{PhysicalSize};
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::ControlFlow;
use glium::glutin::window::WindowId;
use log::{info};

pub fn on_window_event(_display: &mut glium::Display, control_flow: &mut ControlFlow, window_id: WindowId, window_event: WindowEvent) {
   match window_event {
      WindowEvent::CloseRequested => {
         *control_flow = ControlFlow::Exit;
         info!("CloseRequested {:?}", window_id);
      },
      WindowEvent::Resized(PhysicalSize{..}) => {
         info!("Resized");
      },
      _ => (),
   }
}

pub fn on_resume(_display: &mut glium::Display, _control_flow: &mut ControlFlow) {
   info!("Resumed");
}

pub fn on_suspend(_display: &mut glium::Display, control_flow: &mut ControlFlow) {
   info!("Suspended");
   *control_flow = ControlFlow::Wait;
}

pub fn on_redraw_requested(_display: &mut glium::Display, control_flow: &mut ControlFlow) {
   let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
   // *control_flow = ControlFlow::WaitUntil(next_frame_time);
}

pub fn handle_event(event: Event<()>, display: &mut glium::Display, control_flow: &mut ControlFlow) -> bool {
   let mut do_redraw = false;
   match event {
       Event::WindowEvent{window_id, event} =>
         on_window_event(display, control_flow, window_id, event),
       Event::Resumed =>
         on_resume(display, control_flow),
       Event::Suspended =>
         on_suspend(display, control_flow),
       Event::RedrawRequested(_) | Event::MainEventsCleared => {
         on_redraw_requested(display, control_flow);
         do_redraw = true;
       }

       _ => (),
   }
   do_redraw
}