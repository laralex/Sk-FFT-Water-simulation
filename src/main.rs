use std::ops::MulAssign;
use glam::Vec3A;
use imgui::Key;
use render::{Renderer, DrawMode};

// Entry point:
// Setting up a window, logging library
// Starting an infinite drawing loop

mod shader;
mod render;
mod mesh_grid;
mod camera;
mod window;

fn init_logger() {
   use simplelog::*;
   TermLogger::init(LogLevelFilter::Info)
      .expect("Failed to initialize logger");
}

fn main() {
   init_logger();
   let mut window = window::Window::new(
      "Final project - FFT water", (1366, 768), None);

   let mut water_size = 25;
   let mut water_cell_size = 0.05;
   let center_x = water_size as f32 * 0.5;
   let water_center = glam::vec3a(center_x, center_x, 0.0);
   let mut water = render::WaterRenderer::new(
      &window.display, 
      (water_size, water_size),
      water_cell_size);
   
   let mut camera = camera::Camera::default();
   camera.perspective(
      f32::to_radians(60.0), window.aspect_ratio(),0.01, 100.0)
      .translate(3.0*glam::Vec3A::Z);

   let mut camera_steer_sensitivity = 0.1;
   let mut camera_orient_sensitivity = 0.2;
   let mut draw_mode = DrawMode::Wireframe;

   window.run_loop(move |run, ui, display, frame| {
      let mut camera_steer = glam::Vec3A::ZERO;
      let mut camera_orient = glam::Quat::IDENTITY;

      imgui::Window::new("Demo settings")
         .size([550.0, 300.0], imgui::Condition::FirstUseEver)
         .position([20.0, 20.0], imgui::Condition::Appearing)
         .opened(run)
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

               camera_orient.mul_assign(glam::Quat::from_euler(
                     glam::EulerRot::XYZ,
                     ui.io().mouse_delta[0],
                     ui.io().mouse_delta[1],
                     0.0));

               ui.text(format!(
                  r#"Usage:
- Arrows Left/Right : Steers camera to the sides
- Arrows Up/Down    : Steers camera forward/backward
- Left Mouse Button : Mouse rotates camera
"#,
               ));
               ui.separator();

               let frame_time_sec = ui.io().delta_time;
               ui.text(format!(
                  "{:.1} ms, {:.1} fps",
                  frame_time_sec*1000.0, 1.0/frame_time_sec
               ));

               let mouse_pos = ui.io().mouse_pos;
               ui.text(format!(
                  "Mouse Position: ({:.1},{:.1})",
                  mouse_pos[0], mouse_pos[1]
               ));
               ui.separator();
               if ui.button("Reset camera") {
                  camera.translate_to(-10.0*glam::Vec3A::Z);
               }
               imgui::Slider::new("Camera steer sensitivity", 0.01, 1.0)
                  .build(&ui, &mut camera_steer_sensitivity);
               imgui::Slider::new("Camera rotation sensitivity", 0.01, 1.0)
                  .build(&ui, &mut camera_orient_sensitivity);
               ui.separator();
               ui.radio_button("Render as mesh", &mut draw_mode, DrawMode::Mesh);
               ui.radio_button("Render as wireframe", &mut draw_mode, DrawMode::Wireframe);
               let grid_size_changed = imgui::Slider::new("Lattice size", u32::MIN, 1000)
                  .build(&ui, &mut water_size);
               let cell_size_changed = imgui::Slider::new("Cell size", 0.01, 1.0)
                  .build(&ui, &mut water_cell_size);
               if grid_size_changed || cell_size_changed {
                  water.recreate_mesh_grid(display, (water_size, water_size), water_cell_size);
               };
         });

         camera
            .translate(camera_steer*camera_steer_sensitivity)
            //.look_at(water_center)
            ;
         water.set_draw_mode(draw_mode);
         water.draw_to(frame, &camera);
   });
}
