use render::Renderer;

// Entry point:
// Setting up a window, logging library
// Starting an infinite drawing loop

mod shader;
mod render;
mod mesh_grid;
mod camera;
mod window;
mod demo_state;

fn init_logger() {
   use simplelog::*;
   TermLogger::init(LogLevelFilter::Info)
      .expect("Failed to initialize logger");
}

fn main() {
   init_logger();
   let mut window = window::Window::new(
      "Final project - FFT water", (1366, 768), None);

   window.demo_state.water_size = 25;
   let center_x = window.demo_state.water_size as f32 * 0.5;
   let water_center = glam::vec3a(center_x, center_x, 0.0);
   let mut water = render::WaterRenderer::new(
      &window.display, (window.demo_state.water_size, window.demo_state.water_size));
   
   let mut camera = camera::Camera::default();
   camera.perspective(
      f32::to_radians(60.0), window.aspect_ratio(),
       0.01, 100.0);

   window.run_loop(move |run, ui, demo_state| {
      imgui::Window::new("Demo settings")
         .size([500.0, 150.0], imgui::Condition::Always)
         .position([20.0, 20.0], imgui::Condition::Appearing)
         .opened(run)
         .resizable(false)
         .build(&ui, || {
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
               use render::DrawMode;
               ui.radio_button("Render as mesh", &mut demo_state.draw_mode, DrawMode::Mesh);
               ui.radio_button("Render as wireframe", &mut demo_state.draw_mode, DrawMode::Wireframe);
               if imgui::Slider::new("Lattice size", u32::MIN, 1000)
                  .build(ui, &mut demo_state.water_size) {
                  demo_state.recreate_mesh_grid = true;
               };
         });
   },
   move |display, frame, demo_state| {
      let t = demo_state.world_time_sec;
      let orbit_t = (demo_state.water_size as f32)*glam::vec3a(0.2*t.cos(), 0.2*t.sin(), 1.0);
      if demo_state.recreate_mesh_grid {
         let size = (demo_state.water_size, demo_state.water_size);
         water.recreate_mesh_grid(display, size);
         demo_state.recreate_mesh_grid = false;
      }
      camera
         .translate_to(water_center + orbit_t);
      water.set_draw_mode(demo_state.draw_mode);
      water.draw_to(frame, &camera);
   });

}