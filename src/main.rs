// Entry point:
// Setting up a window, logging library
// Starting an infinite drawing loop

mod event;
mod program;
mod render;
mod mesh_grid;
mod camera;

use std::ops::Div;

use glium::Surface;
use glium::backend::glutin::DisplayCreationError;
use glium::glutin::event_loop::EventLoop;
use render::Renderer;

fn init_logger() {
   use simplelog::*;
   TermLogger::init(LogLevelFilter::Info).unwrap();
}

fn make_display(width: u32, height: u32) -> Result<(glium::Display, EventLoop<()>), DisplayCreationError>{
   use glium::glutin::{window, dpi, *};

   let window_builder = window::WindowBuilder::new()
      .with_inner_size(dpi::PhysicalSize::new(width, height))
      .with_title("FFT water project")
      .with_resizable(true);

   let context_builder = ContextBuilder::new()
      .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
      .with_gl_profile(GlProfile::Compatibility)
      .with_gl_robustness(Robustness::TryRobustLoseContextOnReset)
      .with_pixel_format(8, 8)
      .with_double_buffer(Some(true))
      .with_depth_buffer(24);

   let event_loop = EventLoop::new();
   let display = glium::Display::new(
      window_builder, context_builder, &event_loop)?;
   Ok((display, event_loop))
}

fn main() {
   init_logger();
   let (screen_w, screen_h) = (1366, 768);
   let aspect_ratio = (screen_w as f32) / (screen_h as f32);
   let (mut display, event_loop) = make_display(screen_w, screen_h).unwrap();

   let water_size = 10;
   let water_half_size = water_size as f32 / 2.0;
   let water_center = glam::vec3(water_half_size, water_half_size, 0.0);
   let water = render::WaterRenderer::new(&display, (water_size, water_size));
   let mut camera = camera::Camera::default();
   camera
      .perspective(f32::to_radians(60.0), aspect_ratio, 0.01, 100.0);

   let mut t: f32 = 0.0;

   // infinite draw loop
   event_loop.run(move |event, _, control_flow| {
      // poll window events, decide action
      let do_draw = event::handle_event(event, &mut display, control_flow);
      if !do_draw { return; }

      // state step
      t += 0.01;
      camera
         .translate_to(-water_center + water_half_size*glam::vec3(t.sin(), t.cos(), -2.0))
         .look_at(water_center)
         ;
      // log::info!("camera {}", camera.get_view().translation);

      // render step
      let mut frame = display.draw();
      frame.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);
      water.draw_to(&mut frame, &camera);
      frame.finish().unwrap();

   });
}