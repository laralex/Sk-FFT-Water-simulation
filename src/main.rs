// Entry point:
// Setting up a window, logging library
// Starting an infinite drawing loop

use std::borrow::Cow;

use imgui::{Key, MouseButton, CollapsingHeader};
use render::{Renderer, DrawMode};

// Link other source code files
mod shader;
mod render;
mod mesh_grid;
mod camera;
mod window;
mod wave;
mod height_field;
mod consts;

fn main() {
   // initisliaze logger
   use simplelog::*;
   TermLogger::init(LogLevelFilter::Info)
      .expect("Failed to initialize logger");

   let mut window = window::Window::new(
         consts::WINDOW_TITLE, (1600, 900), 13.0, None);
   
         // setting up default simulation parameters

   

   let mut water_size = 25;
   let mut water_cell_size = 0.05;
   let center_x = water_size as f32 * 0.5;
   let water_center = glam::vec3a(center_x, center_x, 0.0);
   let mut water = render::WaterRenderer::new(
      &window.display,
      (water_size, water_size),
      water_cell_size);

   let mut camera = camera::Camera::default();
   let default_camera_translation = glam::vec3a(0.0, -1.0, -1.0);
   let default_camera_direction = -glam::vec3a(1.0, 1.0, 1.0).normalize();
   camera
      .translate_to(default_camera_translation)
      .look_forward(default_camera_direction)
      ;

   let mut camera_steer_sensitivity = consts::CAMERA_DEFAULT_STEER_SENSITIVITY;
   let mut camera_orient_sensitivity = consts::CAMERA_DEFAULT_ORIENT_SENSITIVITY;
   let mut draw_mode = DrawMode::Wireframe;
   let (mut yaw, mut pitch) = (-90.0, 0.0);

   window.run_loop(move |run, ui, display, frame| {
      let mut camera_steer = glam::Vec3A::ZERO;
      let frame_time_sec = ui.io().delta_time;

      camera.with_perspective(
         f32::to_radians(consts::CAMERA_FIELD_OF_VIEW),
         window::Window::aspect_ratio(display),
         consts::CAMERA_NEAR_PLANE,
         consts::CAMERA_FAR_PLANE,
      );

      imgui::Window::new("Demo settings")
         .size([300.0, 300.0], imgui::Condition::FirstUseEver)
         .position([20.0, 20.0], imgui::Condition::Appearing)
         .opened(run)
         .size_constraints([300.0, 300.0], [600.0, 600.0])
         .resizable(true)
         .build(&ui, || {
               if ui.is_key_down(Key::LeftArrow) {
                  camera_steer.x = -1.0;
               }
               if ui.is_key_down(Key::RightArrow) {
                  camera_steer.x = 1.0;
               }
               if ui.is_key_down(Key::UpArrow) {
                  camera_steer.z = -1.0;
               }
               if ui.is_key_down(Key::DownArrow) {
                  camera_steer.z = 1.0;
               }

               let is_left_mouse = ui.is_mouse_down(MouseButton::Left) && !ui.is_window_focused();
               if is_left_mouse {
                  yaw += camera_orient_sensitivity*ui.io().mouse_delta[0];
                  pitch += camera_orient_sensitivity*ui.io().mouse_delta[1];
               }

               let is_right_mouse = ui.is_mouse_dragging(MouseButton::Right);
               if is_right_mouse {
                  camera_steer.y += f32::clamp(ui.io().mouse_delta[1] * camera_orient_sensitivity, -1.0, 1.0);
               }
               ui.text(format!(
                  "Usage:\n- Arrows L/R : Camera side-steer\n- Arrows U/D : Camera forward/back\n- Left Mouse Btn Drag: Camera rotation\n- Right Mouse Btn Drag: Camera up/down",
               ));
               if CollapsingHeader::new("Camera view").build(ui) {
                  if ui.button("Reset") {
                     camera
                        .translate_to(default_camera_translation)
                        .look_at(default_camera_direction);
                  }
                  // ui.item_size([])
                  imgui::Slider::new("Sensitivity", 0.5, 10.0)
                     .build(&ui, &mut camera_steer_sensitivity);

                  ui.radio_button("Render textured", &mut draw_mode, DrawMode::Mesh);
                  ui.radio_button("Render wireframe", &mut draw_mode, DrawMode::Wireframe);
               }

               if CollapsingHeader::new("Stats").build(ui) {
                  ui.text(format!(
                     "{:.1} ms, {:.1} fps",
                     frame_time_sec*1000.0, 1.0/frame_time_sec
                  ));
   
                  // let mouse_pos = ui.io().mouse_pos;
                  // ui.text(format!(
                  //    "Mouse position: (X: {:.1}, Y: {:.1})",
                  //    mouse_pos[0], mouse_pos[1]
                  // ));
   
                  let camera_pos = camera.position();
                  ui.text(format!(
                     "Camera position: (X: {:.1}, Y: {:.1}, Z: {:.1})",
                     camera_pos.x, camera_pos.y, camera_pos.z,
                  ));

               }

               if CollapsingHeader::new("Mesh")
                  .default_open(true).build(ui) {
                  let mut grid_size = water_size as i32;
                  let grid_size_changed = ui.input_int(
                     "Facets number (Lx/dx)", &mut grid_size).build();
                  let cell_size_changed = ui.input_float(
                     "Facet size (dx)", &mut water_cell_size).build();
                  let mut fft_domain_size_idx = 2;
                  let fft_domain_size_variants = vec![128, 256, 512, 1024];
                  let fft_domain_size_changed = ui.combo("Height field size (Nx)",
                  &mut fft_domain_size_idx, &fft_domain_size_variants,
                     |fft_size| Cow::Owned(fft_size.to_string()));
                  if grid_size_changed || cell_size_changed {
                     water_size = grid_size.clamp(4, 1024) as u32;
                     water_cell_size = water_cell_size.clamp(0.01, 1.0);
                     water.recreate_mesh_grid(display, (water_size, water_size), water_cell_size);
                  };
               }
         });

         let camera_direction = glam::vec3a(
            yaw.to_radians().cos(),
            0.0,
            yaw.to_radians().sin(),
         );

         camera
            .translate(camera_steer*camera_steer_sensitivity*frame_time_sec)
            .look_forward(camera_direction)
            ;
         water.set_draw_mode(draw_mode);
         water.draw_to(frame, &camera);
   });
}
