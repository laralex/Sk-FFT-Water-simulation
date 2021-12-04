use glium::backend::glutin;
use glium::{Frame, uniform};
use crate::camera::Camera;

pub trait Renderer {
   fn draw_to(&self, frame: &mut Frame, camera: &Camera);
}
pub struct WaterRenderer<'a> {
   mesh_grid_program: glium::Program,
   mesh_grid_vertices: glium::VertexBuffer<crate::mesh_grid::Vertex>,
   mesh_grid_indices: glium::index::NoIndices,
   draw_parameters: glium::DrawParameters<'a>,
}

impl<'a> WaterRenderer<'a> {
   pub fn new(display: &glutin::Display, mesh_grid_size: (usize, usize)) -> Self {
      let mesh_grid_program = crate::mesh_grid::make_program(display);
      let (mesh_grid_vertices, mesh_grid_indices)
         = crate::mesh_grid::make_tri_mesh(display, mesh_grid_size);
      let draw_parameters = glium::DrawParameters {
         depth: glium::Depth {
             test: glium::draw_parameters::DepthTest::IfLess,
             write: true,
             .. Default::default()
         },
         .. Default::default()
      };
      Self {
         mesh_grid_program,
         mesh_grid_vertices,
         mesh_grid_indices,
         draw_parameters,
      }
   }
}

impl<'a> Renderer for WaterRenderer<'a> {
   fn draw_to(&self, frame: &mut Frame, camera: &Camera) {
      use glium::Surface;
      let uniforms = &uniform! {model_view_projection:
         camera.get_view_projection().to_cols_array_2d()};
      frame.draw(
         &self.mesh_grid_vertices,
         &self.mesh_grid_indices,
         &self.mesh_grid_program,
         uniforms,
         &self.draw_parameters,
      ).unwrap()
   }
}