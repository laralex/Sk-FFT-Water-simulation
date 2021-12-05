pub struct DemoState {
   pub draw_mode: crate::render::DrawMode,
   pub recreate_mesh_grid: bool,
   pub world_time_sec: f32,
   pub delta_time_sec: f32,
   pub water_size: u32,
}

impl Default for DemoState {
   fn default() -> Self {
      Self {
         draw_mode: crate::render::DrawMode::Wireframe,
         recreate_mesh_grid: false,
         world_time_sec: 0.0,
         delta_time_sec: 0.0,
         water_size: 10,
      }
   }
}