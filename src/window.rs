// Creation of graphical window, handing of window/keyboard/mouse events
// Creation of user interface (ImGUI)

use glium::backend::Facade;
use glium::glutin;
use glium::Surface;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::ControlFlow;
use imgui::{FontConfig, FontSource, Ui};
use imgui_winit_support::{HiDpiMode};
use std::path::PathBuf;
use std::time::Instant;
use gl;
pub struct Window {
   pub event_loop: glutin::event_loop::EventLoop<()>,
   pub display: glium::Display,
   pub imgui: imgui::Context,
   pub platform: imgui_winit_support::WinitPlatform,
   pub renderer: imgui_glium_renderer::Renderer,
   pub font_size: f32,
}

impl Window {
   pub fn new(title: &str, (width, height): (u32, u32), font_size_pt: f64, imgui_ini_file: Option<PathBuf>) -> Self {
      let (display, event_loop) = make_display(title, width, height)
         .expect("Failed to create `glium::Display`");

      let mut imgui = imgui::Context::create();
      imgui.set_ini_filename(imgui_ini_file);

      let platform = init_winit(&display, &mut imgui);

      let font_size = init_fonts(&mut imgui,
         font_size_pt,
         platform.hidpi_factor());

      let renderer = imgui_glium_renderer::Renderer::init(&mut imgui, &display)
         .expect("Failed to initialize `imgui_glium_renderer::Renderer`");

      Self {
         event_loop, display, imgui, platform, renderer, font_size
      }
   }

   pub fn aspect_ratio(display: &glium::Display) -> f32 {
      let (width, height) = display.get_framebuffer_dimensions();
      width as f32 / height as f32
   }

   pub fn run_loop<F>(self, mut user_action: F)
    where F : FnMut(&mut bool, &mut Ui, &glium::Display, &mut glium::Frame) + 'static,  {
      let Window {
          event_loop,
          mut display,
          mut imgui,
          mut platform,
          renderer: mut ui_renderer,
          ..
      } = self;

      platform.handle_event::<()>(imgui.io_mut(), display.gl_window().window(), &Event::RedrawEventsCleared);
      let world_begin_time = Instant::now();
      let mut last_frame = world_begin_time;

      event_loop.run(move |event, _, control_flow| match event {
          Event::NewEvents(_) => {
            let now = Instant::now();
            let delta_time = now - last_frame;
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
            let gl_window = display.gl_window();
            platform.prepare_render(&ui, gl_window.window());

            let mut target = display.draw();
            target.clear_color_and_depth((0.05, 0.05, 0.05, 1.0), 1.0);

            let mut run = true;
            user_action(&mut run, &mut ui, &display, &mut target);
            if !run {
               *control_flow = ControlFlow::Exit;
            }

            // ui render
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
      .with_inner_size(glutin::dpi::PhysicalSize::new(width, height))
      .with_position(glutin::dpi::PhysicalPosition::new(80, 20))
      .with_title(title.to_owned())
      .with_resizable(false);

   let context_builder = ContextBuilder::new()
      .with_gl(GlRequest::Specific(Api::OpenGl, (4, 5)))
      .with_gl_profile(GlProfile::Core)
      .with_gl_robustness(Robustness::NoError)
      .with_pixel_format(8, 8)
      .with_depth_buffer(24)
      .with_double_buffer(Some(false))
      .with_vsync(false);

   let event_loop = glutin::event_loop::EventLoop::new();
   let display = glium::Display::new(
      window_builder, context_builder, &event_loop)?;

   // init opengl
   extern crate gl;
   use gl::types::*;
   unsafe {
      gl::load_with(|s| display.gl_window().context().get_proc_address(s) );
      gl::Viewport::load_with(|s| display.gl_window().context().get_proc_address(s));
   }

   Ok((display, event_loop))
}

fn init_winit(display: &glium::Display, imgui: &mut imgui::Context) -> imgui_winit_support::WinitPlatform {
   let mut platform = imgui_winit_support::WinitPlatform::init(imgui);
   let gl_window = display.gl_window();
   let window = gl_window.window();
   platform.attach_window(imgui.io_mut(), window, 
   HiDpiMode::Default);
   platform
}

fn init_fonts(imgui: &mut imgui::Context, font_size_pt: f64, hidpi_factor: f64) -> f32 {
   let font_size = (font_size_pt * hidpi_factor) as f32;
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