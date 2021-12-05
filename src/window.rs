use glium::glutin;
use glium::Surface;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::ControlFlow;
use imgui::{FontConfig, FontSource, Ui};
use imgui_winit_support::{HiDpiMode};
use std::path::PathBuf;
use std::time::Instant;

use crate::demo_state::DemoState;

pub struct Window {
   pub event_loop: glutin::event_loop::EventLoop<()>,
   pub display: glium::Display,
   pub imgui: imgui::Context,
   pub platform: imgui_winit_support::WinitPlatform,
   pub renderer: imgui_glium_renderer::Renderer,
   pub font_size: f32,
   pub demo_state: crate::demo_state::DemoState
}

impl Window {
   pub fn new(title: &str, (width, height): (u32, u32), imgui_ini_file: Option<PathBuf>) -> Self {
      let (display, event_loop) = make_display(title, width, height)
         .expect("Failed to create `glium::Display`");

      let mut imgui = imgui::Context::create();
      imgui.set_ini_filename(imgui_ini_file);

      let platform = init_winit(&display, &mut imgui);

      let font_size = init_fonts(&mut imgui, platform.hidpi_factor());

      let renderer = imgui_glium_renderer::Renderer::init(&mut imgui, &display)
         .expect("Failed to initialize `imgui_glium_renderer::Renderer`");

      let demo_state = DemoState::default();

      Self {
         event_loop, display, imgui, platform, renderer, font_size, demo_state
      }
   }

   pub fn aspect_ratio(&self) -> f32 {
      let (width, height) = self.display.get_framebuffer_dimensions();
      width as f32 / height as f32
   }

   pub fn run_loop<UiF, DrawF>(self, mut run_ui: UiF, mut user_render: DrawF)
    where UiF  : FnMut(&mut bool, &mut Ui, &mut DemoState) + 'static,
          DrawF: FnMut(&glium::Display, &mut glium::Frame, &mut DemoState) + 'static  {
      let Window {
          event_loop,
          mut display,
          mut imgui,
          mut platform,
          renderer: mut ui_renderer,
          mut demo_state,
          ..
      } = self;
      let world_begin_time = Instant::now();
      let mut last_frame = world_begin_time;

      event_loop.run(move |event, _, control_flow| match event {
          Event::NewEvents(_) => {
              let now = Instant::now();
              let delta_time = now - last_frame;
              demo_state.delta_time_sec = delta_time.as_secs_f32();
              demo_state.world_time_sec = (now - world_begin_time).as_secs_f32();
              imgui.io_mut().update_delta_time(delta_time);
              last_frame = now;
          }
          Event::MainEventsCleared => {
              let gl_window = display.gl_window();
              platform
                  .prepare_frame(imgui.io_mut(), gl_window.window())
                  .expect("Failed to prepare frame");
              gl_window.window().request_redraw();
          }
          Event::RedrawRequested(_) => {
              let mut ui = imgui.frame();

              let mut run = true;
              run_ui(&mut run, &mut ui, &mut demo_state);
              if !run {
                  *control_flow = ControlFlow::Exit;
              }

              let gl_window = display.gl_window();
              let mut target = display.draw();
              target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);
         
               // TODO: I don't like passing display here
              user_render(&display, &mut target, &mut demo_state);

              // ui render
              platform.prepare_render(&ui, gl_window.window());
              let draw_data = ui.render();
              ui_renderer
                  .render(&mut target, draw_data)
                  .expect("Rendering failed");

              target.finish().expect("Failed to swap buffers");
          },
          Event::WindowEvent {event: WindowEvent::CloseRequested, ..} =>
            *control_flow = ControlFlow::Exit,
          event => {
            let gl_window = display.gl_window();
            platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
          }
      })
  }
}

type DisplayCreationResult = Result<
   (glium::Display, glutin::event_loop::EventLoop<()>),
   glium::backend::glutin::DisplayCreationError>;

pub fn make_display(title: &str, width: u32, height: u32) -> DisplayCreationResult {
   use glium::glutin::*;

   let window_builder = glutin::window::WindowBuilder::new()
      .with_inner_size(glutin::dpi::LogicalSize::new(width, height))
      .with_title(title.to_owned())
      .with_resizable(true);

   let context_builder = ContextBuilder::new()
      .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
      .with_gl_profile(GlProfile::Core)
      .with_gl_robustness(Robustness::TryRobustLoseContextOnReset)
      .with_pixel_format(8, 8)
      .with_depth_buffer(24)
      .with_double_buffer(Some(true))
      .with_vsync(true);

   let event_loop = glutin::event_loop::EventLoop::new();
   let display = glium::Display::new(
      window_builder, context_builder, &event_loop)?;
   Ok((display, event_loop))
}

fn init_winit(display: &glium::Display, imgui: &mut imgui::Context) -> imgui_winit_support::WinitPlatform {
   let mut platform = imgui_winit_support::WinitPlatform::init(imgui);
   let gl_window = display.gl_window();
   let window = gl_window.window();
   platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
   platform
}

fn init_fonts(imgui: &mut imgui::Context, hidpi_factor: f64) -> f32 {
   let font_size = (13.0 * hidpi_factor) as f32;
   imgui.fonts().add_font(&[
      FontSource::DefaultFontData {
         config: Some(FontConfig {
               size_pixels: font_size,
               ..FontConfig::default()
         }),
      },
   ]);

   imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
   font_size
}